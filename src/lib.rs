#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;
extern crate core;

use core::convert::TryInto;
use core::{str, u16, u32};

use alloc::vec::Vec;

use arraystring::{typenum::U24, ArrayString};

const MAGIC: &str = "PinEightGBFS\r\n\u{1a}\n";
const GBFS_HEADER_LENGTH: usize = 32;
const FILENAME_LEN: usize = 24;
const DIR_ENTRY_LEN: usize = 32;

#[derive(Debug, Clone)]
struct GBFSHeader {
    total_len: u32,       /* total length of archive */
    dir_off: u16,         /* offset in bytes to directory */
    dir_num_members: u16, /* number of files */
}

impl GBFSHeader {
    fn from_slice(data: &[u8; GBFS_HEADER_LENGTH]) -> GBFSHeader {
        // Read magic
        let mut magic: [u8; 16] = [0; 16];
        magic.clone_from_slice(&data[0..16]);
        // Ensure magic is correct
        let magic_as_str = str::from_utf8(&magic).unwrap();
        assert_eq!(magic_as_str, MAGIC);
        // Read total length of archive
        let total_len = u32::from_le_bytes(data[16..20].try_into().unwrap());
        // Read offset in bytes to directory
        let dir_off = u16::from_le_bytes(data[20..22].try_into().unwrap());
        // Read number of files in dir
        let dir_num_members = u16::from_le_bytes(data[22..24].try_into().unwrap());
        // Read reserved bytes
        let reserved: [u8; 8] = data[24..32].try_into().unwrap();
        // Ensure reserved is unused; otherwise, we may be trying to interpret a newer, unknown version
        // of the format.
        assert_eq!(reserved, [0; 8]);

        return GBFSHeader {
            total_len,
            dir_off,
            dir_num_members,
        };
    }
}

#[derive(Debug, Clone)]
struct GBFSFileEntry {
    /// Name of file; at most 24 bytes
    name: Filename,
    /// Length of file in bytes
    len: u32,
    /// Offset of first file byte from start of filesystem
    data_offset: u32,
}

#[derive(Debug, Clone)]
pub struct GBFSFilesystem<'a> {
    /// Backing data slice
    data: &'a [u8],
    /// Filesystem header
    hdr: GBFSHeader,
    /// Directory
    dir: Vec<GBFSFileEntry>,
}

pub type Filename = ArrayString<U24>;
/// Represents a file extracted from GBFS.
#[derive(Debug, Clone)]
pub struct File<'a> {
    /// Name of the file. At most 24 bytes in size.
    pub filename: Filename,
    /// Data contained in the file.
    pub data: &'a [u8],
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
            let filename_full: [u8; FILENAME_LEN] = data[entry_start..(entry_start + FILENAME_LEN)]
                .try_into()
                .unwrap();
            // Filenames are padded with null bytes, remove them
            let mut filename: Vec<u8> = Vec::with_capacity(FILENAME_LEN);
            for b in filename_full.iter() {
                if *b != 0 {
                    filename.push(*b);
                } else {
                    break;
                }
            }
            let filename = Filename::try_from_utf8(filename).unwrap();

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
            dir: dir_entries,
        };
    }

    /// Gets file by index in directory table.
    fn get_file_by_index(&self, index: usize) -> File<'a> {
        return File {
            filename: self.dir[index].name,
            data: &self.data[self.dir[index].data_offset as usize
                ..(self.dir[index].data_offset as usize + self.dir[index].len as usize)],
        };
    }

    /// Gets the file with name.
    /// `None` is returned if no file has that name.
    pub fn get_file_by_name(&self, name: Filename) -> Option<File> {
        for (i, entry) in self.dir.iter().enumerate() {
            if entry.name == name {
                return Some(self.get_file_by_index(i));
            }
        }
        return None;
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
        if self.next_file_index <= self.fs.hdr.dir_num_members as usize {
            return Some(self.fs.get_file_by_index(self.next_file_index));
        } else {
            return None;
        }
    }
}
