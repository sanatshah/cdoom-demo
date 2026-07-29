//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::mus2mid;

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
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

/// Converts a MUS byte buffer into a newly allocated MIDI byte buffer.
///
/// Returns `0` on success and non-zero on failure.
///
/// # Safety
///
/// `input` must point to `input_len` readable bytes. `output` and `output_len`
/// must be valid writable pointers. On success, the caller owns `*output` and
/// must release it with `cdoom_rust_free_buffer(*output, *output_len)`.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_mus2mid(
    input: *const u8,
    input_len: usize,
    output: *mut *mut u8,
    output_len: *mut usize,
) -> i32 {
    if output.is_null() || output_len.is_null() {
        return 1;
    }

    *output = ptr::null_mut();
    *output_len = 0;

    if input.is_null() && input_len > 0 {
        return 1;
    }

    let input = slice::from_raw_parts(input, input_len);

    match mus2mid::convert_mus_to_midi(input) {
        Ok(midi) => {
            let mut midi = midi.into_boxed_slice();
            *output_len = midi.len();
            *output = midi.as_mut_ptr();
            std::mem::forget(midi);
            0
        }
        Err(_) => 1,
    }
}

/// Frees a Rust-owned byte buffer returned through the C ABI.
///
/// # Safety
///
/// `ptr` and `len` must match a buffer returned by cdoom-rust and not already
/// freed. Passing a NULL pointer is allowed.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_free_buffer(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }

    drop(Box::from_raw(slice::from_raw_parts_mut(ptr, len)));
}
