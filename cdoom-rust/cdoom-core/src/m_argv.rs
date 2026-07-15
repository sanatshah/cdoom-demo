//! Command-line argument helpers ported from `m_argv.c`.

use std::mem;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

const MAXARGVS: usize = 100;

#[repr(C)]
struct CFile {
    _private: [u8; 0],
}

#[derive(Debug, Eq, PartialEq)]
pub enum ResponseParseError {
    UnclosedQuotes,
    TooManyArguments,
}

unsafe extern "C" {
    fn fclose(stream: *mut CFile) -> c_int;
    fn fopen(filename: *const c_char, mode: *const c_char) -> *mut CFile;
    fn fseek(stream: *mut CFile, offset: i64, whence: c_int) -> c_int;
    fn ftell(stream: *mut CFile) -> i64;
    fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut CFile) -> usize;
    fn free(ptr: *mut c_void);
    fn malloc(size: usize) -> *mut c_void;
    fn printf(format: *const c_char, ...) -> c_int;
    fn strcasecmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

const SEEK_SET: c_int = 0;
const SEEK_END: c_int = 2;

fn is_response_space(byte: u8) -> bool {
    byte.is_ascii_whitespace()
}

/// Parses response-file bytes using the same quote/whitespace rules as C.
pub fn parse_response_args(bytes: &[u8]) -> Result<Vec<Vec<u8>>, ResponseParseError> {
    let mut args = Vec::new();
    let mut k = 0;

    while k < bytes.len() {
        while k < bytes.len() && is_response_space(bytes[k]) {
            k += 1;
        }

        if k >= bytes.len() {
            break;
        }

        let arg = if bytes[k] == b'"' {
            k += 1;
            let arg_start = k;

            while k < bytes.len() && bytes[k] != b'"' && bytes[k] != b'\n' {
                k += 1;
            }

            if k >= bytes.len() || bytes[k] == b'\n' {
                return Err(ResponseParseError::UnclosedQuotes);
            }

            let arg = bytes[arg_start..k].to_vec();
            k += 1;
            arg
        } else {
            let arg_start = k;

            while k < bytes.len() && !is_response_space(bytes[k]) {
                k += 1;
            }

            bytes[arg_start..k].to_vec()
        };

        if args.len() >= MAXARGVS {
            return Err(ResponseParseError::TooManyArguments);
        }

        args.push(arg);
    }

    Ok(args)
}

/// Checks for a command-line parameter with the requested trailing arg count.
///
/// # Safety
///
/// `argv` must point to an array of `argc` C string pointers.
pub unsafe fn check_parm_with_args(
    argc: c_int,
    argv: *mut *mut c_char,
    check: *const c_char,
    num_args: c_int,
) -> c_int {
    let mut i = 1;

    while i < argc - num_args {
        let arg = unsafe { *argv.add(i as usize) };

        if arg.is_null() {
            break;
        }

        if unsafe { strcasecmp(check, arg) } == 0 {
            return i;
        }

        i += 1;
    }

    0
}

unsafe fn argv_ptr(argv: *mut *mut *mut c_char) -> *mut *mut c_char {
    unsafe { *argv }
}

unsafe fn argv_at(argv: *mut *mut *mut c_char, i: c_int) -> *mut c_char {
    unsafe { *argv_ptr(argv).add(i as usize) }
}

unsafe fn set_argv_at(argv: *mut *mut *mut c_char, i: c_int, value: *mut c_char) {
    unsafe {
        *argv_ptr(argv).add(i as usize) = value;
    }
}

fn parse_or_error(bytes: &[u8], filename: *const c_char) -> Vec<Vec<u8>> {
    match parse_response_args(bytes) {
        Ok(args) => args,
        Err(ResponseParseError::UnclosedQuotes) => fatal_error(&format!(
            "Quotes unclosed in response file '{}'",
            unsafe { std::ffi::CStr::from_ptr(filename) }.to_string_lossy()
        )),
        Err(ResponseParseError::TooManyArguments) => {
            fatal_error("Too many arguments in the response file!")
        }
    }
}

fn fatal_error(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1);
}

unsafe fn duplicate_arg(arg: &[u8]) -> *mut c_char {
    let copy = unsafe { malloc(arg.len() + 1) as *mut c_char };

    if copy.is_null() {
        fatal_error("Failed to allocate response file argument");
    }

    unsafe {
        ptr::copy_nonoverlapping(arg.as_ptr(), copy.cast::<u8>(), arg.len());
        *copy.add(arg.len()) = 0;
    }

    copy
}

unsafe fn duplicate_cstr(value: *const c_char) -> *mut c_char {
    let bytes = unsafe { std::ffi::CStr::from_ptr(value) }.to_bytes();

    unsafe { duplicate_arg(bytes) }
}

unsafe fn file_length(handle: *mut CFile) -> c_int {
    if unsafe { fseek(handle, 0, SEEK_END) } != 0 {
        fatal_error("Failed to seek response file");
    }

    let length = unsafe { ftell(handle) };

    if length < 0 {
        fatal_error("Failed to determine response file length");
    }

    if unsafe { fseek(handle, 0, SEEK_SET) } != 0 {
        fatal_error("Failed to rewind response file");
    }

    length as c_int
}

