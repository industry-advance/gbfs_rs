#![no_std]
#![forbid(unsafe_code)]
#![feature(const_if_match)]
#![feature(const_loop)]
#![feature(const_fn)]
#![feature(const_panic)]
#![feature(const_mut_refs)]
#![feature(const_in_array_repeat_expressions)]

mod header;
use header::*;

use core::u32;

use byte_slice_cast::AsSliceOf;
use arrayvec::ArrayVec;
use arraystring::{typenum::U24, ArrayString};

const FILENAME_LEN: usize = 24;
const DIR_ENTRY_LEN: usize = 32;
// TODO: Allow control at build-time by user for different ROM use/flexibility tradeoffs.
const NUM_FS_ENTRIES: usize = 2048;

pub type Filename = ArrayString<U24>;

#[derive(Debug, Clone)]
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
    fn name_is_equal(&self, name: Filename) -> bool {
        // Unfortunately, the const fn constructor for GBFSFilesystem
        // can't use dynamically-sized data structures.
        // Therefore, we have to strip out the trailing nulls from the filename here.
        let no_nulls: ArrayVec<[u8; crate::FILENAME_LEN]> = self.name
            .iter()
            .filter(|x| **x != 0)
            .map(|x| *x)
            .collect();
        let our_name = Filename::try_from_utf8(&no_nulls.as_ref()).expect("Encountered file in FS with name that's not UTF-8");
        return name == our_name;
    }
}

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
    /// Constructs a new filesystem reader from a byte slice.

    /// Creates a fully `const` and `static`-friendly filesystem.
    /// Perfect for embedding assets in ROM without having to worry about
    /// runtime overhead in terms of CPU or RAM usage, as well as lifetimes.
    /// All the files returned have the `'static` lifetime.
    pub const fn from_slice(data: &'a [u8]) -> GBFSFilesystem<'a> {
        // Brace yourself for some very ugly code caused by the limitations of const fn below.
        // Create the FS header
        // Forgive me God, for I have sinned
        // TODO: Clean up this mess (maybe a macro?)
        let hdr = GBFSHeader::from_slice(&[
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            data[9], data[10], data[11], data[12], data[13], data[14], data[15], data[16],
            data[17], data[18], data[19], data[20], data[21], data[22], data[23], data[24],
            data[25], data[26], data[27], data[28], data[29], data[30], data[31],
        ]);
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
        return GBFSFilesystem {
            data,
            hdr,
            dir: dir_entries,
        };
    }

    /// Gets file data by index in directory table.
    fn get_file_data_by_index(&self, index: usize) -> &'a [u8] {
        // The storage format changes based on whether we have a static filesystem or
        // once created at runtime.
        let dir_entry_wrapped = self.dir[index].clone();
        let dir_entry = dir_entry_wrapped
            .expect("Attempt to access file with nonexistent index")
            .clone();
        return &self.data[dir_entry.data_offset as usize
            ..(dir_entry.data_offset as usize + dir_entry.len as usize)];
    }

    // TODO: DRY the methods below

    /// Similar to `get_file_by_name()`, but returns a slice of the file's data instead of a `File` struct.
    /// This is useful because you don't have to keep around the `File` struct for lifetime reasons.
    pub fn get_file_data_by_name(&self, name: Filename) -> Option<&'a [u8]> {
        // In this case, dir entries are stored in a fixed-size
        // array using an Option to denote occupied slots.
        for (i, entry) in self.dir.iter().enumerate() {
            match entry {
                Some(inner_entry) => {
                    if inner_entry.name_is_equal(name) {
                        return Some(self.get_file_data_by_index(i));
                    }
                }
                None => return None,
            }
        }
        return None;
    }

    /// Returns a reference to the file data as a slice of u32's.
    /// Note that this function currently panics if the length of data is not divisible by 2.
    pub fn get_file_data_by_name_as_u16_slice(&self, name: Filename) -> Option<&'a [u16]> {
        // TODO: Proper error handling
        if (self.data.len() % 2) != 0 {
            panic!("Attempt to obtain u16 from file with odd number of bytes");
        }
        for (i, entry) in self.dir.iter().enumerate() {
            match entry {
                Some(inner_entry) => {
                    if inner_entry.name_is_equal(name) {
                        return Some(self.get_file_data_by_index(i).as_slice_of::<u16>().unwrap());
                    }
                }
                None => return None,
            }
        }
        return None;
    }

    /// Returns a reference to the file data as a slice of u32's.
    /// Note that this function currently panics if the length of data is not divisible by 2.
    pub fn get_file_data_by_name_as_u32_slice(&self, name: Filename) -> Option<&'a [u32]> {
        // TODO: Proper error handling
        if (self.data.len() % 4) != 0 {
            panic!("Attempt to obtain u32 from file with number of bytes not divisible by 4");
        }
        for (i, entry) in self.dir.iter().enumerate() {
            match entry {
                Some(inner_entry) => {
                    if inner_entry.name_is_equal(name) {
                        return Some(self.get_file_data_by_index(i).as_slice_of::<u32>().unwrap());
                    }
                }
                None => return None,
            }
        }
        return None;
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
