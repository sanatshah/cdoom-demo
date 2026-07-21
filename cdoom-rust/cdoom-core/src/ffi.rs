//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::p_rejectpad;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uint};
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

/// Pads a short REJECT lump using Chocolate Doom's vanilla overflow emulation.
///
/// # Safety
///
/// `array` must point to at least `len` writable bytes when `len` is non-zero.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_pad_reject_array(
    array: *mut u8,
    len: c_uint,
    totallines: c_int,
    pad_with_ff: c_int,
) {
    if len == 0 {
        return;
    }

    let Ok(len) = usize::try_from(len) else {
        return;
    };

    if array.is_null() {
        return;
    }

    // SAFETY: The caller provides a writable buffer of `len` bytes.
    let array = unsafe { std::slice::from_raw_parts_mut(array, len) };
    p_rejectpad::pad_reject_array(array, totallines, pad_with_ff != 0);
}
