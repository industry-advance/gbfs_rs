#![no_main]
use gbfs::GBFSFilesystem;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    GBFSFilesystem::from_slice(data);
});
