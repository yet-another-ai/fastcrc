use crc64fast_nvme::Digest as Crc64NvmeDigest;
use magnus::{function, prelude::*, Error, RString, Ruby};

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

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let fast_crc = ruby.define_module("FastCRC")?;

    let crc32 = fast_crc.define_class("CRC32", ruby.class_object())?;
    crc32.define_singleton_method("checksum", function!(crc32_checksum, 1))?;
    crc32.define_singleton_method("hexdigest", function!(crc32_hexdigest, 1))?;

    let crc64_nvme = fast_crc.define_class("CRC64NVME", ruby.class_object())?;
    crc64_nvme.define_singleton_method("checksum", function!(crc64_nvme_checksum, 1))?;
    crc64_nvme.define_singleton_method("hexdigest", function!(crc64_nvme_hexdigest, 1))?;

    Ok(())
}
