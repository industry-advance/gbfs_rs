#![no_std]
#![forbid(unsafe_code)]
#![feature(const_if_match)]
#![feature(const_loop)]
#![feature(const_fn)]
#![feature(const_panic)]
#![feature(const_mut_refs)]
#![feature(const_in_array_repeat_expressions)]

mod constfs;
mod file;
mod header;
pub use crate::constfs::*;
pub use crate::file::*;
use header::*;

extern crate alloc;
extern crate core;

use core::convert::TryInto;
use core::u32;

use alloc::vec::Vec;

const FILENAME_LEN: usize = 24;
const DIR_ENTRY_LEN: usize = 32;

#[derive(Debug, Clone)]
struct GBFSFileEntry {
    /// Name of file; at most 24 bytes
    name: Filename,
    /// Length of file in bytes
    len: u32,
    /// Offset of first file byte from start of filesystem
    data_offset: u32,
}

#[derive(Clone)]
pub struct GBFSFilesystem<'a> {
    /// Backing data slice
    data: &'a [u8],
    /// Filesystem header
    hdr: GBFSHeader,
    /// Directory
    dir_as_vec: Option<Vec<GBFSFileEntry>>,
    dir_as_array: Option<[Option<GBFSFileEntry>; NUM_CONST_FS_ENTRIES]>,
}

impl<'a> GBFSFilesystem<'a> {
    /// Constructs a new filesystem reader from a byte slice.
    pub fn from_slice(data: &'a [u8]) -> GBFSFilesystem<'a> {
        // Read the header
        let hdr = GBFSHeader::from_slice(&data[0..GBFS_HEADER_LENGTH].try_into().unwrap());

        // Read directory entries
        let mut dir_entries: Vec<GBFSFileEntry> = Vec::new();
        for i in 0..hdr.dir_num_members {
            let entry_start = hdr.dir_off as usize + ((i as usize) * DIR_ENTRY_LEN);
            // Extract filename
            let filename: [u8; FILENAME_LEN] = data[entry_start..(entry_start + FILENAME_LEN)]
                .try_into()
                .unwrap();
            let filename = Filename { backing: filename };

            // Extract length of file in bytes
            let len = u32::from_le_bytes(
                data[(entry_start + FILENAME_LEN)..(entry_start + FILENAME_LEN + 4)]
                    .try_into()
                    .unwrap(),
            );

            // Extract offset of file data from FS start
            let data_offset = u32::from_le_bytes(
                data[(entry_start + FILENAME_LEN + 4)..(entry_start + FILENAME_LEN + 8)]
                    .try_into()
                    .unwrap(),
            );

            dir_entries.push(GBFSFileEntry {
                name: filename,
                len,
                data_offset,
            });
        }

        return GBFSFilesystem {
            data,
            hdr,
            dir_as_vec: Some(dir_entries),
            dir_as_array: None,
        };
    }

    /// Gets file by index in directory table.
    fn get_file_by_index(&self, index: usize) -> File<'a> {
        // The storage format changes based on whether we have a static filesystem or
        // once created at runtime.
        if self.dir_as_vec.is_some() {
            let dir = self.dir_as_vec.as_ref().unwrap();
            return File {
                filename: dir[index].name,
                data: &self.data[dir[index].data_offset as usize
                    ..(dir[index].data_offset as usize + dir[index].len as usize)],
            };
        } else {
            let dir = self.dir_as_array.as_ref().unwrap();
            return File {
                filename: dir[index].as_ref().unwrap().name,
                data: &self.data[dir[index].as_ref().unwrap().data_offset as usize
                    ..(dir[index].as_ref().unwrap().data_offset as usize
                        + dir[index].as_ref().unwrap().len as usize)],
            };
        }
    }

    /// Gets file data by index in directory table.
    fn get_file_data_by_index(&self, index: usize) -> &'a [u8] {
        // The storage format changes based on whether we have a static filesystem or
        // once created at runtime.
        if self.dir_as_vec.is_some() {
            let dir = self.dir_as_vec.as_ref().unwrap();
            return &self.data[dir[index].data_offset as usize
                ..(dir[index].data_offset as usize + dir[index].len as usize)];
        } else {
            let dir = self.dir_as_array.as_ref().unwrap();
            return &self.data[dir[index].as_ref().unwrap().data_offset as usize
                ..(dir[index].as_ref().unwrap().data_offset as usize
                    + dir[index].as_ref().unwrap().len as usize)];
        }
    }

    // TODO: DRY the methods below

    /// Gets the file with name.
    /// `None` is returned if no file has that name.
    pub fn get_file_by_name(&self, name: FilenameString) -> Option<File> {
        // The iteration changes based on whether we have a static filesystem or
        // once created at runtime.
        if self.dir_as_vec.is_some() {
            let dir = self.dir_as_vec.as_ref().unwrap();
            for (i, entry) in dir.iter().enumerate() {
                if entry.name.as_string() == name {
                    return Some(self.get_file_by_index(i));
                }
            }
            return None;
        } else {
            // In this case, dir entries are stored in a fixed-size
            // array using an Option to denote occupied slots.
            let dir = self.dir_as_array.as_ref().unwrap();
            for (i, entry) in dir.iter().enumerate() {
                match entry {
                    Some(inner_entry) => {
                        if inner_entry.name.as_string() == name {
                            return Some(self.get_file_by_index(i));
                        }
                    }
                    None => return None,
                }
            }
            return None;
        }
    }

    /// Similar to `get_file_by_name()`, but returns a slice of the file's data instead of a `File` struct.
    /// This is useful because you don't have to keep around the `File` struct for lifetime reasons.
    pub fn get_file_data_by_name(&self, name: FilenameString) -> Option<&'a [u8]> {
        // The iteration changes based on whether we have a static filesystem or
        // once created at runtime.
        if self.dir_as_vec.is_some() {
            let dir = self.dir_as_vec.as_ref().unwrap();
            for (i, entry) in dir.iter().enumerate() {
                if entry.name.as_string() == name {
                    return Some(self.get_file_data_by_index(i));
                }
            }
            return None;
        } else {
            // In this case, dir entries are stored in a fixed-size
            // array using an Option to denote occupied slots.
            let dir = self.dir_as_array.as_ref().unwrap();
            for (i, entry) in dir.iter().enumerate() {
                match entry {
                    Some(inner_entry) => {
                        if inner_entry.name.as_string() == name {
                            return Some(self.get_file_data_by_index(i));
                        }
                    }
                    None => return None,
                }
            }
            return None;
        }
    }
}

impl<'a> IntoIterator for GBFSFilesystem<'a> {
    type Item = File<'a>;
    type IntoIter = GBFSFilesystemIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        return GBFSFilesystemIterator {
            fs: self,
            next_file_index: 0,
        };
    }
}

pub struct GBFSFilesystemIterator<'a> {
    fs: GBFSFilesystem<'a>,
    next_file_index: usize,
}

impl<'a> Iterator for GBFSFilesystemIterator<'a> {
    type Item = File<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_file_index < self.fs.hdr.dir_num_members as usize {
            let ret = Some(self.fs.get_file_by_index(self.next_file_index));
            self.next_file_index += 1;
            return ret;
        } else {
            return None;
        }
    }
}
