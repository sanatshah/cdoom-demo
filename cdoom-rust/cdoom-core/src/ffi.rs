//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::m_bbox;
use crate::m_fixed::Fixed;
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
    crate::m_fixed::fixed_mul(a, b)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_fixed_div(a: Fixed, b: Fixed) -> Fixed {
    crate::m_fixed::fixed_div(a, b)
}

/// Clears a Chocolate Doom bbox in-place.
///
/// # Safety
///
/// `box_` must point to at least four writable `fixed_t` entries using the
/// `BOXTOP`, `BOXBOTTOM`, `BOXLEFT`, `BOXRIGHT` layout from `m_bbox.h`.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_clear_box(box_: *mut Fixed) {
    let box_ = unsafe { &mut *(box_ as *mut [Fixed; m_bbox::BOX_LEN]) };
    m_bbox::clear_box(box_);
}

/// Adds a point to a Chocolate Doom bbox in-place.
///
/// # Safety
///
/// `box_` must point to at least four writable `fixed_t` entries using the
/// `BOXTOP`, `BOXBOTTOM`, `BOXLEFT`, `BOXRIGHT` layout from `m_bbox.h`.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_add_to_box(box_: *mut Fixed, x: Fixed, y: Fixed) {
    let box_ = unsafe { &mut *(box_ as *mut [Fixed; m_bbox::BOX_LEN]) };
    m_bbox::add_to_box(box_, x, y);
}
