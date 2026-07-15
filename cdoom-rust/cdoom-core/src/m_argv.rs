//! Command-line argument helpers migrated from `m_argv.c`.

use std::ffi::CStr;
use std::fs;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

const MAXARGVS: usize = 100;
const DIR_SEPARATOR: u8 = b'/';

unsafe extern "C" {
    fn exit(status: c_int) -> !;
    fn free(ptr: *mut c_void);
    fn malloc(size: usize) -> *mut c_void;
    fn printf(format: *const c_char, ...) -> c_int;
}

#[no_mangle]
pub static mut myargc: c_int = 0;

#[no_mangle]
pub static mut myargv: *mut *mut c_char = ptr::null_mut();

#[no_mangle]
pub static mut exedir: *mut c_char = ptr::null_mut();

fn cstr_bytes(ptr: *const c_char) -> &'static [u8] {
    // SAFETY: The C ABI passes NUL-terminated strings with process lifetime
    // ownership semantics. Callers must not pass NULL for arguments that C
    // would dereference.
    unsafe { CStr::from_ptr(ptr).to_bytes() }
}

fn ascii_eq_ignore_case(lhs: &[u8], rhs: &[u8]) -> bool {
    lhs.len() == rhs.len()
        && lhs
            .iter()
            .zip(rhs.iter())
            .all(|(&a, &b)| a.eq_ignore_ascii_case(&b))
}

fn duplicate_bytes(bytes: &[u8]) -> *mut c_char {
    // SAFETY: `malloc` returns C-owned memory compatible with later `free`.
    let ptr = unsafe { malloc(bytes.len() + 1) as *mut c_char };
    if ptr.is_null() {
        fatal("Failed to duplicate argument string");
    }

    // SAFETY: `ptr` points at `bytes.len() + 1` writable bytes.
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, ptr, bytes.len());
        *ptr.add(bytes.len()) = 0;
    }

    ptr
}

fn fatal(message: &str) -> ! {
    eprintln!("{message}");
    // SAFETY: Matches the fatal process exit style used by the original
    // response-file parser for unrecoverable setup errors.
    unsafe { exit(1) }
}

fn allocate_argv(capacity: usize) -> *mut *mut c_char {
    // SAFETY: `malloc` returns C-owned memory compatible with later `free`.
    let newargv =
        unsafe { malloc(std::mem::size_of::<*mut c_char>() * capacity) as *mut *mut c_char };
    if newargv.is_null() {
        fatal("Failed to allocate argument vector");
    }

    // SAFETY: `newargv` points at `capacity` writable pointer slots.
    unsafe {
        ptr::write_bytes(newargv, 0, capacity);
    }

    newargv
}

fn print_found_response_file(filename: *const c_char) {
    const FORMAT: &[u8] = b"Found response file %s!\n\0";

    // SAFETY: `FORMAT` and `filename` are NUL-terminated C strings.
    unsafe {
        printf(FORMAT.as_ptr() as *const c_char, filename);
    }
}

fn no_such_response_file() -> ! {
    const MESSAGE: &[u8] = b"\nNo such response file!\0";

    // SAFETY: `MESSAGE` is a static NUL-terminated string.
    unsafe {
        printf(MESSAGE.as_ptr() as *const c_char);
        exit(1);
    }
}

