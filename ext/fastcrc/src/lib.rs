use crc64fast_nvme::Digest as Crc64NvmeDigest;
use magnus::{function, method, prelude::*, Error, RString, Ruby};
use std::cell::RefCell;

fn bytes_from_rstring(input: &RString) -> &[u8] {
    // Safe because we do not yield to Ruby while holding the slice reference.
    unsafe { input.as_slice() }
}

fn crc32_checksum(input: RString) -> u32 {
    crc32fast::hash(bytes_from_rstring(&input))
}

fn crc32_hexdigest(input: RString) -> String {
    format!("{:08x}", crc32_checksum(input))
}

fn crc64_nvme_checksum(input: RString) -> u64 {
    let mut digest = Crc64NvmeDigest::new();
    digest.write(bytes_from_rstring(&input));
    digest.sum64()
}

fn crc64_nvme_hexdigest(input: RString) -> String {
    format!("{:016x}", crc64_nvme_checksum(input))
}

#[magnus::wrap(class = "FastCRC::CRC32", free_immediately, size)]
struct Crc32 {
    hasher: RefCell<crc32fast::Hasher>,
}

impl Crc32 {
    fn new() -> Self {
        Self {
            hasher: RefCell::new(crc32fast::Hasher::new()),
        }
    }

    fn update(&self, input: RString) {
        self.hasher
            .borrow_mut()
            .update(bytes_from_rstring(&input));
    }

    fn checksum(&self) -> u32 {
        self.hasher.borrow().clone().finalize()
    }

    fn hexdigest(&self) -> String {
        format!("{:08x}", self.checksum())
    }

    fn reset(&self) {
        self.hasher.borrow_mut().reset();
    }
}

#[magnus::wrap(class = "FastCRC::CRC64NVME", free_immediately, size)]
struct Crc64Nvme {
    digest: RefCell<Crc64NvmeDigest>,
}

impl Crc64Nvme {
    fn new() -> Self {
        Self {
            digest: RefCell::new(Crc64NvmeDigest::new()),
        }
    }

    fn update(&self, input: RString) {
        self.digest
            .borrow_mut()
            .write(bytes_from_rstring(&input));
    }

    fn checksum(&self) -> u64 {
        self.digest.borrow().sum64()
    }

    fn hexdigest(&self) -> String {
        format!("{:016x}", self.checksum())
    }

    fn reset(&self) {
        *self.digest.borrow_mut() = Crc64NvmeDigest::new();
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
