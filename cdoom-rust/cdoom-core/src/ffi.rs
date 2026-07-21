//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uint};
use std::sync::OnceLock;

use crate::p_rejectpad;

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

/// Pads a REJECT lump tail using Chocolate Doom's vanilla overflow emulation.
///
/// # Safety
///
/// When `len` is non-zero, `array` must point to at least `len` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_pad_reject_array(
    array: *mut u8,
    len: c_uint,
    total_lines: c_int,
    pad_with_ff: c_int,
) {
    if len == 0 {
        return;
    }

    if array.is_null() {
        return;
    }

    // SAFETY: The caller promises that `array` points to `len` writable bytes.
    let array = unsafe { std::slice::from_raw_parts_mut(array, len as usize) };
    p_rejectpad::pad_reject_array(array, total_lines, pad_with_ff != 0);
}
