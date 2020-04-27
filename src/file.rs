use alloc::vec::Vec;
use arrayvec::ArrayVec;
use core::convert::TryInto;

use arraystring::{typenum::U24, ArrayString};

pub type FilenameString = ArrayString<U24>;
/// Name of the file.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Filename {
    pub(crate) backing: [u8; crate::FILENAME_LEN],
}

impl Filename {
    pub fn as_string(&self) -> FilenameString {
        // Unfortunately, the const fn constructor for GBFSFilesystem
        // can't use dynamically-sized data structures.
        // Therefore, we have to strip out the trailing nulls from the filename here.
        let no_nulls: ArrayVec<[u8; crate::FILENAME_LEN]> = self
            .backing
            .iter()
            .filter(|x| **x != 0)
            .map(|x| *x)
            .collect();
        // Guaranteed to always be valid unicode
        return FilenameString::try_from_utf8(&no_nulls.as_ref()).unwrap();
    }
}
/// Represents a file extracted from GBFS.
#[derive(Debug, Clone)]
pub struct File<'a> {
    /// Name of the file. At mo   array_str(ArrayString<U24>),st 24 bytes in size.
    pub filename: Filename,
    /// Data contained in the file.
    pub data: &'a [u8],
}

impl<'a> File<'a> {
    /// Converts the file data to a vector of u16's.
    /// Note that this function currently panics if the length of data is not divisible by 2.
    pub fn to_u16_vec(&self) -> Vec<u16> {
        // TODO: Proper error handling
        // TODO: Figure out whether returning data on the stack is possible here
        if (self.data.len() % 2) != 0 {
            panic!("Attempt to obtain u16 from file with odd number of bytes");
        }
        let mut u16_vec: Vec<u16> = Vec::with_capacity(self.data.len() / 2);
        self.data
            .chunks(2)
            .for_each(|x| u16_vec.push(u16::from_be_bytes(x.try_into().unwrap())));
        return u16_vec;
    }

    /// Converts the file data to a vector of u32's.
    /// Note that this function currently panics if the length of data is not divisible by 2.
    pub fn to_u32_vec(&self) -> Vec<u32> {
        // TODO: Proper error handling
        // TODO: Figure out whether returning data on the stack is possible here
        if (self.data.len() % 4) != 0 {
            panic!("Attempt to obtain u32 from file with number of bytes not divisible by 4");
        }
        let mut u32_vec: Vec<u32> = Vec::with_capacity(self.data.len() / 4);
        self.data
            .chunks(4)
            .for_each(|x| u32_vec.push(u32::from_be_bytes(x.try_into().unwrap())));
        return u32_vec;
    }
}
