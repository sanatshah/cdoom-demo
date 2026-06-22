//! WAD binary format parsing shared with the C engine.

use std::os::raw::c_char;
use std::slice;

const HEADER_SIZE: usize = 12;
const DIRECTORY_ENTRY_SIZE: usize = 16;
const MAX_NAME_LEN: usize = 8;

pub const PARSE_OK: i32 = 0;
pub const PARSE_INVALID_ARGUMENT: i32 = -1;
pub const PARSE_SHORT_READ: i32 = -2;
pub const PARSE_INVALID_IDENTIFICATION: i32 = -3;
pub const PARSE_OVERFLOW: i32 = -4;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CdoomRustWadHeader {
    pub identification: [u8; 4],
    pub num_lumps: i32,
    pub info_table_offset: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CdoomRustWadDirectoryEntry {
    pub file_pos: i32,
    pub size: i32,
    pub name: [u8; 8],
}

fn read_i32_le(bytes: &[u8]) -> i32 {
    i32::from_le_bytes(bytes.try_into().expect("slice has exactly four bytes"))
}

fn parse_header(bytes: &[u8]) -> Result<CdoomRustWadHeader, i32> {
    if bytes.len() < HEADER_SIZE {
        return Err(PARSE_SHORT_READ);
    }

    let identification: [u8; 4] = bytes[0..4]
        .try_into()
        .expect("slice has exactly four bytes");
    if identification != *b"IWAD" && identification != *b"PWAD" {
        return Err(PARSE_INVALID_IDENTIFICATION);
    }

    let num_lumps = read_i32_le(&bytes[4..8]);
    let info_table_offset = read_i32_le(&bytes[8..12]);
    if num_lumps < 0 || info_table_offset < 0 {
        return Err(PARSE_INVALID_ARGUMENT);
    }

    Ok(CdoomRustWadHeader {
        identification,
        num_lumps,
        info_table_offset,
    })
}

fn parse_directory_entry(bytes: &[u8]) -> Result<CdoomRustWadDirectoryEntry, i32> {
    if bytes.len() < DIRECTORY_ENTRY_SIZE {
        return Err(PARSE_SHORT_READ);
    }

    let file_pos = read_i32_le(&bytes[0..4]);
    let size = read_i32_le(&bytes[4..8]);
    if file_pos < 0 || size < 0 {
        return Err(PARSE_INVALID_ARGUMENT);
    }

    let name = bytes[8..16]
        .try_into()
        .expect("slice has exactly eight bytes");

    Ok(CdoomRustWadDirectoryEntry {
        file_pos,
        size,
        name,
    })
}

fn lump_name_hash_bytes(name: &[u8]) -> u32 {
    let mut result = 5381_u32;

    for byte in name
        .iter()
        .copied()
        .take(MAX_NAME_LEN)
        .take_while(|byte| *byte != 0)
    {
        result = ((result << 5) ^ result) ^ u32::from(byte.to_ascii_uppercase());
    }

    result
}

/// Parse the 12-byte WAD header from disk bytes.
///
/// Returns `0` on success and writes `out`; otherwise returns a negative parse
/// status and leaves `out` unchanged.
///
/// # Safety
///
/// `data` must point to `data_len` readable bytes. `out` must point to writable
/// storage for one [`CdoomRustWadHeader`].
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_wad_parse_header(
    data: *const u8,
    data_len: usize,
    out: *mut CdoomRustWadHeader,
) -> i32 {
    if data.is_null() || out.is_null() {
        return PARSE_INVALID_ARGUMENT;
    }

    // SAFETY: The caller promises that `data` points to `data_len` readable bytes.
    let bytes = unsafe { slice::from_raw_parts(data, data_len) };
    match parse_header(bytes) {
        Ok(header) => {
            // SAFETY: The caller promises that `out` points to writable storage.
            unsafe {
                *out = header;
            }
            PARSE_OK
        }
        Err(status) => status,
    }
}

/// Parse one 16-byte WAD directory entry from disk bytes.
///
/// Returns `0` on success and writes `out`; otherwise returns a negative parse
/// status and leaves `out` unchanged.
///
/// # Safety
///
/// `data` must point to `data_len` readable bytes. `out` must point to writable
/// storage for one [`CdoomRustWadDirectoryEntry`].
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_wad_parse_directory_entry(
    data: *const u8,
    data_len: usize,
    out: *mut CdoomRustWadDirectoryEntry,
) -> i32 {
    if data.is_null() || out.is_null() {
        return PARSE_INVALID_ARGUMENT;
    }

    // SAFETY: The caller promises that `data` points to `data_len` readable bytes.
    let bytes = unsafe { slice::from_raw_parts(data, data_len) };
    match parse_directory_entry(bytes) {
        Ok(entry) => {
            // SAFETY: The caller promises that `out` points to writable storage.
            unsafe {
                *out = entry;
            }
            PARSE_OK
        }
        Err(status) => status,
    }
}

/// Return the byte size needed for `num_lumps` WAD directory entries.
///
/// # Safety
///
/// `out_size` must point to writable storage for one `usize`.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_wad_directory_size(
    num_lumps: i32,
    out_size: *mut usize,
) -> i32 {
    if num_lumps < 0 || out_size.is_null() {
        return PARSE_INVALID_ARGUMENT;
    }

    let Some(size) = (num_lumps as usize).checked_mul(DIRECTORY_ENTRY_SIZE) else {
        return PARSE_OVERFLOW;
    };

    // SAFETY: The caller promises that `out_size` points to writable storage.
    unsafe {
        *out_size = size;
    }
    PARSE_OK
}

