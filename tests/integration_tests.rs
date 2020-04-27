extern crate gbfs_rs;

use gbfs_rs::*;

use std::fs::File;
use std::io::Read;

const TEST_FS_DATA: &'static [u8] = include_bytes!("../test_assets/assets.gbfs");
const TEST_FS: GBFSFilesystem<'static> = const_fs(TEST_FS_DATA);
const NUM_FILES_IN_TEST_FS: usize = 570;

#[test]
fn open_gbfs() {
    let mut file = File::open("test_assets/assets.gbfs").unwrap();
    let mut test_data = Vec::new();
    file.read_to_end(&mut test_data).unwrap();
    let _gbfs = GBFSFilesystem::from_slice(test_data.as_ref());
}

#[test]
fn read_file_by_name() {
    let mut file = File::open("test_assets/assets.gbfs").unwrap();
    let mut test_data = Vec::new();
    file.read_to_end(&mut test_data).unwrap();
    let gbfs = GBFSFilesystem::from_slice(test_data.as_ref());
    let filename = FilenameString::try_from_str("copper1Tiles").unwrap();
    let gbfs_file = gbfs.get_file_by_name(filename).unwrap();
    assert_eq!(gbfs_file.filename.as_string(), filename);
    assert_eq!(gbfs_file.data.len(), 256);
}

#[test]
fn read_file_by_name_const_fs() {
    let filename = FilenameString::try_from_str("copper1Tiles").unwrap();
    assert_eq!(
        TEST_FS
            .get_file_by_name(filename)
            .unwrap()
            .filename
            .as_string(),
        filename
    );
}

#[test]
fn iterate_const_fs() {
    let files: Vec<gbfs_rs::File> = TEST_FS.into_iter().collect();
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
    let gbfs = GBFSFilesystem::from_slice(test_data.as_ref());
    let filename = FilenameString::try_from_str("copper1Tiles").unwrap();
    let gbfs_file = gbfs.get_file_by_name(filename).unwrap();
    assert_eq!(gbfs_file.filename.as_string(), filename);
    assert_eq!(gbfs_file.data.len(), 256);
    let gbfs_file_as_u16 = gbfs_file.to_u16_vec();
    assert_eq!(gbfs_file_as_u16.len(), 128);
}

#[test]
fn read_file_as_u32() {
    let mut file = File::open("test_assets/assets.gbfs").unwrap();
    let mut test_data = Vec::new();
    file.read_to_end(&mut test_data).unwrap();
    let gbfs = GBFSFilesystem::from_slice(test_data.as_ref());
    let filename = FilenameString::try_from_str("copper1Tiles").unwrap();
    let gbfs_file = gbfs.get_file_by_name(filename).unwrap();
    assert_eq!(gbfs_file.filename.as_string(), filename);
    assert_eq!(gbfs_file.data.len(), 256);
    let gbfs_file_as_u32 = gbfs_file.to_u32_vec();
    assert_eq!(gbfs_file_as_u32.len(), 64);
}
