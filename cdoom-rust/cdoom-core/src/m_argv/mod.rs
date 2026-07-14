use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

fn args_limit(argc: c_int, num_args: c_int) -> c_int {
    argc - num_args
}

pub fn check_parm_with_args(check: &[u8], argv: &[Option<&[u8]>], num_args: c_int) -> c_int {
    let limit = args_limit(argv.len() as c_int, num_args);

    for i in 1..limit {
        let Some(arg) = argv[i as usize] else {
            break;
        };

        if check.eq_ignore_ascii_case(arg) {
            return i;
        }
    }

    0
}

/// # Safety
///
/// `check` must point to a NUL-terminated string. `argv` must point to at
/// least `argc` entries that are either NULL or point to NUL-terminated
/// strings, matching Chocolate Doom's `myargv` contract.
pub unsafe fn check_parm_with_args_ffi(
    check: *const c_char,
    argc: c_int,
    argv: *const *const c_char,
    num_args: c_int,
) -> c_int {
    if check.is_null() || argv.is_null() {
        return 0;
    }

    let check = unsafe { CStr::from_ptr(check) };
    let limit = args_limit(argc, num_args);

    for i in 1..limit {
        let arg = unsafe { *argv.offset(i as isize) };

        if arg.is_null() {
            break;
        }

        let arg = unsafe { CStr::from_ptr(arg) };

        if check.to_bytes().eq_ignore_ascii_case(arg.to_bytes()) {
            return i;
        }
    }

    0
}
