//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::tables;
use crate::types::{Angle, Byte, Fixed};
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

/// Returns a pointer to the 4096-entry tangent lookup table.
#[no_mangle]
pub extern "C" fn cdoom_rust_tables_finetangent() -> *const Fixed {
    tables::finetangent().as_ptr()
}

/// Returns a pointer to the 10240-entry sine lookup table.
#[no_mangle]
pub extern "C" fn cdoom_rust_tables_finesine() -> *const Fixed {
    tables::finesine().as_ptr()
}

/// Returns a pointer to the cosine view into the sine lookup table.
#[no_mangle]
pub extern "C" fn cdoom_rust_tables_finecosine() -> *const Fixed {
    tables::finecosine().as_ptr()
}

/// Returns a pointer to the 2049-entry tangent-to-angle lookup table.
#[no_mangle]
pub extern "C" fn cdoom_rust_tables_tantoangle() -> *const Angle {
    tables::tantoangle().as_ptr()
}

/// Returns a pointer to the flattened 5x256 gamma correction table.
#[no_mangle]
pub extern "C" fn cdoom_rust_tables_gammatable() -> *const Byte {
    tables::gammatable_flat().as_ptr()
}

/// Rust implementation of `SlopeDiv`.
#[no_mangle]
pub extern "C" fn cdoom_rust_tables_slope_div(num: c_uint, den: c_uint) -> c_int {
    tables::slope_div(num, den)
}
