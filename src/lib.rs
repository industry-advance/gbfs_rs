#![no_std]
#![forbid(unsafe_code)]
#![feature(const_fn)]
#![feature(const_panic)]
#![feature(const_mut_refs)]
#![feature(const_in_array_repeat_expressions)]

//! This crate enables reading of Gameboy Filesystem (`GBFS`)-formatted data.
//! It's primarily designed for use in GBA games, and as such is fully `no_std` compatible (even `alloc` is not required).

mod header;
use header::*;

use core::fmt;
use core::u32;

use arraystring::{typenum::U24, ArrayString};
use arrayvec::ArrayVec;
use byte_slice_cast::AsSliceOf;

/// Maximum length of a filename in bytes. Is 24 in the pin-eight C implementation
const FILENAME_LEN: usize = 24;
const DIR_ENTRY_LEN: usize = 32;
// TODO: Allow control at build-time by user for different ROM use/flexibility tradeoffs.
const NUM_FS_ENTRIES: usize = 2048;

/// Top-level error type for this crate.
#[derive(Debug, Clone)]
pub enum GBFSError {
    /// Returned when an invalid filename is encountered in the GBFS archive.
    InvalidFilenameError(arraystring::Error),
    /// Returned when trying to get a slice of u16/u32 from a file which size is not a multiple of 2/4 bytes.
    FileLengthNotMultipleOf { multiple: usize, length: usize },
    /// Returned when a file with the given name does not exist.
    NoSuchFile(Filename),
    /// Returned when trying to open a GBFS archive which starts with incorrect magic bytes.
    WrongMagic,
}

impl fmt::Display for GBFSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use GBFSError::*;
        match self {
            InvalidFilenameError(err) => write!(f, "Encountered file with invalid name: {}", err),
            FileLengthNotMultipleOf {multiple, length} => write!(f, "Attempt to access file as slice of values with length of {} bytes, but file is {} bytes long and length is not multiple of {} bytes", multiple, length, multiple),
            NoSuchFile(name) => write!(f, "File \"{}\" does not exist in filesystem", name),
            WrongMagic => write!(f, "GBFS archive has incorrect magic bytes"),
        }
    }
}

/// The name of a GBFS file. This is not a regular string because filenames have a limited length.
pub type Filename = ArrayString<U24>;

#[derive(Debug, Clone)]
// Needed to ensure proper alignment for casting u8 slices to u16/u32 slices
#[repr(align(4))]
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
        let no_nulls: ArrayVec<[u8; crate::FILENAME_LEN]> =
            self.name.iter().filter(|x| **x != 0).map(|x| *x).collect();
        match Filename::try_from_utf8(&no_nulls.as_ref()) {
            Err(e) => return Err(GBFSError::InvalidFilenameError(e)),
            Ok(our_name) => return Ok(name == our_name),
        }
    }
}

/// A filesystem that files can be read from.
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
        // Brace yourself for some very ugly code caused by the limitations of const fn below.
        // Create the FS header
        // Forgive me God, for I have sinned
        // TODO: Clean up this mess (maybe a macro?)
        let hdr: GBFSHeader;

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
        while i < hdr.dir_num_members as usize {
            let entry_start = hdr.dir_off as usize + ((i as usize) * DIR_ENTRY_LEN);
            // Extract filename
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
            let len = u32::from_le_bytes([
                data[(entry_start + FILENAME_LEN)],
                data[entry_start + FILENAME_LEN + 1],
                data[entry_start + FILENAME_LEN + 2],
                data[entry_start + FILENAME_LEN + 3],
            ]);

            // Extract offset of file data from FS start
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
        let dir_entry_wrapped = self.dir[index].clone();
        let dir_entry = dir_entry_wrapped
            // This should never trigger.
            .expect("Attempt to access file with nonexistent index. This is a bug in gbfs_rs.")
            .clone();
        return &self.data[dir_entry.data_offset as usize
            ..(dir_entry.data_offset as usize + dir_entry.len as usize)];
    }

    /// Returns a reference to the file data as a slice of u8's.
    /// An error is returned if the file does not exist.
    pub fn get_file_data_by_name(&self, name: Filename) -> Result<&'a [u8], GBFSError> {
        // In this case, dir entries are stored in a fixed-size
        // array using an Option to denote occupied slots.
        for (i, entry) in self.dir.iter().enumerate() {
            match entry {
                Some(inner_entry) => {
                    if inner_entry.name_is_equal(name)? {
                        return Ok(self.get_file_data_by_index(i));
                    }
                }
                None => return Err(GBFSError::NoSuchFile(name.clone())),
            }
        }
        return Err(GBFSError::NoSuchFile(name.clone()));
    }

    /// Returns a reference to the file data as a slice of u32's.
    /// An error is returned if the file does not exist or it's length is not a multiple of 2.
    pub fn get_file_data_by_name_as_u16_slice(
        &self,
        name: Filename,
    ) -> Result<&'a [u16], GBFSError> {
        if (self.data.len() % 2) != 0 {
            return Err(GBFSError::FileLengthNotMultipleOf {
                multiple: 2,
                length: self.data.len(),
            });
        }
        return Ok(self
            .get_file_data_by_name(name)?
            .as_slice_of::<u16>()
            .unwrap());
    }

    /// Returns a reference to the file data as a slice of u32's.
    /// An error is returned if the file does not exist or it's length is not a multiple of 4.
    pub fn get_file_data_by_name_as_u32_slice(
        &self,
        name: Filename,
    ) -> Result<&'a [u32], GBFSError> {
        if (self.data.len() % 4) != 0 {
            return Err(GBFSError::FileLengthNotMultipleOf {
                multiple: 4,
                length: self.data.len(),
            });
        }
        return Ok(self
            .get_file_data_by_name(name)?
            .as_slice_of::<u32>()
            .unwrap());
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
