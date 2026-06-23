use crc_fast::{checksum, checksum_with_params, CrcAlgorithm, CrcParams, Digest};
use magnus::{function, method, prelude::*, Error, RClass, RString, Ruby};
use std::cell::RefCell;

fn bytes_from_rstring(input: &RString) -> &[u8] {
    // Safe because we do not yield to Ruby while holding the slice reference.
    unsafe { input.as_slice() }
}

fn format_checksum(checksum: u64, hex_width: usize) -> String {
    format!("{:0width$x}", checksum, width = hex_width)
}

fn crc64_params() -> CrcParams {
    CrcParams::new(
        "CRC-64",
        64,
        0x0000_0000_0000_001b,
        0x0000_0000_0000_0000,
        true,
        0x0000_0000_0000_0000,
        0x46a5_a938_8a5b_effe,
    )
}

fn crc64_jones_params() -> CrcParams {
    CrcParams::new(
        "CRC-64/JONES",
        64,
        0xad93_d235_94c9_35a9,
        0xffff_ffff_ffff_ffff,
        true,
        0x0000_0000_0000_0000,
        0xcaa7_1716_8609_f281,
    )
}

macro_rules! impl_crc_methods {
    ($struct_name:ident, $hex_width:expr) => {
        impl_crc_methods!($struct_name, $hex_width, 0u64);
    };

    ($struct_name:ident, $hex_width:expr, $hexdigest_xor:expr) => {
        fn update(&self, input: RString) {
            self.digest.borrow_mut().update(bytes_from_rstring(&input));
        }

        fn checksum(&self) -> u64 {
            self.digest.borrow().finalize()
        }

        fn hexdigest(&self) -> String {
            format_checksum(self.checksum() ^ $hexdigest_xor, $hex_width)
        }

        fn reset(&self) {
            self.digest.borrow_mut().reset();
        }
    };
}

macro_rules! define_known_crc {
    ($struct_name:ident, $ruby_class:literal, $algorithm:expr, $hex_width:expr) => {
        #[magnus::wrap(class = $ruby_class, free_immediately, size)]
        struct $struct_name {
            digest: RefCell<Digest>,
        }

        impl $struct_name {
            fn new() -> Self {
                Self {
                    digest: RefCell::new(Digest::new($algorithm)),
                }
            }

            fn checksum_once(input: RString) -> u64 {
                checksum($algorithm, bytes_from_rstring(&input))
            }

            fn hexdigest_once(input: RString) -> String {
                format_checksum(Self::checksum_once(input), $hex_width)
            }

            impl_crc_methods!($struct_name, $hex_width);
        }
    };
}

macro_rules! define_custom_crc {
    ($struct_name:ident, $ruby_class:literal, $params_fn:ident, $hex_width:expr) => {
        define_custom_crc!($struct_name, $ruby_class, $params_fn, $hex_width, 0u64);
    };

    (
        $struct_name:ident,
        $ruby_class:literal,
        $params_fn:ident,
        $hex_width:expr,
        $hexdigest_xor:expr
    ) => {
        #[magnus::wrap(class = $ruby_class, free_immediately, size)]
        struct $struct_name {
            digest: RefCell<Digest>,
        }

        impl $struct_name {
            fn new() -> Self {
                Self {
                    digest: RefCell::new(Digest::new_with_params($params_fn())),
                }
            }

            fn checksum_once(input: RString) -> u64 {
                checksum_with_params($params_fn(), bytes_from_rstring(&input))
            }

            fn hexdigest_once(input: RString) -> String {
                format_checksum(Self::checksum_once(input) ^ $hexdigest_xor, $hex_width)
            }

            impl_crc_methods!($struct_name, $hex_width, $hexdigest_xor);
        }
    };
}