unsafe fn load_response_file(
    argv_index: c_int,
    filename: *const c_char,
    argc: *mut c_int,
    argv: *mut *mut *mut c_char,
) {
    let handle = unsafe { fopen(filename, c"rb".as_ptr()) };

    if handle.is_null() {
        unsafe {
            printf(c"\nNo such response file!".as_ptr());
            std::process::exit(1);
        }
    }

    unsafe {
        printf(c"Found response file %s!\n".as_ptr(), filename);
    }

    let size = unsafe { file_length(handle) };
    let file = unsafe { malloc(size as usize + 1) as *mut u8 };

    if file.is_null() {
        fatal_error("Failed to allocate response file buffer");
    }

    let mut offset = 0;
    while offset < size as usize {
        let count = unsafe {
            fread(
                file.add(offset).cast::<c_void>(),
                1,
                size as usize - offset,
                handle,
            )
        };

        if count == 0 {
            fatal_error(&format!(
                "Failed to read full contents of '{}'",
                unsafe { std::ffi::CStr::from_ptr(filename) }.to_string_lossy()
            ));
        }

        offset += count;
    }

    unsafe {
        fclose(handle);
    }

    let response_args = parse_or_error(
        unsafe { std::slice::from_raw_parts(file, size as usize) },
        filename,
    );
    let old_argc = unsafe { *argc };
    let old_argv = unsafe { argv_ptr(argv) };
    let newargv = unsafe { malloc(mem::size_of::<*mut c_char>() * MAXARGVS) as *mut *mut c_char };

    if newargv.is_null() {
        fatal_error("Failed to allocate response argv");
    }

    unsafe {
        ptr::write_bytes(newargv, 0, MAXARGVS);
    }

    if argv_index >= MAXARGVS as c_int {
        fatal_error("Too many arguments up to the response file!");
    }

    let mut newargc = 0;
    for i in 0..argv_index {
        unsafe {
            *newargv.add(i as usize) = *old_argv.add(i as usize);
            *old_argv.add(i as usize) = ptr::null_mut();
        }
        newargc += 1;
    }

    for arg in &response_args {
        if newargc >= MAXARGVS as c_int {
            fatal_error("Too many arguments in the response file!");
        }

        unsafe {
            *newargv.add(newargc as usize) = duplicate_arg(arg);
        }
        newargc += 1;
    }

    if newargc + old_argc - (argv_index + 1) >= MAXARGVS as c_int {
        fatal_error("Too many arguments following the response file!");
    }

    for i in (argv_index + 1)..old_argc {
        unsafe {
            *newargv.add(newargc as usize) = *old_argv.add(i as usize);
            *old_argv.add(i as usize) = ptr::null_mut();
        }
        newargc += 1;
    }

    for i in 0..old_argc {
        let arg = unsafe { *old_argv.add(i as usize) };

        if !arg.is_null() {
            unsafe {
                free(arg.cast::<c_void>());
                *old_argv.add(i as usize) = ptr::null_mut();
            }
        }
    }

    unsafe {
        free(old_argv.cast::<c_void>());
        *argv = newargv;
        *argc = newargc;
        free(file.cast::<c_void>());
    }
}

/// Expands `@file` and `-response file` entries in-place.
///
/// # Safety
///
/// `argc` and `argv` must point to Chocolate Doom's global `myargc` and
/// `myargv` variables. The argv storage must use C `malloc`/`free` ownership.
pub unsafe fn find_response_file(argc: *mut c_int, argv: *mut *mut *mut c_char) {
    let mut i = 1;

    while i < unsafe { *argc } {
        let arg = unsafe { argv_at(argv, i) };

        if !arg.is_null() && unsafe { *arg } == b'@' as c_char {
            unsafe { load_response_file(i, arg.add(1), argc, argv) };
        }

        i += 1;
    }

    loop {
        let response_arg =
            unsafe { check_parm_with_args(*argc, argv_ptr(argv), c"-response".as_ptr(), 1) };

        if response_arg <= 0 {
            break;
        }

        unsafe {
            free(argv_at(argv, response_arg).cast::<c_void>());
            set_argv_at(argv, response_arg, duplicate_cstr(c"-_".as_ptr()));
            load_response_file(
                response_arg + 1,
                argv_at(argv, response_arg + 1),
                argc,
                argv,
            );
        }
    }
}

/// Returns the basename of argv[0].
///
/// # Safety
///
/// `argv` must point to an argv array with a valid argv[0].
pub unsafe fn get_executable_name(argv: *mut *mut c_char) -> *const c_char {
    let path = unsafe { *argv };
    let bytes = unsafe { std::ffi::CStr::from_ptr(path) }.to_bytes();
    let mut basename = 0;

    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'/' || cfg!(windows) && *byte == b'\\' {
            basename = index + 1;
        }
    }

    unsafe { path.add(basename) }
}

/// Returns the executable directory string allocated by Chocolate Doom helpers.
///
/// # Safety
///
/// `argv` must point to an argv array with a valid argv[0].
pub unsafe fn set_exe_dir(argv: *mut *mut c_char) -> *mut c_char {
    #[cfg(windows)]
    let separator = b'\\';
    #[cfg(not(windows))]
    let separator = b'/';

    let path = unsafe { *argv };
    let bytes = unsafe { std::ffi::CStr::from_ptr(path) }.to_bytes();
    let dirname_len = bytes
        .iter()
        .enumerate()
        .filter_map(|(index, byte)| {
            if *byte == b'/' || cfg!(windows) && *byte == b'\\' {
                Some(index)
            } else {
                None
            }
        })
        .last();

    let mut result = match dirname_len {
        Some(len) => bytes[..len].to_vec(),
        None => b".".to_vec(),
    };
    result.push(separator);

    unsafe { duplicate_arg(&result) }
}
