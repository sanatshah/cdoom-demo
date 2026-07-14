//! Dedicated server entry helpers from `d_dedicated.c` and `net_dedicated.c`.

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::slice;

pub type DedicatedStep = Option<extern "C" fn()>;

const NOT_DEDICATED_OPTIONS: [&[u8]; 26] = [
    b"-deh\0",
    b"-iwad\0",
    b"-cdrom\0",
    b"-gameversion\0",
    b"-nomonsters\0",
    b"-respawn\0",
    b"-fast\0",
    b"-altdeath\0",
    b"-deathmatch\0",
    b"-turbo\0",
    b"-merge\0",
    b"-af\0",
    b"-as\0",
    b"-aa\0",
    b"-file\0",
    b"-wart\0",
    b"-skill\0",
    b"-episode\0",
    b"-timer\0",
    b"-avg\0",
    b"-warp\0",
    b"-loadgame\0",
    b"-longtics\0",
    b"-extratics\0",
    b"-dup\0",
    b"-shorttics\0",
];

pub fn net_client_run() {}

pub fn dedicated_main(
    print_banner: DedicatedStep,
    init_zone: DedicatedStep,
    run_server: DedicatedStep,
) {
    if let Some(print_banner) = print_banner {
        print_banner();
    }

    if let Some(init_zone) = init_zone {
        init_zone();
    }

    if let Some(run_server) = run_server {
        run_server();
    }
}

pub unsafe fn rejected_option(argc: c_int, argv: *const *const c_char) -> *const c_char {
    if argc <= 1 || argv.is_null() {
        return std::ptr::null();
    }

    // SAFETY: `myargv` is a C argv array with `myargc` entries. Individual
    // entries may be NULL after response-file processing, matching M_CheckParm.
    let args = unsafe { slice::from_raw_parts(argv, argc as usize) };

    for option in NOT_DEDICATED_OPTIONS {
        let option_without_nul = &option[..option.len() - 1];

        for arg in args.iter().skip(1).copied() {
            if arg.is_null() {
                continue;
            }

            // SAFETY: Non-NULL argv entries are NUL-terminated C strings.
            let arg = unsafe { CStr::from_ptr(arg) };
            if arg.to_bytes().eq_ignore_ascii_case(option_without_nul) {
                return option.as_ptr().cast();
            }
        }
    }

    std::ptr::null()
}