fn load_response_file(argv_index: c_int, filename: *const c_char) {
    let filename_bytes = cstr_bytes(filename);
    let Ok(mut file) = fs::read(String::from_utf8_lossy(filename_bytes).as_ref()) else {
        no_such_response_file();
    };

    print_found_response_file(filename);

    let size = file.len();
    file.push(0);

    let newargv = allocate_argv(MAXARGVS);
    let mut newargc: c_int = 0;

    if argv_index as usize >= MAXARGVS {
        fatal("Too many arguments up to the response file!");
    }

    // SAFETY: `myargv` contains at least `argv_index` entries set up by main.
    unsafe {
        for i in 0..argv_index as isize {
            *newargv.offset(i) = *myargv.offset(i);
            *myargv.offset(i) = ptr::null_mut();
            newargc += 1;
        }
    }

    let mut k = 0usize;
    while k < size {
        while k < size && file[k].is_ascii_whitespace() {
            k += 1;
        }

        if k >= size {
            break;
        }

        let argstart;

        if file[k] == b'"' {
            k += 1;
            argstart = k;

            while k < size && file[k] != b'"' && file[k] != b'\n' {
                k += 1;
            }

            if k >= size || file[k] == b'\n' {
                fatal(&format!(
                    "Quotes unclosed in response file '{}'",
                    String::from_utf8_lossy(filename_bytes)
                ));
            }

            file[k] = 0;
            k += 1;
        } else {
            argstart = k;

            while k < size && !file[k].is_ascii_whitespace() {
                k += 1;
            }

            file[k] = 0;
            k += 1;
        }

        if newargc as usize >= MAXARGVS {
            fatal("Too many arguments in the response file!");
        }

        let arg = duplicate_bytes(
            &file[argstart..file[argstart..].iter().position(|&b| b == 0).unwrap() + argstart],
        );

        // SAFETY: `newargc < MAXARGVS`, so this slot is writable.
        unsafe {
            *newargv.offset(newargc as isize) = arg;
        }
        newargc += 1;
    }

    // SAFETY: `myargv` contains `myargc` entries set up by main.
    unsafe {
        if newargc + myargc - (argv_index + 1) >= MAXARGVS as c_int {
            fatal("Too many arguments following the response file!");
        }

        for i in (argv_index + 1)..myargc {
            *newargv.offset(newargc as isize) = *myargv.offset(i as isize);
            *myargv.offset(i as isize) = ptr::null_mut();
            newargc += 1;
        }

        for i in 0..myargc {
            let old = *myargv.offset(i as isize);
            if !old.is_null() {
                free(old as *mut c_void);
                *myargv.offset(i as isize) = ptr::null_mut();
            }
        }

        free(myargv as *mut c_void);
        myargv = newargv;
        myargc = newargc;
    }
}

pub unsafe fn check_parm_with_args(check: *const c_char, num_args: c_int) -> c_int {
    let check = cstr_bytes(check);

    // SAFETY: Accesses the process-global argument vector maintained by this
    // module and initialized from C `main`.
    unsafe {
        let limit = myargc - num_args;
        let mut i = 1;

        while i < limit {
            let arg = *myargv.offset(i as isize);
            if arg.is_null() {
                break;
            }

            if ascii_eq_ignore_case(check, cstr_bytes(arg)) {
                return i;
            }

            i += 1;
        }
    }

    0
}

pub unsafe fn parm_exists(check: *const c_char) -> c_int {
    if unsafe { check_parm_with_args(check, 0) } != 0 {
        1
    } else {
        0
    }
}

pub unsafe fn find_response_file() {
    // SAFETY: Accesses the process-global argument vector maintained by this
    // module and initialized from C `main`.
    unsafe {
        let mut i = 1;
        while i < myargc {
            let arg = *myargv.offset(i as isize);
            if !arg.is_null() && *arg == b'@' as c_char {
                load_response_file(i, arg.add(1));
            }
            i += 1;
        }

        loop {
            let response = b"-response\0";
            let index = check_parm_with_args(response.as_ptr() as *const c_char, 1);
            if index <= 0 {
                break;
            }

            free(*myargv.offset(index as isize) as *mut c_void);
            *myargv.offset(index as isize) = duplicate_bytes(b"-_");
            load_response_file(index + 1, *myargv.offset((index + 1) as isize));
        }
    }
}

pub unsafe fn executable_name() -> *const c_char {
    // SAFETY: `myargv[0]` is initialized by C `main`.
    unsafe {
        let argv0 = *myargv;
        let bytes = cstr_bytes(argv0);
        match bytes.iter().rposition(|&b| b == DIR_SEPARATOR) {
            Some(index) => argv0.add(index + 1),
            None => argv0,
        }
    }
}

pub unsafe fn set_exe_dir() {
    // SAFETY: `myargv[0]` is initialized by C `main`; `exedir` is a C-owned
    // global that may be replaced by this function.
    unsafe {
        let bytes = cstr_bytes(*myargv);
        let dirname = match bytes.iter().rposition(|&b| b == DIR_SEPARATOR) {
            Some(index) => &bytes[..index],
            None => b".",
        };

        let mut joined = Vec::with_capacity(dirname.len() + 1);
        joined.extend_from_slice(dirname);
        joined.push(DIR_SEPARATOR);

        exedir = duplicate_bytes(&joined);
    }
}
