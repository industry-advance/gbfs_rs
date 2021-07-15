const MAGIC: &[u8] = "PinEightGBFS\r\n\u{1a}\n".as_bytes();
pub(crate) const GBFS_HEADER_LENGTH: usize = 32;

use crate::GBFSError;

#[derive(Debug, Clone)]
pub(crate) struct GBFSHeader {
    pub(crate) total_len: u32,       /* total length of archive */
    pub(crate) dir_off: u16,         /* offset in bytes to directory */
    pub(crate) dir_num_members: u16, /* number of files */
}

impl GBFSHeader {
    pub(crate) const fn from_slice(
        data: &[u8; GBFS_HEADER_LENGTH],
    ) -> Result<GBFSHeader, GBFSError> {
        // I apologize for the ugly code below. It's needed due to const fn limitations.

        // Ensure magic is correct by checking char-for-char
        let mut i = 0;
        while i < 16 {
            if data[i] != MAGIC[i] {
                return Err(GBFSError::HeaderInvalid);
            }
            i += 1;
        }

        // Read total length of archive
        let total_len = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        // Read offset in bytes to directory
        let dir_off = u16::from_le_bytes([data[20], data[21]]);
        // Read number of files in dir
        let dir_num_members = u16::from_le_bytes([data[22], data[23]]);

        // Ensure reserved bytes are unused; otherwise, we may be trying to interpret a newer, unknown version
        // of the format.
        // We can't use assert_eq! here because it's not permitted in const fn's.
        let mut i = 24;
        // Are reserved bytes 0 as expected?
        while i < 32 {
            if data[i] != 0 {
                return Err(GBFSError::HeaderInvalid);
            }
            i += 1;
        }
        return Ok(GBFSHeader {
            total_len,
            dir_off,
            dir_num_members,
        });
    }
}
