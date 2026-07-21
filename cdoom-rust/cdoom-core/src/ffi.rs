//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::p_rejectpad;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uint};
use std::slice;
use std::sync::OnceLock;

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

/// Pads a too-short REJECT lump using Chocolate Doom's Vanilla-compatible bytes.
///
/// # Safety
///
/// `array` must be valid for writes of `len` bytes when `len` is non-zero.
#[no_mangle]
pub extern "C" fn cdoom_rust_pad_reject_array(
    array: *mut u8,
    len: c_uint,
    totallines: c_int,
    pad_with_ff: bool,
) {
    if len == 0 || array.is_null() {
        return;
    }

    // SAFETY: The C caller owns the REJECT lump buffer and passes its writable
    // length. This wrapper keeps the raw pointer handling at the ABI boundary.
    let array = unsafe { slice::from_raw_parts_mut(array, len as usize) };
    p_rejectpad::pad_reject_array(array, totallines, pad_with_ff);
}
