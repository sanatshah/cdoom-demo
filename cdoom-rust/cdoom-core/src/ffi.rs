//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::OnceLock;

use crate::m_bbox;
use crate::m_fixed::{self, Angle, Fixed};

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

/// Multiplies two 16.16 fixed-point values with Chocolate Doom semantics.
#[no_mangle]
pub extern "C" fn cdoom_rust_fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    m_fixed::fixed_mul(a, b)
}

/// Divides two 16.16 fixed-point values with Chocolate Doom saturation semantics.
#[no_mangle]
pub extern "C" fn cdoom_rust_fixed_div(a: Fixed, b: Fixed) -> Fixed {
    m_fixed::fixed_div(a, b)
}

/// Converts a BAM angle to a fine table index.
#[no_mangle]
pub extern "C" fn cdoom_rust_angle_to_fine_index(angle: Angle) -> u32 {
    m_fixed::angle_to_fine_index(angle)
}

/// Resets a four-entry Chocolate Doom bbox array.
///
/// # Safety
///
/// `box_values` must point to at least four writable `fixed_t` entries.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_clear_box(box_values: *mut Fixed) {
    unsafe { m_bbox::clear_box_ptr(box_values) };
}

/// Adds a point to a four-entry Chocolate Doom bbox array.
///
/// # Safety
///
/// `box_values` must point to at least four writable `fixed_t` entries.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_add_to_box(box_values: *mut Fixed, x: Fixed, y: Fixed) {
    unsafe { m_bbox::add_to_box_ptr(box_values, x, y) };
}
