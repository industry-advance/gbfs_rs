#![no_std]
#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]

//! This crate enables reading of Gameboy Filesystem (`GBFS`)-formatted data.
//! It's primarily designed for use in GBA games, and as such is fully `no_std` compatible (even `alloc` is not required).

mod error;
pub use error::*;
mod header;
use header::*;

use core::str;
use core::u32;

use arrayvec::{ArrayString, ArrayVec};
use byte_slice_cast::AsSliceOf;

/// Maximum length of a filename in bytes. Is 24 in the pin-eight C implementation
pub const FILENAME_LEN: usize = 24;
/// Length of a single file's entry in the directory that precedes the data.
const DIR_ENTRY_LEN: usize = 32;
// TODO: Allow control at build-time by user for different ROM use/flexibility tradeoffs.
const NUM_FS_ENTRIES: usize = 2048;

/// The name of a GBFS file. This is not a regular string because filenames have a limited length.
type Filename = ArrayString<FILENAME_LEN>;

#[derive(Debug, Copy, Clone)]
struct GBFSFileEntry {
    /// Name of file; at most 24 bytes.
    /// TODO: Once const fn's can perform subslicing, use a slice here
    name: [u8; FILENAME_LEN],
    /// Length of file in bytes
    len: u32,
    /// Offset of first file byte from start of filesystem
    data_offset: u32,
}

impl GBFSFileEntry {
    /// Compare the name with a Filename.
    fn name_is_equal(&self, name: Filename) -> Result<bool, GBFSError> {
        // Unfortunately, the const fn constructor for GBFSFilesystem
        // can't use dynamically-sized data structures.
        // Therefore, we have to strip out the trailing nulls from the filename here.
        let no_nulls: ArrayVec<u8, { FILENAME_LEN }> =
            self.name.iter().filter(|x| **x != 0).copied().collect();
        let filename_str: &str = match str::from_utf8(no_nulls.as_ref()) {
            Ok(s) => s,
            Err(e) => return Err(GBFSError::Utf8Error(e)),
        };
        match Filename::from(filename_str) {
            Err(_) => return Err(GBFSError::FilenameTooLong(FILENAME_LEN, filename_str.len())),
            Ok(our_name) => return Ok(name == our_name),
        }
    }
}

/// A filesystem that files can be read from.
// Needed to ensure proper alignment for casting u8 slices to u16/u32 slices
#[repr(align(4))]
#[repr(C)]
#[derive(Clone)]
pub struct GBFSFilesystem<'a> {
    /// Backing data slice
    data: &'a [u8],
    /// Filesystem header
    hdr: GBFSHeader,
    /// Directory
    dir: [Option<GBFSFileEntry>; NUM_FS_ENTRIES],
}