macro_rules! register_crc_class {
    (
        $module:expr,
        $ruby:expr,
        $class_name:literal,
        $struct_name:ident,
        $width:expr,
        $check:expr,
        $catalog_name:literal
    ) => {{
        let class = $module.define_class($class_name, $ruby.class_object())?;
        class.define_singleton_method("new", function!($struct_name::new, 0))?;
        class.define_singleton_method("checksum", function!($struct_name::checksum_once, 1))?;
        class.define_singleton_method("hexdigest", function!($struct_name::hexdigest_once, 1))?;
        class.define_method("update", method!($struct_name::update, 1))?;
        class.define_method("checksum", method!($struct_name::checksum, 0))?;
        class.define_method("hexdigest", method!($struct_name::hexdigest, 0))?;
        class.define_method("reset", method!($struct_name::reset, 0))?;
        class.const_set("WIDTH", $width)?;
        class.const_set("CHECK", $check)?;
        class.const_set("NAME", $catalog_name)?;
        class
    }};
}

fn crc16_dnp_table_entry(index: u8) -> u16 {
    let mut value = u16::from(index);

    for _ in 0..8 {
        value = if value & 1 == 1 {
            (value >> 1) ^ 0xa6bc
        } else {
            value >> 1
        };
    }

    value
}

fn crc16_dnp_update(mut checksum: u16, data: &[u8]) -> u16 {
    for byte in data {
        let table_index = ((checksum ^ u16::from(*byte)) & 0xff) as u8;
        checksum = ((((u32::from(checksum)) << 8) ^ u32::from(crc16_dnp_table_entry(table_index)))
            & 0xffff) as u16;
    }

    checksum
}

#[magnus::wrap(class = "FastCRC::CRC16DNP", free_immediately, size)]
struct Crc16Dnp {
    checksum: RefCell<u16>,
}

impl Crc16Dnp {
    fn new() -> Self {
        Self {
            checksum: RefCell::new(0),
        }
    }

    fn checksum_once(input: RString) -> u64 {
        u64::from(crc16_dnp_update(0, bytes_from_rstring(&input)))
    }

    fn hexdigest_once(input: RString) -> String {
        format_checksum(Self::checksum_once(input) ^ 0xffff, 4)
    }

    fn update(&self, input: RString) {
        let mut checksum = self.checksum.borrow_mut();
        *checksum = crc16_dnp_update(*checksum, bytes_from_rstring(&input));
    }

    fn checksum(&self) -> u64 {
        u64::from(*self.checksum.borrow())
    }

    fn hexdigest(&self) -> String {
        format_checksum(self.checksum() ^ 0xffff, 4)
    }

    fn reset(&self) {
        *self.checksum.borrow_mut() = 0;
    }
}

define_known_crc!(Crc16, "FastCRC::CRC16", CrcAlgorithm::Crc16Arc, 4);
define_known_crc!(
    Crc16Ccitt,
    "FastCRC::CRC16CCITT",
    CrcAlgorithm::Crc16Ibm3740,
    4
);
define_known_crc!(
    Crc16Genibus,
    "FastCRC::CRC16Genibus",
    CrcAlgorithm::Crc16Genibus,
    4
);
define_known_crc!(
    Crc16Kermit,
    "FastCRC::CRC16Kermit",
    CrcAlgorithm::Crc16Kermit,
    4
);
define_known_crc!(
    Crc16Modbus,
    "FastCRC::CRC16Modbus",
    CrcAlgorithm::Crc16Modbus,
    4
);
define_known_crc!(Crc16Qt, "FastCRC::CRC16QT", CrcAlgorithm::Crc16IbmSdlc, 4);
define_known_crc!(Crc16Usb, "FastCRC::CRC16USB", CrcAlgorithm::Crc16Usb, 4);
define_known_crc!(Crc16X25, "FastCRC::CRC16X25", CrcAlgorithm::Crc16IbmSdlc, 4);
define_known_crc!(
    Crc16XModem,
    "FastCRC::CRC16XModem",
    CrcAlgorithm::Crc16Xmodem,
    4
);
define_known_crc!(
    Crc16ZModem,
    "FastCRC::CRC16ZModem",
    CrcAlgorithm::Crc16Xmodem,
    4
);