/// Hash a Doom lump name using the original WAD lookup algorithm.
///
/// # Safety
///
/// `name` must be either null, a valid C string, or an 8-byte WAD lump name.
/// The function reads at most 8 bytes, stopping earlier at NUL.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_wad_lump_name_hash(name: *const c_char) -> u32 {
    if name.is_null() {
        return 5381;
    }

    let mut bytes = [0_u8; MAX_NAME_LEN];
    let mut len = 0;

    for i in 0..MAX_NAME_LEN {
        // SAFETY: The caller promises readable bytes up to NUL or 8 bytes.
        let byte = unsafe { *name.add(i) as u8 };
        if byte == 0 {
            break;
        }

        bytes[i] = byte;
        len += 1;
    }

    lump_name_hash_bytes(&bytes[..len])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_iwad_header_as_little_endian() {
        let bytes = [b'I', b'W', b'A', b'D', 0x34, 0x12, 0, 0, 0x78, 0x56, 0, 0];

        let header = parse_header(&bytes).unwrap();

        assert_eq!(header.identification, *b"IWAD");
        assert_eq!(header.num_lumps, 0x1234);
        assert_eq!(header.info_table_offset, 0x5678);
    }

    #[test]
    fn rejects_unknown_wad_identification() {
        let bytes = [b'B', b'A', b'D', b'!', 1, 0, 0, 0, 12, 0, 0, 0];

        assert_eq!(parse_header(&bytes), Err(PARSE_INVALID_IDENTIFICATION));
    }

    #[test]
    fn parses_directory_entry_name_and_offsets() {
        let bytes = [
            0x20, 0, 0, 0, 0x10, 0, 0, 0, b'M', b'A', b'P', b'0', b'1', 0, 0, 0,
        ];

        let entry = parse_directory_entry(&bytes).unwrap();

        assert_eq!(entry.file_pos, 0x20);
        assert_eq!(entry.size, 0x10);
        assert_eq!(&entry.name, b"MAP01\0\0\0");
    }

    #[test]
    fn lump_name_hash_is_case_insensitive_and_limited_to_eight_bytes() {
        assert_eq!(
            lump_name_hash_bytes(b"playpal"),
            lump_name_hash_bytes(b"PLAYPAL")
        );
        assert_eq!(
            lump_name_hash_bytes(b"12345678ignored"),
            lump_name_hash_bytes(b"12345678"),
        );
    }
}
