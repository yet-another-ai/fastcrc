use crc_fast::{crc32_iso_hdlc, crc64_nvme, CrcAlgorithm, Digest};
use magnus::{function, method, prelude::*, Error, RString, Ruby};
use std::cell::RefCell;

fn bytes_from_rstring(input: &RString) -> &[u8] {
    // Safe because we do not yield to Ruby while holding the slice reference.
    unsafe { input.as_slice() }
}

fn crc32_checksum(input: RString) -> u32 {
    crc32_iso_hdlc(bytes_from_rstring(&input))
}

fn crc32_hexdigest(input: RString) -> String {
    format!("{:08x}", crc32_checksum(input))
}

fn crc64_nvme_checksum(input: RString) -> u64 {
    crc64_nvme(bytes_from_rstring(&input))
}

fn crc64_nvme_hexdigest(input: RString) -> String {
    format!("{:016x}", crc64_nvme_checksum(input))
}

#[magnus::wrap(class = "FastCRC::CRC32", free_immediately, size)]
struct Crc32 {
    digest: RefCell<Digest>,
}

impl Crc32 {
    fn new() -> Self {
        Self {
            digest: RefCell::new(Digest::new(CrcAlgorithm::Crc32IsoHdlc)),
        }
    }

    fn update(&self, input: RString) {
        self.digest.borrow_mut().update(bytes_from_rstring(&input));
    }

    fn checksum(&self) -> u32 {
        self.digest.borrow().finalize() as u32
    }

    fn hexdigest(&self) -> String {
        format!("{:08x}", self.checksum())
    }

    fn reset(&self) {
        self.digest.borrow_mut().reset();
    }
}

#[magnus::wrap(class = "FastCRC::CRC64NVME", free_immediately, size)]
struct Crc64Nvme {
    digest: RefCell<Digest>,
}

impl Crc64Nvme {
    fn new() -> Self {
        Self {
            digest: RefCell::new(Digest::new(CrcAlgorithm::Crc64Nvme)),
        }
    }

    fn update(&self, input: RString) {
        self.digest.borrow_mut().update(bytes_from_rstring(&input));
    }

    fn checksum(&self) -> u64 {
        self.digest.borrow().finalize()
    }

    fn hexdigest(&self) -> String {
        format!("{:016x}", self.checksum())
    }

    fn reset(&self) {
        self.digest.borrow_mut().reset();
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let fast_crc = ruby.define_module("FastCRC")?;

    let crc32 = fast_crc.define_class("CRC32", ruby.class_object())?;
    crc32.define_singleton_method("new", function!(Crc32::new, 0))?;
    crc32.define_singleton_method("checksum", function!(crc32_checksum, 1))?;
    crc32.define_singleton_method("hexdigest", function!(crc32_hexdigest, 1))?;
    crc32.define_method("update", method!(Crc32::update, 1))?;
    crc32.define_method("checksum", method!(Crc32::checksum, 0))?;
    crc32.define_method("hexdigest", method!(Crc32::hexdigest, 0))?;
    crc32.define_method("reset", method!(Crc32::reset, 0))?;

    let crc64_nvme = fast_crc.define_class("CRC64NVME", ruby.class_object())?;
    crc64_nvme.define_singleton_method("new", function!(Crc64Nvme::new, 0))?;
    crc64_nvme.define_singleton_method("checksum", function!(crc64_nvme_checksum, 1))?;
    crc64_nvme.define_singleton_method("hexdigest", function!(crc64_nvme_hexdigest, 1))?;
    crc64_nvme.define_method("update", method!(Crc64Nvme::update, 1))?;
    crc64_nvme.define_method("checksum", method!(Crc64Nvme::checksum, 0))?;
    crc64_nvme.define_method("hexdigest", method!(Crc64Nvme::hexdigest, 0))?;
    crc64_nvme.define_method("reset", method!(Crc64Nvme::reset, 0))?;

    Ok(())
}
