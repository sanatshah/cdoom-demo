//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::m_bbox::{add_to_box, clear_box, BOX_COORDS};
use crate::m_fixed::{angle_to_fine_index, fixed_div, fixed_mul, Fixed};
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
pub extern "C" fn cdoom_rust_fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    fixed_mul(a, b)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_fixed_div(a: Fixed, b: Fixed) -> Fixed {
    fixed_div(a, b)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_angle_to_fine_index(angle: u32) -> i32 {
    angle_to_fine_index(angle)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_m_clear_box(box_coords: *mut Fixed) {
    if box_coords.is_null() {
        return;
    }

    // SAFETY: C callers pass a mutable four-entry bbox array matching m_bbox.h.
    let box_coords = unsafe { &mut *(box_coords as *mut [Fixed; BOX_COORDS]) };
    clear_box(box_coords);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_m_add_to_box(box_coords: *mut Fixed, x: Fixed, y: Fixed) {
    if box_coords.is_null() {
        return;
    }

    // SAFETY: C callers pass a mutable four-entry bbox array matching m_bbox.h.
    let box_coords = unsafe { &mut *(box_coords as *mut [Fixed; BOX_COORDS]) };
    add_to_box(box_coords, x, y);
}
