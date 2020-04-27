extern crate gbfs_rs;

use gbfs_rs::*;

use std::fs::File;
use std::io::Read;

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
    let filename = Filename::try_from_str("copper1Tiles").unwrap();
    let gbfs_file = gbfs.get_file_by_name(filename).unwrap();
    assert_eq!(gbfs_file.filename, filename);
    assert_eq!(gbfs_file.data.len(), 256);
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
    let filename = Filename::try_from_str("copper1Tiles").unwrap();
    let gbfs_file = gbfs.get_file_by_name(filename).unwrap();
    assert_eq!(gbfs_file.filename, filename);
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
    let filename = Filename::try_from_str("copper1Tiles").unwrap();
    let gbfs_file = gbfs.get_file_by_name(filename).unwrap();
    assert_eq!(gbfs_file.filename, filename);
    assert_eq!(gbfs_file.data.len(), 256);
    let gbfs_file_as_u32 = gbfs_file.to_u32_vec();
    assert_eq!(gbfs_file_as_u32.len(), 64);
}
