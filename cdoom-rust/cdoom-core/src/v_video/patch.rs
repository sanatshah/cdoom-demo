//! Doom patch layout helpers shared by the Rust video routines.

use std::ffi::c_void;
use std::ptr;

const PATCH_COLUMN_OFS_OFFSET: usize = 8;

/// Header prefix for Doom patch lumps.
///
/// The real on-disk layout has `columnofs[width]`; the C header declares eight
/// entries as a historical convenience. Rust treats incoming patch pointers as
/// opaque byte blobs and reads only the fields needed by the drawing code.
#[repr(C, packed)]
pub struct PatchHeader {
    pub width: i16,
    pub height: i16,
    pub leftoffset: i16,
    pub topoffset: i16,
    pub columnofs: [i32; 8],
}

pub type PatchPtr = *mut c_void;

fn read_i16_le(patch: PatchPtr, offset: usize) -> i16 {
    // SAFETY: Callers pass a pointer to a Chocolate Doom patch lump. Unaligned
    // reads match the packed C struct and WAD byte layout.
    let value = unsafe { ptr::read_unaligned((patch as *const u8).add(offset) as *const i16) };
    i16::from_le(value)
}

fn read_i32_le(patch: PatchPtr, offset: usize) -> i32 {
    // SAFETY: See read_i16_le.
    let value = unsafe { ptr::read_unaligned((patch as *const u8).add(offset) as *const i32) };
    i32::from_le(value)
}

pub fn width(patch: PatchPtr) -> i32 {
    i32::from(read_i16_le(patch, 0))
}

pub fn height(patch: PatchPtr) -> i32 {
    i32::from(read_i16_le(patch, 2))
}

pub fn left_offset(patch: PatchPtr) -> i32 {
    i32::from(read_i16_le(patch, 4))
}

pub fn top_offset(patch: PatchPtr) -> i32 {
    i32::from(read_i16_le(patch, 6))
}

pub fn column_offset(patch: PatchPtr, column: i32) -> usize {
    let offset = PATCH_COLUMN_OFS_OFFSET + column as usize * std::mem::size_of::<i32>();
    read_i32_le(patch, offset) as usize
}
