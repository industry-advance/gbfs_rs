use crate::Filename;
use core::{convert, fmt, str};

/// Various error conditions that can occur when working with GBFS archives.
#[derive(Debug, Clone, PartialEq)]
pub enum GBFSError {
    /// Returned when the archive's header is not valid.
    HeaderInvalid,
    /// Returned when a file with a name that's too long is encountered in the archive or provided by the user.
    FilenameTooLong(usize, usize),
    /// Returned when a filename with invalid UTF-8 is encountered in the archive.
    Utf8Error(str::Utf8Error),
    /// Returned when casting to the requested slice type fails.
    Cast(byte_slice_cast::Error),
    /// Returned when a file with the given name does not exist.
    NoSuchFile(Filename),
    /// Returned when a file is truncated.
    Truncated,
    /// Returned when an archive contains too many entries.
    TooManyEntries(usize, usize),
}

impl fmt::Display for GBFSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use GBFSError::*;
        match self {
            HeaderInvalid => write!(f, "Encountered invalid header"),
            FilenameTooLong(expected, actual) => write!(
                f,
                "Encountered filename of invalid length: at most {} bytes are supported, but got {}",
                expected, actual
            ),
            GBFSError::Utf8Error(err) => write!(f, "Encountered filename that's not valid UTF-8 starting at position {} in filename", err.valid_up_to()),
            Cast(err) => write!(f, "Failed to cast from u8 slice: {}", err),
            NoSuchFile(name) => write!(f, "File \"{}\" does not exist in filesystem", name),
            Truncated => write!(f, "Encountered truncated file entry"),
            TooManyEntries(expected, actual) => write!(
                f,
                "Encountered archive with too many entries: at most {} entries are supported, but got {}",
                expected, actual
            ),
        }
    }
}

impl convert::From<byte_slice_cast::Error> for GBFSError {
    fn from(error: byte_slice_cast::Error) -> Self {
        GBFSError::Cast(error)
    }
}
