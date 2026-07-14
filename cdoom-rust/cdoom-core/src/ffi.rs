//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::OnceLock;

use crate::w_wad;

static VERSION: OnceLock<CString> = OnceLock::new();

fn version_cstr() -> &'static CString {
    VERSION.get_or_init(|| {
        CString::new(env!("CARGO_PKG_VERSION")).expect("version must not contain NUL")
    })
}

/// Returns a pointer to a static, NUL-terminated version string.
///
/// # Safety
///
/// The returned pointer is valid for the process lifetime and must not be freed.
#[no_mangle]
pub extern "C" fn cdoom_rust_version() -> *const c_char {
    version_cstr().as_ptr()
}

/// One-time initialization hook for future Rust subsystems.
///
/// Returns `0` on success. Reserved for later migration phases.
#[no_mangle]
pub extern "C" fn cdoom_rust_init() -> i32 {
    0
}

/// Parses a WAD or single-lump file and returns summary metadata.
///
/// Returns 0 on success. Non-zero values are stable status codes from the
/// Rust WAD parser.
///
/// # Safety
///
/// `path` must be a valid NUL-terminated string. Output pointers may be NULL
/// if the corresponding field is not needed. Name buffers must match the
/// supplied lengths.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_w_wad_file_summary(
    path: *const c_char,
    lump_count: *mut u32,
    total_size: *mut u32,
    directory_checksum: *mut u64,
    first_name: *mut c_char,
    first_name_len: usize,
    last_name: *mut c_char,
    last_name_len: usize,
) -> i32 {
    w_wad::ffi_file_summary(
        path,
        lump_count,
        total_size,
        directory_checksum,
        first_name,
        first_name_len,
        last_name,
        last_name_len,
    )
}
