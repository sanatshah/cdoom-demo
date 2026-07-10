//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::OnceLock;

use crate::m_bbox::{self, BOX_COORDS};
use crate::m_fixed::{self, Fixed};

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
    m_fixed::fixed_mul(a, b)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_fixed_div(a: Fixed, b: Fixed) -> Fixed {
    m_fixed::fixed_div(a, b)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_m_clear_box(box_: *mut Fixed) {
    if box_.is_null() {
        return;
    }

    // SAFETY: Chocolate Doom passes a mutable fixed_t[4] bbox buffer.
    let box_ = unsafe { &mut *(box_ as *mut [Fixed; BOX_COORDS]) };
    m_bbox::clear_box(box_);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_m_add_to_box(box_: *mut Fixed, x: Fixed, y: Fixed) {
    if box_.is_null() {
        return;
    }

    // SAFETY: Chocolate Doom passes a mutable fixed_t[4] bbox buffer.
    let box_ = unsafe { &mut *(box_ as *mut [Fixed; BOX_COORDS]) };
    m_bbox::add_to_box(box_, x, y);
}
