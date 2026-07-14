//! WAD directory parsing for the `w_wad` migration.
//!
//! This intentionally mirrors Chocolate Doom's C loader semantics for WAD
//! headers, single-lump files, lump names, and directory metadata.

use std::ffi::CStr;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::raw::{c_char, c_int};
use std::path::{Path, MAIN_SEPARATOR};

const WAD_HEADER_LEN: usize = 12;
const WAD_DIRECTORY_ENTRY_LEN: usize = 16;
const MAX_VANILLA_PWAD_LUMPS: i32 = 4046;
const DIRECTORY_CHECKSUM_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const DIRECTORY_CHECKSUM_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LumpMetadata {
    pub name: [u8; 8],
    pub position: i32,
    pub size: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WadSummary {
    pub lump_count: u32,
    pub total_size: u32,
    pub directory_checksum: u64,
    pub first_name: [u8; 8],
    pub last_name: [u8; 8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum WadError {
    NullPointer = 1,
    InvalidPath = 2,
    Io = 3,
    InvalidWadId = 4,
    InvalidDirectory = 5,
    TooManyPwadLumps = 6,
}

impl WadError {
    fn code(self) -> c_int {
        self as c_int
    }
}

pub fn summarize_file(path: &Path) -> Result<WadSummary, WadError> {
    let lumps = load_lumps(path)?;
    Ok(summarize_lumps(&lumps))
}

fn load_lumps(path: &Path) -> Result<Vec<LumpMetadata>, WadError> {
    if has_wad_extension(path) {
        load_wad_lumps(path)
    } else {
        load_single_lump(path)
    }
}

fn load_wad_lumps(path: &Path) -> Result<Vec<LumpMetadata>, WadError> {
    let mut file = File::open(path).map_err(|_| WadError::Io)?;
    let mut header = [0u8; WAD_HEADER_LEN];
    file.read_exact(&mut header)
        .map_err(|_| WadError::InvalidDirectory)?;

    let identification = &header[0..4];
    if identification != b"IWAD" && identification != b"PWAD" {
        return Err(WadError::InvalidWadId);
    }

    let num_lumps = read_i32_le(&header[4..8]);
    if identification == b"PWAD" && num_lumps > MAX_VANILLA_PWAD_LUMPS {
        return Err(WadError::TooManyPwadLumps);
    }
    if num_lumps < 0 {
        return Err(WadError::InvalidDirectory);
    }

    let directory_offset = read_i32_le(&header[8..12]);
    if directory_offset < 0 {
        return Err(WadError::InvalidDirectory);
    }

    let num_lumps_usize = num_lumps as usize;
    let directory_len = num_lumps_usize
        .checked_mul(WAD_DIRECTORY_ENTRY_LEN)
        .ok_or(WadError::InvalidDirectory)?;
    let mut directory = vec![0u8; directory_len];
    file.seek(SeekFrom::Start(directory_offset as u64))
        .map_err(|_| WadError::InvalidDirectory)?;
    file.read_exact(&mut directory)
        .map_err(|_| WadError::InvalidDirectory)?;

    let mut lumps = Vec::with_capacity(num_lumps_usize);
    for entry in directory.chunks_exact(WAD_DIRECTORY_ENTRY_LEN) {
        let mut name = [0u8; 8];
        name.copy_from_slice(&entry[8..16]);
        lumps.push(LumpMetadata {
            position: read_i32_le(&entry[0..4]),
            size: read_i32_le(&entry[4..8]),
            name,
        });
    }

    Ok(lumps)
}

fn load_single_lump(path: &Path) -> Result<Vec<LumpMetadata>, WadError> {
    let size = File::open(path)
        .and_then(|file| file.metadata())
        .map_err(|_| WadError::Io)?
        .len();
    let size = i32::try_from(size).map_err(|_| WadError::InvalidDirectory)?;

    Ok(vec![LumpMetadata {
        name: extract_file_base(path),
        position: 0,
        size,
    }])
}

fn summarize_lumps(lumps: &[LumpMetadata]) -> WadSummary {
    let mut checksum = DIRECTORY_CHECKSUM_OFFSET;
    checksum_u32(&mut checksum, lumps.len() as u32);

    let mut total_size = 0u32;
    for lump in lumps {
        checksum_bytes(&mut checksum, &lump.name);
        checksum_i32(&mut checksum, lump.position);
        checksum_i32(&mut checksum, lump.size);
        total_size = total_size.wrapping_add(lump.size as u32);
    }

    WadSummary {
        lump_count: lumps.len() as u32,
        total_size,
        directory_checksum: checksum,
        first_name: lumps.first().map_or([0; 8], |lump| lump.name),
        last_name: lumps.last().map_or([0; 8], |lump| lump.name),
    }
}

fn has_wad_extension(path: &Path) -> bool {
    let Some(path) = path.to_str() else {
        return false;
    };
    path.len() >= 3 && path[path.len() - 3..].eq_ignore_ascii_case("wad")
}

fn extract_file_base(path: &Path) -> [u8; 8] {
    let path = path.to_string_lossy();
    let mut start = 0;
    for (index, ch) in path.char_indices() {
        if ch == MAIN_SEPARATOR {
            start = index + ch.len_utf8();
        }
    }

    let mut name = [0u8; 8];
    for (index, byte) in path[start..]
        .bytes()
        .take_while(|byte| *byte != b'.')
        .take(8)
        .enumerate()
    {
        name[index] = byte.to_ascii_uppercase();
    }
    name
}

fn read_i32_le(bytes: &[u8]) -> i32 {
    i32::from_le_bytes(bytes.try_into().expect("slice length checked by caller"))
}

fn checksum_bytes(checksum: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *checksum ^= u64::from(*byte);
        *checksum = checksum.wrapping_mul(DIRECTORY_CHECKSUM_PRIME);
    }
}

fn checksum_u32(checksum: &mut u64, value: u32) {
    checksum_bytes(checksum, &value.to_be_bytes());
}

fn checksum_i32(checksum: &mut u64, value: i32) {
    checksum_u32(checksum, value as u32);
}

fn copy_lump_name(dest: *mut c_char, dest_len: usize, name: &[u8; 8]) {
    if dest.is_null() || dest_len == 0 {
        return;
    }

    let copy_len = name
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(name.len())
        .min(dest_len - 1);

    // SAFETY: The caller supplied a writable buffer of `dest_len` bytes. We
    // copy at most `dest_len - 1` bytes and always append a NUL terminator.
    unsafe {
        for index in 0..copy_len {
            *dest.add(index) = name[index] as c_char;
        }
        *dest.add(copy_len) = 0;
    }
}

/// C ABI facade for parsing a WAD/single-lump file and returning metadata.
///
/// Returns 0 on success or a stable `WadError` status code on failure.
///
/// # Safety
///
/// `path` must point to a NUL-terminated string. Output pointers may be NULL
/// when the caller does not need the corresponding value. Name buffers must
/// have the supplied lengths.
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn ffi_file_summary(
    path: *const c_char,
    lump_count: *mut u32,
    total_size: *mut u32,
    directory_checksum: *mut u64,
    first_name: *mut c_char,
    first_name_len: usize,
    last_name: *mut c_char,
    last_name_len: usize,
) -> c_int {
    if path.is_null() {
        return WadError::NullPointer.code();
    }

    // SAFETY: The public ABI contract requires `path` to be NUL-terminated.
    let path = unsafe { CStr::from_ptr(path) };
    let Ok(path) = path.to_str() else {
        return WadError::InvalidPath.code();
    };

    match summarize_file(Path::new(path)) {
        Ok(summary) => {
            // SAFETY: Each output pointer is checked for NULL before writing.
            unsafe {
                if !lump_count.is_null() {
                    *lump_count = summary.lump_count;
                }
                if !total_size.is_null() {
                    *total_size = summary.total_size;
                }
                if !directory_checksum.is_null() {
                    *directory_checksum = summary.directory_checksum;
                }
            }
            copy_lump_name(first_name, first_name_len, &summary.first_name);
            copy_lump_name(last_name, last_name_len, &summary.last_name);
            0
        }
        Err(error) => error.code(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_base_matches_c_truncation_semantics() {
        assert_eq!(
            extract_file_base(Path::new("abc/longfilename.lmp")),
            *b"LONGFILE"
        );
        assert_eq!(extract_file_base(Path::new("abc/e1m1")), *b"E1M1\0\0\0\0");
    }
}