impl<'a> GBFSFilesystem<'a> {
    /// Constructs a new filesystem from a GBFS-formatted byte slice.
    ///
    /// To make lifetime management easier it's probably a good idea to use a slice with a `static` lifetime here.
    /// It's also a good idea to ensure this function is called at compile time with a `const` argument,
    /// to avoid having to store the filesystem index in RAM.
    pub const fn from_slice(data: &'a [u8]) -> Result<GBFSFilesystem<'a>, GBFSError> {
        // TODO: Assert slice alignment
        // Brace yourself for some very ugly code caused by the limitations of const fn below.
        // Create the FS header
        // Forgive me God, for I have sinned
        // TODO: Clean up this mess (maybe a macro?)
        let hdr: GBFSHeader;

        if data.len() < header::GBFS_HEADER_LENGTH {
            return Err(GBFSError::HeaderInvalid);
        }
        match GBFSHeader::from_slice(&[
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            data[9], data[10], data[11], data[12], data[13], data[14], data[15], data[16],
            data[17], data[18], data[19], data[20], data[21], data[22], data[23], data[24],
            data[25], data[26], data[27], data[28], data[29], data[30], data[31],
        ]) {
            Ok(val) => hdr = val,
            Err(err) => return Err(err),
        }
        // Create the FS entry table
        // Read directory entries
        let mut dir_entries: [Option<GBFSFileEntry>; NUM_FS_ENTRIES] = [None; NUM_FS_ENTRIES];
        // Can't use a for loop here because they're not yet supported in const fn's
        let mut i = 0;
        if (hdr.dir_num_members as usize) > NUM_FS_ENTRIES {
            return Err(GBFSError::TooManyEntries(
                NUM_FS_ENTRIES,
                hdr.dir_num_members as usize,
            ));
        }
        while i < hdr.dir_num_members as usize {
            let entry_start = hdr.dir_off as usize + ((i as usize) * DIR_ENTRY_LEN);
            // Extract filename
            if data.len() < entry_start + FILENAME_LEN {
                return Err(GBFSError::Truncated);
            }
            // TODO: DRY
            let filename: [u8; FILENAME_LEN] = [
                data[entry_start],
                data[entry_start + 1],
                data[entry_start + 2],
                data[entry_start + 3],
                data[entry_start + 4],
                data[entry_start + 5],
                data[entry_start + 6],
                data[entry_start + 7],
                data[entry_start + 8],
                data[entry_start + 9],
                data[entry_start + 10],
                data[entry_start + 11],
                data[entry_start + 12],
                data[entry_start + 13],
                data[entry_start + 14],
                data[entry_start + 15],
                data[entry_start + 16],
                data[entry_start + 17],
                data[entry_start + 18],
                data[entry_start + 19],
                data[entry_start + 20],
                data[entry_start + 21],
                data[entry_start + 22],
                data[entry_start + 23],
            ];

            // Extract length of file in bytes
            if data.len() < entry_start + FILENAME_LEN + 4 {
                return Err(GBFSError::Truncated);
            };
            let len = u32::from_le_bytes([
                data[(entry_start + FILENAME_LEN)],
                data[entry_start + FILENAME_LEN + 1],
                data[entry_start + FILENAME_LEN + 2],
                data[entry_start + FILENAME_LEN + 3],
            ]);

            // Extract offset of file data from FS start
            if data.len() < entry_start + FILENAME_LEN + 8 {
                return Err(GBFSError::Truncated);
            };
            let data_offset = u32::from_le_bytes([
                data[(entry_start + FILENAME_LEN + 4)],
                data[entry_start + FILENAME_LEN + 5],
                data[entry_start + FILENAME_LEN + 6],
                data[entry_start + FILENAME_LEN + 7],
            ]);

            dir_entries[i] = Some(GBFSFileEntry {
                name: filename,
                len,
                data_offset,
            });
            i += 1;
        }
        return Ok(GBFSFilesystem {
            data,
            hdr,
            dir: dir_entries,
        });
    }

    /// Gets file data by index in directory table.
    fn get_file_data_by_index(&self, index: usize) -> &'a [u8] {
        // The storage format changes based on whether we have a static filesystem or
        // once created at runtime.
        let dir_entry_wrapped = self.dir[index];
        let dir_entry = dir_entry_wrapped
            // This should never trigger.
            .expect("Attempt to access file with nonexistent index. This is a bug in gbfs_rs.");
        return &self.data[dir_entry.data_offset as usize
            ..(dir_entry.data_offset as usize + dir_entry.len as usize)];
    }

    /// Returns a reference to the file data as a slice of u8's.
    /// An error is returned if the file does not exist or the filename is invalid.
    /// All filenames longer than `FILENAME_LEN` characters are invalid.
    pub fn get_file_data_by_name(&self, str_name: &str) -> Result<&'a [u8], GBFSError> {
        let name: Filename;
        match Filename::from(str_name) {
            Ok(val) => name = val,
            Err(_) => return Err(GBFSError::FilenameTooLong(FILENAME_LEN, str_name.len())),
        }

        // In this case, dir entries are stored in a fixed-size
        // array using an Option to denote occupied slots.
        for (i, entry) in self.dir.iter().enumerate() {
            match entry {
                Some(inner_entry) => {
                    if inner_entry.name_is_equal(name)? {
                        return Ok(self.get_file_data_by_index(i));
                    }
                }
                None => return Err(GBFSError::NoSuchFile(name)),
            }
        }
        return Err(GBFSError::NoSuchFile(name));
    }

    /// Returns a reference to the file data as a slice of u32's.
    /// An error is returned if the file does not exist, it's length is not a multiple of 2
    /// or the filename is invalid.
    /// All filenames longer than 24 characters are invalid.
    pub fn get_file_data_by_name_as_u16_slice(&self, name: &str) -> Result<&'a [u16], GBFSError> {
        return Ok(self.get_file_data_by_name(name)?.as_slice_of::<u16>()?);
    }

    /// Returns a reference to the file data as a slice of u32's.
    /// An error is returned if the file does not exist, it's length is not a multiple of 4
    /// or the filename is invalid.
    /// All filenames longer than 24 characters are invalid.
    pub fn get_file_data_by_name_as_u32_slice(&self, name: &str) -> Result<&'a [u32], GBFSError> {
        return Ok(self.get_file_data_by_name(name)?.as_slice_of::<u32>()?);
    }
}

impl<'a> IntoIterator for GBFSFilesystem<'a> {
    type Item = &'a [u8];
    type IntoIter = GBFSFilesystemIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        return GBFSFilesystemIterator {
            fs: self,
            next_file_index: 0,
        };
    }
}

/// Returns the data of each file in the filesystem.
pub struct GBFSFilesystemIterator<'a> {
    fs: GBFSFilesystem<'a>,
    next_file_index: usize,
}

impl<'a> Iterator for GBFSFilesystemIterator<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_file_index < self.fs.hdr.dir_num_members as usize {
            let ret = Some(self.fs.get_file_data_by_index(self.next_file_index));
            self.next_file_index += 1;
            return ret;
        } else {
            return None;
        }
    }
}
