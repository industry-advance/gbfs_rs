use crate::Filename;
use core::{convert, fmt};

/// Various error conditions that can occur when working with GBFS archives.
#[derive(Debug, Clone, PartialEq)]
pub enum GBFSError {
    /// Returned when an invalid filename is encountered in the GBFS archive.
    ArchiveInvalidFilename(arraystring::Error),
    /// Returned when an invalid filename is supplied by the calling code.
    UserInvalidFilename(arraystring::error::OutOfBounds),
    /// Returned when casting to the requested slice type fails.
    Cast(byte_slice_cast::Error),
    /// Returned when a file with the given name does not exist.
    NoSuchFile(Filename),
    /// Returned when trying to open a GBFS archive which starts with incorrect magic bytes.
    WrongMagic,
}

impl fmt::Display for GBFSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use GBFSError::*;
        match self {
            ArchiveInvalidFilename(err) => write!(f, "Encountered file with invalid name: {}", err),
            UserInvalidFilename(err) => write!(f, "Was given invalid filename: {}", err),
            Cast(err) => write!(f, "Failed to cast from u8 slice: {}", err),
            NoSuchFile(name) => write!(f, "File \"{}\" does not exist in filesystem", name),
            WrongMagic => write!(f, "GBFS archive has incorrect magic bytes"),
        }
    }
}

impl convert::From<byte_slice_cast::Error> for GBFSError {
    fn from(error: byte_slice_cast::Error) -> Self {
        GBFSError::Cast(error)
    }
}
