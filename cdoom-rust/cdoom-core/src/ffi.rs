//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::OnceLock;

use crate::m_argv;
use crate::m_cheat::{self, CheatSeq};

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
    argv: *mut *mut c_char,
    check: *const c_char,
    num_args: c_int,
) -> c_int {
    m_argv::check_parm_with_c_args(argc, argv, check, num_args)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_find_response_file(
    argc: *mut c_int,
    argv: *mut *mut *mut c_char,
    error_message: *mut *mut c_char,
    missing_file: *mut c_int,
) -> c_int {
    if !error_message.is_null() {
        *error_message = ptr::null_mut();
    }

    if !missing_file.is_null() {
        *missing_file = 0;
    }

    match m_argv::find_response_file_c(argc, argv) {
        Ok(()) => 0,
        Err(error) => {
            if !missing_file.is_null() && matches!(error, m_argv::ResponseError::MissingFile(_)) {
                *missing_file = 1;
            }

            if !error_message.is_null() {
                *error_message = m_argv::duplicate_error_message(&error);
            }

            1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_get_executable_name(argv0: *const c_char) -> *const c_char {
    m_argv::executable_name(argv0)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_set_exe_dir(
    argv0: *const c_char,
    exedir: *mut *mut c_char,
    error_message: *mut *mut c_char,
) -> c_int {
    if !error_message.is_null() {
        *error_message = ptr::null_mut();
    }

    if exedir.is_null() {
        if !error_message.is_null() {
            *error_message =
                m_argv::duplicate_error_message(&m_argv::ResponseError::AllocationFailed);
        }

        return 1;
    }

    match m_argv::duplicate_exe_dir(argv0) {
        Ok(value) => {
            *exedir = value;
            0
        }
        Err(error) => {
            if !error_message.is_null() {
                *error_message = m_argv::duplicate_error_message(&error);
            }

            1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_cht_check_cheat(cheat: *mut CheatSeq, key: c_char) -> c_int {
    if cheat.is_null() {
        return 0;
    }

    m_cheat::check_cheat(&mut *cheat, key) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_cht_get_param(cheat: *const CheatSeq, buffer: *mut c_char) {
    if cheat.is_null() || buffer.is_null() {
        return;
    }

    m_cheat::get_param(&*cheat, buffer);
}
