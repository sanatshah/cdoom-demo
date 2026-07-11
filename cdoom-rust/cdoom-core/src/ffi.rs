//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::{m_bbox, m_fixed};
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
pub extern "C" fn cdoom_rust_fixed_mul(a: i32, b: i32) -> i32 {
    m_fixed::fixed_mul(a, b)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_fixed_div(a: i32, b: i32) -> i32 {
    m_fixed::fixed_div(a, b)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_angle_to_fine_index(angle: u32) -> u32 {
    m_fixed::angle_to_fine_index(angle)
}

/// Clears a four-element Chocolate Doom bbox array in-place.
///
/// # Safety
///
/// `bbox` must point to at least four writable `fixed_t` elements, matching
/// the contract of `M_ClearBox` in the C code.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_clear_box(bbox: *mut i32) {
    let bbox = unsafe { &mut *(bbox as *mut [i32; m_bbox::BOX_SIZE]) };
    m_bbox::clear_box(bbox);
}

/// Adds a point to a four-element Chocolate Doom bbox array in-place.
///
/// # Safety
///
/// `bbox` must point to at least four writable `fixed_t` elements, matching
/// the contract of `M_AddToBox` in the C code.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_add_to_box(bbox: *mut i32, x: i32, y: i32) {
    let bbox = unsafe { &mut *(bbox as *mut [i32; m_bbox::BOX_SIZE]) };
    m_bbox::add_to_box(bbox, x, y);
}
