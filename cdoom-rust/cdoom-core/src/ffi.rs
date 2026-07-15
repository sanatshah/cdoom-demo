//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::m_argv;
use crate::m_cheat::{self, CheatSeq};
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
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

/// Rust implementation of `M_CheckParmWithArgs`.
///
/// # Safety
///
/// `argv` must point to an array of `argc` C string pointers.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_check_parm_with_args(
    argc: c_int,
    argv: *mut *mut c_char,
    check: *const c_char,
    num_args: c_int,
) -> c_int {
    unsafe { m_argv::check_parm_with_args(argc, argv, check, num_args) }
}

/// Rust implementation of `M_CheckParm`.
///
/// # Safety
///
/// `argv` must point to an array of `argc` C string pointers.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_check_parm(
    argc: c_int,
    argv: *mut *mut c_char,
    check: *const c_char,
) -> c_int {
    unsafe { m_argv::check_parm_with_args(argc, argv, check, 0) }
}

/// Rust implementation of `M_ParmExists`.
///
/// # Safety
///
/// `argv` must point to an array of `argc` C string pointers.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_parm_exists(
    argc: c_int,
    argv: *mut *mut c_char,
    check: *const c_char,
) -> c_int {
    (unsafe { m_argv::check_parm_with_args(argc, argv, check, 0) } != 0) as c_int
}

/// Rust implementation of `M_FindResponseFile`.
///
/// # Safety
///
/// `argc` and `argv` must point to Chocolate Doom's `myargc`/`myargv` globals.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_find_response_file(
    argc: *mut c_int,
    argv: *mut *mut *mut c_char,
) {
    unsafe { m_argv::find_response_file(argc, argv) }
}

/// Rust implementation of `M_GetExecutableName`.
///
/// # Safety
///
/// `argv` must point to an argv array with a valid argv[0].
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_get_executable_name(argv: *mut *mut c_char) -> *const c_char {
    unsafe { m_argv::get_executable_name(argv) }
}

/// Rust implementation of `M_SetExeDir`.
///
/// # Safety
///
/// `argv` must point to an argv array with a valid argv[0].
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_set_exe_dir(argv: *mut *mut c_char) -> *mut c_char {
    unsafe { m_argv::set_exe_dir(argv) }
}

/// Rust implementation of `cht_CheckCheat`.
///
/// # Safety
///
/// `cht` must point to a valid C `cheatseq_t`.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_cht_check_cheat(cht: *mut CheatSeq, key: c_char) -> c_int {
    unsafe { m_cheat::check_cheat(cht, key) }
}

/// Rust implementation of `cht_GetParam`.
///
/// # Safety
///
/// `cht` and `buffer` must be valid for `cht.parameter_chars` bytes.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_cht_get_param(cht: *const CheatSeq, buffer: *mut c_char) {
    unsafe { m_cheat::get_param(cht, buffer) }
}
