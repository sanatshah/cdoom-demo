//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.
#![allow(non_snake_case, non_upper_case_globals)]

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
pub static finetangent: [FixedT; tables::FINEANGLES / 2] = tables::FINETANGENT;

#[no_mangle]
pub static finesine: [FixedT; tables::FINEANGLES * 5 / 4] = tables::FINESINE;

#[no_mangle]
pub static mut finecosine: *const FixedT =
    unsafe { finesine.as_ptr().add(tables::FINECOSINE_OFFSET) };

#[no_mangle]
pub static tantoangle: [AngleT; tables::SLOPERANGE as usize + 1] = tables::TANTOANGLE;

#[no_mangle]
pub static gammatable: [Byte; tables::GAMMATABLE_ROWS * tables::GAMMATABLE_COLUMNS] =
    tables::GAMMATABLE_FLAT;

#[no_mangle]
pub extern "C" fn SlopeDiv(num: u32, den: u32) -> i32 {
    tables::slope_div(num, den)
}
