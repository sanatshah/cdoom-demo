//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use std::ffi::c_void;
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

/// Checks for a command-line parameter in the supplied argv array.
///
/// Returns the argument index or `0` when not found, matching `M_CheckParm`.
///
/// # Safety
///
/// `check` must point to a NUL-terminated string. `argv` must contain at least
/// `argc` entries following Chocolate Doom's `myargv` layout.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_check_parm_with_args(
    check: *const c_char,
    argc: c_int,
    argv: *const *const c_char,
    num_args: c_int,
) -> c_int {
    unsafe { crate::m_argv::check_parm_with_args_ffi(check, argc, argv, num_args) }
}

/// Advances a Chocolate Doom cheat sequence by one key.
///
/// Returns `1` when the cheat has completed, otherwise `0`.
///
/// # Safety
///
/// `cht` must point to a valid `cheatseq_t`, which is layout-compatible with
/// `m_cheat::CheatSeq`.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_cht_check_cheat(cht: *mut c_void, key: c_char) -> c_int {
    if cht.is_null() {
        return 0;
    }

    let cht = unsafe { &mut *(cht as *mut CheatSeq) };

    crate::m_cheat::check_cheat(cht, key)
}

/// Copies the parameter buffer from a Chocolate Doom cheat sequence.
///
/// # Safety
///
/// `cht` must point to a valid `cheatseq_t`, and `buffer` must be large enough
/// for `parameter_chars` bytes.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_cht_get_param(cht: *const c_void, buffer: *mut c_char) {
    if cht.is_null() || buffer.is_null() {
        return;
    }

    let cht = unsafe { &*(cht as *const CheatSeq) };
    let buffer = unsafe { std::slice::from_raw_parts_mut(buffer, cht.parameter_chars as usize) };

    crate::m_cheat::get_param(cht, buffer);
}
