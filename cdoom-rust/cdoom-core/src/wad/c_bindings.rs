//! C runtime bindings used by the WAD FFI layer.

use std::ffi::{c_char, c_int, c_uint, c_void};
use std::os::raw::c_ulong;

pub type LumpIndex = c_int;

/// Matches `wad_file_t` — only fields we read are declared.
#[repr(C)]
pub struct WadFile {
    pub file_class: *mut c_void,
    pub mapped: *mut u8,
    pub length: c_uint,
    pub path: *const c_char,
}

/// Matches `lumpinfo_t`.
#[repr(C)]
pub struct LumpInfo {
    pub name: [u8; 8],
    pub wad_file: *mut WadFile,
    pub position: c_int,
    pub size: c_int,
    pub cache: *mut c_void,
    pub next: LumpIndex,
}

extern "C" {
    pub fn W_OpenFile(path: *const c_char) -> *mut WadFile;
    pub fn W_CloseFile(wad: *mut WadFile);
    pub fn W_Read(
        wad: *mut WadFile,
        offset: c_uint,
        buffer: *mut c_void,
        buffer_len: usize,
    ) -> usize;

    pub fn I_Error(fmt: *const c_char, ...) -> !;
    pub fn I_Realloc(ptr: *mut c_void, size: usize) -> *mut c_void;

    pub fn Z_Malloc(size: c_int, tag: c_int, user: *mut c_void) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Z_ChangeTag2(ptr: *mut c_void, tag: c_int, file: *const c_char, line: c_int);

    pub fn M_ExtractFileBase(path: *const c_char, dest: *mut c_char);
    pub fn M_BaseName(path: *const c_char) -> *const c_char;

    pub fn V_BeginRead(nbytes: usize);

    pub fn strcasecmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strncasecmp(s1: *const c_char, s2: *const c_char, n: c_ulong) -> c_int;
    pub fn printf(fmt: *const c_char, ...) -> c_int;
    pub fn strdup(s: *const c_char) -> *mut c_char;
    pub fn free(ptr: *mut c_void);
}

pub const PU_STATIC: c_int = 1;
pub const PU_CACHE: c_int = 8;
