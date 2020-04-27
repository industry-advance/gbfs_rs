//! This module declares a special kind of filesystem, which is
//! constant and where all the work has been done at compile-time.
//! At runtime, all you have to do is get references to files.

use crate::{Filename, GBFSFileEntry, GBFSFilesystem, GBFSHeader, DIR_ENTRY_LEN, FILENAME_LEN};
// TODO: Expose to lib users so that it can better be tweaked for low-ROM environments
pub(crate) const NUM_CONST_FS_ENTRIES: usize = 2048;

/// Creates a fully `const` and `static`-friendly filesystem.
/// Perfect for embedding assets in ROM without having to worry about
/// runtime overhead in terms of CPU or RAM usage, as well as lifetimes.
/// All the files returned have the `'static` lifetime.
pub const fn const_fs(data: &'static [u8]) -> GBFSFilesystem<'static> {
    // Brace yourself for some very ugly code caused by the limitations of const fn below.
    // Create the FS header
    // Forgive me God, for I have sinned
    // TODO: Clean up this mess (maybe a macro?)
    let hdr = GBFSHeader::from_slice(&[
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8], data[9],
        data[10], data[11], data[12], data[13], data[14], data[15], data[16], data[17], data[18],
        data[19], data[20], data[21], data[22], data[23], data[24], data[25], data[26], data[27],
        data[28], data[29], data[30], data[31],
    ]);
    // Create the FS entry table
    // Read directory entries
    let mut dir_entries: [Option<GBFSFileEntry>; NUM_CONST_FS_ENTRIES] =
        [None; NUM_CONST_FS_ENTRIES];
    // Can't use a for loop here because they're not yet supported in const fn's
    let mut i = 0;
    while i < hdr.dir_num_members as usize {
        let entry_start = hdr.dir_off as usize + ((i as usize) * DIR_ENTRY_LEN);
        // Extract filename
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

        let filename = Filename { backing: filename };

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
        dir_as_vec: None,
        dir_as_array: Some(dir_entries),
    };
}
