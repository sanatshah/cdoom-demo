//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::sync::OnceLock;

use crate::{m_argv, m_cheat};

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
pub unsafe extern "C" fn cdoom_rust_m_check_parm_with_args(
    argc: c_int,
    argv: *const *const c_char,
    check: *const c_char,
    num_args: c_int,
) -> c_int {
    unsafe { m_argv::check_parm_with_args(argc, argv, check, num_args) }
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_check_parm(
    argc: c_int,
    argv: *const *const c_char,
    check: *const c_char,
) -> c_int {
    unsafe { m_argv::check_parm(argc, argv, check) }
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_parm_exists(
    argc: c_int,
    argv: *const *const c_char,
    check: *const c_char,
) -> c_int {
    i32::from(unsafe { m_argv::parm_exists(argc, argv, check) })
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_find_response_file(
    argc: *mut c_int,
    argv: *mut *mut *mut c_char,
) {
    unsafe { m_argv::find_response_file(argc, argv) }
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_get_executable_name(path: *const c_char) -> *const c_char {
    unsafe { m_argv::get_executable_name(path) }
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_set_exe_dir(path: *const c_char, exedir: *mut *mut c_char) {
    unsafe { m_argv::set_exe_dir(path, exedir) }
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_cht_check_cheat(
    cht: *mut m_cheat::CheatSeq,
    key: c_char,
) -> c_int {
    i32::from(unsafe { cht.as_mut() }.is_some_and(|cht| m_cheat::check_cheat(cht, key)))
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_cht_get_param(
    cht: *const m_cheat::CheatSeq,
    buffer: *mut c_char,
) {
    let Some(cht) = (unsafe { cht.as_ref() }) else {
        return;
    };

    if buffer.is_null() {
        return;
    }

    let count = cht.parameter_chars.max(0) as usize;
    let buffer = unsafe { std::slice::from_raw_parts_mut(buffer, count) };
    m_cheat::get_param(cht, buffer);
}