define_known_crc!(Crc32, "FastCRC::CRC32", CrcAlgorithm::Crc32IsoHdlc, 8);
define_known_crc!(
    Crc32Bzip2,
    "FastCRC::CRC32BZip2",
    CrcAlgorithm::Crc32Bzip2,
    8
);
define_known_crc!(Crc32C, "FastCRC::CRC32c", CrcAlgorithm::Crc32Iscsi, 8);
define_known_crc!(Crc32Jam, "FastCRC::CRC32Jam", CrcAlgorithm::Crc32Jamcrc, 8);
define_known_crc!(Crc32Mpeg, "FastCRC::CRC32MPEG", CrcAlgorithm::Crc32Mpeg2, 8);
define_known_crc!(
    Crc32Posix,
    "FastCRC::CRC32POSIX",
    CrcAlgorithm::Crc32Cksum,
    8
);
define_known_crc!(Crc32Xfer, "FastCRC::CRC32XFER", CrcAlgorithm::Crc32Xfer, 8);

define_custom_crc!(Crc64, "FastCRC::CRC64", crc64_params, 16);
define_custom_crc!(Crc64Jones, "FastCRC::CRC64Jones", crc64_jones_params, 16);
define_known_crc!(Crc64Nvme, "FastCRC::CRC64NVMe", CrcAlgorithm::Crc64Nvme, 16);
define_known_crc!(Crc64Xz, "FastCRC::CRC64XZ", CrcAlgorithm::Crc64Xz, 16);

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let fast_crc = ruby.define_module("FastCRC")?;

    register_crc_class!(fast_crc, ruby, "CRC16", Crc16, 16, 0xbb3du64, "CRC-16/ARC");
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC16CCITT",
        Crc16Ccitt,
        16,
        0x29b1u64,
        "CRC-16/IBM-3740"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC16DNP",
        Crc16Dnp,
        16,
        0xbeffu64,
        "CRC-16/DNP"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC16Genibus",
        Crc16Genibus,
        16,
        0xd64eu64,
        "CRC-16/GENIBUS"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC16Kermit",
        Crc16Kermit,
        16,
        0x2189u64,
        "CRC-16/KERMIT"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC16Modbus",
        Crc16Modbus,
        16,
        0x4b37u64,
        "CRC-16/MODBUS"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC16QT",
        Crc16Qt,
        16,
        0x906eu64,
        "CRC-16/IBM-SDLC"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC16USB",
        Crc16Usb,
        16,
        0xb4c8u64,
        "CRC-16/USB"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC16X25",
        Crc16X25,
        16,
        0x906eu64,
        "CRC-16/IBM-SDLC"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC16XModem",
        Crc16XModem,
        16,
        0x31c3u64,
        "CRC-16/XMODEM"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC16ZModem",
        Crc16ZModem,
        16,
        0x31c3u64,
        "CRC-16/XMODEM"
    );

    register_crc_class!(
        fast_crc,
        ruby,
        "CRC32",
        Crc32,
        32,
        0xcbf4_3926u64,
        "CRC-32/ISO-HDLC"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC32BZip2",
        Crc32Bzip2,
        32,
        0xfc89_1918u64,
        "CRC-32/BZIP2"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC32c",
        Crc32C,
        32,
        0xe306_9283u64,
        "CRC-32/ISCSI"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC32Jam",
        Crc32Jam,
        32,
        0x340b_c6d9u64,
        "CRC-32/JAMCRC"
    );
    let crc32_mpeg: RClass = register_crc_class!(
        fast_crc,
        ruby,
        "CRC32MPEG",
        Crc32Mpeg,
        32,
        0x0376_e6e7u64,
        "CRC-32/MPEG-2"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC32POSIX",
        Crc32Posix,
        32,
        0x765e_7680u64,
        "CRC-32/CKSUM"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC32XFER",
        Crc32Xfer,
        32,
        0xbd0b_e338u64,
        "CRC-32/XFER"
    );

    register_crc_class!(
        fast_crc,
        ruby,
        "CRC64",
        Crc64,
        64,
        0x46a5_a938_8a5b_effe_u64,
        "CRC-64"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC64Jones",
        Crc64Jones,
        64,
        0xcaa7_1716_8609_f281u64,
        "CRC-64/JONES"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC64NVMe",
        Crc64Nvme,
        64,
        0xae8b_1486_0a79_9888u64,
        "CRC-64/NVME"
    );
    register_crc_class!(
        fast_crc,
        ruby,
        "CRC64XZ",
        Crc64Xz,
        64,
        0x995d_c9bb_df19_39fau64,
        "CRC-64/XZ"
    );

    fast_crc.const_set("CRC32Mpeg", crc32_mpeg)?;

    Ok(())
}
