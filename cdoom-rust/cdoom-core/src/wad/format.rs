//! On-disk WAD layout types (little-endian).

use std::mem;

/// WAD container kind (`IWAD` or `PWAD`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WadKind {
    Iwad,
    Pwad,
}

impl WadKind {
    pub fn from_ident(bytes: &[u8; 4]) -> Option<Self> {
        match bytes {
            b"IWAD" => Some(Self::Iwad),
            b"PWAD" => Some(Self::Pwad),
            _ => None,
        }
    }
}

/// 12-byte WAD header.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct WadHeader {
    pub identification: [u8; 4],
    pub numlumps: i32,
    pub infotableofs: i32,
}

/// 16-byte directory entry.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct FileLump {
    pub filepos: i32,
    pub size: i32,
    pub name: [u8; 8],
}

pub const WAD_HEADER_SIZE: usize = mem::size_of::<WadHeader>();
pub const FILE_LUMP_SIZE: usize = mem::size_of::<FileLump>();
pub const PWAD_LUMP_LIMIT: i32 = 4046;

/// Decode a little-endian i32 from raw bytes.
#[inline]
pub fn le_i32(bytes: [u8; 4]) -> i32 {
    i32::from_le_bytes(bytes)
}

/// Decode a little-endian i32 already stored in native layout after read.
#[inline]
pub fn swap_le_i32(value: i32) -> i32 {
    i32::from_le_bytes(value.to_le_bytes())
}
