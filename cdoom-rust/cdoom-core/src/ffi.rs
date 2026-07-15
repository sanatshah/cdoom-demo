//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::sync::OnceLock;

use crate::m_cheat::CheatSeq;

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

/// Checks for the given parameter in the process command-line arguments.
///
/// # Safety
///
/// `check` must point to a valid NUL-terminated C string. `myargc` and
/// `myargv` must have been initialized by Chocolate Doom startup code.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_check_parm_with_args(
    check: *const c_char,
    num_args: c_int,
) -> c_int {
    unsafe { crate::m_argv::check_parm_with_args(check, num_args) }
}

/// Checks for the given parameter without requiring following arguments.
///
/// # Safety
///
/// `check` must point to a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_check_parm(check: *const c_char) -> c_int {
    unsafe { crate::m_argv::check_parm_with_args(check, 0) }
}

/// Returns non-zero when the given command-line parameter exists.
///
/// # Safety
///
/// `check` must point to a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_parm_exists(check: *const c_char) -> c_int {
    unsafe { crate::m_argv::parm_exists(check) }
}

/// Expands `@file` and `-response file` arguments in-place.
///
/// # Safety
///
/// `myargc` and `myargv` must be initialized with C-allocated strings/vector
/// following Chocolate Doom startup ownership rules.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_find_response_file() {
    unsafe { crate::m_argv::find_response_file() }
}

/// Returns the executable basename from `myargv[0]`.
///
/// # Safety
///
/// `myargv[0]` must point to a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_get_executable_name() -> *const c_char {
    unsafe { crate::m_argv::executable_name() }
}

/// Sets the process executable directory global.
///
/// # Safety
///
/// `myargv[0]` must point to a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_set_exe_dir() {
    unsafe { crate::m_argv::set_exe_dir() }
}

/// Checks a cheat sequence state machine for a newly pressed key.
///
/// # Safety
///
/// `cheat` must point to a valid `cdoom_rust_CheatSeq`, layout-compatible with
/// Chocolate Doom's `cheatseq_t`.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_cht_check_cheat(cheat: *mut CheatSeq, key: c_char) -> c_int {
    let cheat = unsafe { &mut *cheat };
    crate::m_cheat::check_cheat(cheat, key)
}

/// Copies the collected cheat parameter bytes into `buffer`.
///
/// # Safety
///
/// `cheat` must be valid and `buffer` must have room for `parameter_chars`
/// bytes.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_cht_get_param(cheat: *mut CheatSeq, buffer: *mut c_char) {
    let cheat = unsafe { &*cheat };
    crate::m_cheat::get_param(cheat, buffer);
}
