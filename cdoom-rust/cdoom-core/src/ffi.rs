//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::m_bbox;
use crate::m_fixed::{self, Fixed};
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

/// Multiplies two Chocolate Doom 16.16 fixed-point values.
#[no_mangle]
pub extern "C" fn cdoom_rust_fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    m_fixed::fixed_mul(a, b)
}

/// Divides two Chocolate Doom 16.16 fixed-point values.
#[no_mangle]
pub extern "C" fn cdoom_rust_fixed_div(a: Fixed, b: Fixed) -> Fixed {
    m_fixed::fixed_div(a, b)
}

/// Clears a four-element Chocolate Doom bounding box.
///
/// # Safety
///
/// `box_` must point to at least four writable `fixed_t` values.
#[no_mangle]
pub extern "C" fn cdoom_rust_m_clear_box(box_: *mut Fixed) {
    // SAFETY: C callers pass Chocolate Doom bbox arrays with four fixed_t slots.
    let box_ = unsafe { &mut *box_.cast::<[Fixed; m_bbox::BOX_COUNT]>() };
    m_bbox::clear_box(box_);
}

/// Adds a point to a four-element Chocolate Doom bounding box.
///
/// # Safety
///
/// `box_` must point to at least four writable `fixed_t` values.
#[no_mangle]
pub extern "C" fn cdoom_rust_m_add_to_box(box_: *mut Fixed, x: Fixed, y: Fixed) {
    // SAFETY: C callers pass Chocolate Doom bbox arrays with four fixed_t slots.
    let box_ = unsafe { &mut *box_.cast::<[Fixed; m_bbox::BOX_COUNT]>() };
    m_bbox::add_to_box(box_, x, y);
}
