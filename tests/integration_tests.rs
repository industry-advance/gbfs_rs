#![feature(const_panic)]

extern crate gbfs_rs;

use gbfs_rs::*;

use std::fs::File;
use std::io::Read;

const TEST_FS_DATA: &'static [u8] = include_bytes!("../test_assets/assets.gbfs");
const TEST_FS: GBFSFilesystem<'static> = match GBFSFilesystem::from_slice(TEST_FS_DATA) {
    Ok(val) => val,
    // FIXME: Stop being lazy and implement a const fn-compatible mechanism to turn the error into &str
    Err(_) => panic!("Failed to construct filesystem!"),
};
const NUM_FILES_IN_TEST_FS: usize = 570;

#[test]
fn open_gbfs() {
    let mut file = File::open("test_assets/assets.gbfs").unwrap();
    let mut test_data = Vec::new();
    file.read_to_end(&mut test_data).unwrap();
    let _gbfs = GBFSFilesystem::from_slice(test_data.as_ref());
}

#[test]
fn read_file_data_by_name() {
    let mut file = File::open("test_assets/assets.gbfs").unwrap();
    let mut test_data = Vec::new();
    file.read_to_end(&mut test_data).unwrap();
    let gbfs = GBFSFilesystem::from_slice(test_data.as_ref()).unwrap();
    let file_data = gbfs.get_file_data_by_name("copper1Tiles").unwrap();
    assert_eq!(file_data.len(), 256);
}

#[test]
fn read_file_data_by_name_const_fs() {
    let data: &'static [u8] = TEST_FS.get_file_data_by_name("copper1Tiles").unwrap();
    assert_eq!(data.len(), 256);
}

#[test]
fn iterate_const_fs() {
    let files: Vec<&[u8]> = TEST_FS.into_iter().collect();
    assert_eq!(files.len(), NUM_FILES_IN_TEST_FS);
}

#[test]
fn file_iterator() {
    let mut file = File::open("test_assets/assets.gbfs").unwrap();
    let mut test_data = Vec::new();
    file.read_to_end(&mut test_data).unwrap();
    let gbfs = GBFSFilesystem::from_slice(test_data.as_ref());
    let mut iter = gbfs.into_iter();
    let _first = iter.next().unwrap();
}
#[test]
fn read_file_as_u16() {
    let mut file = File::open("test_assets/assets.gbfs").unwrap();
    let mut test_data = Vec::new();
    file.read_to_end(&mut test_data).unwrap();
    let gbfs = GBFSFilesystem::from_slice(test_data.as_ref()).unwrap();
    let gbfs_file = gbfs
        .get_file_data_by_name_as_u16_slice("copper1Tiles")
        .unwrap();
    assert_eq!(gbfs_file.len(), 128);
}
#[test]
fn read_file_as_u32() {
    let mut file = File::open("test_assets/assets.gbfs").unwrap();
    let mut test_data = Vec::new();
    file.read_to_end(&mut test_data).unwrap();
    let gbfs = GBFSFilesystem::from_slice(test_data.as_ref()).unwrap();
    let gbfs_file = gbfs
        .get_file_data_by_name_as_u32_slice("copper1Tiles")
        .unwrap();
    assert_eq!(gbfs_file.len(), 64);
}
