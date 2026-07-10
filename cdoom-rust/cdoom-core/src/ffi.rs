//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::tables;
use crate::types::{AngleT, Byte, FixedT};
use std::ffi::CString;
use std::os::raw::c_char;
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

#[no_mangle]
pub extern "C" fn cdoom_rust_tables_finetangent() -> *const FixedT {
    tables::FINETANGENT.as_ptr()
}

#[no_mangle]
pub extern "C" fn cdoom_rust_tables_finetangent_len() -> usize {
    tables::FINETANGENT.len()
}

#[no_mangle]
pub extern "C" fn cdoom_rust_tables_finesine() -> *const FixedT {
    tables::FINESINE.as_ptr()
}

#[no_mangle]
pub extern "C" fn cdoom_rust_tables_finesine_len() -> usize {
    tables::FINESINE.len()
}

#[no_mangle]
pub extern "C" fn cdoom_rust_tables_finecosine() -> *const FixedT {
    tables::finecosine().as_ptr()
}

#[no_mangle]
pub extern "C" fn cdoom_rust_tables_finecosine_len() -> usize {
    tables::finecosine().len()
}

#[no_mangle]
pub extern "C" fn cdoom_rust_tables_tantoangle() -> *const AngleT {
    tables::TANTOANGLE.as_ptr()
}

#[no_mangle]
pub extern "C" fn cdoom_rust_tables_tantoangle_len() -> usize {
    tables::TANTOANGLE.len()
}

#[no_mangle]
pub extern "C" fn cdoom_rust_tables_gammatable() -> *const Byte {
    tables::GAMMATABLE.as_ptr().cast::<Byte>()
}

#[no_mangle]
pub extern "C" fn cdoom_rust_tables_gammatable_len() -> usize {
    tables::GAMMATABLE.len() * tables::GAMMA_COLS
}

#[no_mangle]
pub extern "C" fn cdoom_rust_tables_slope_div(num: u32, den: u32) -> i32 {
    tables::slope_div(num, den)
}
