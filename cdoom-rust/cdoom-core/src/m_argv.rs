//! Behavioral port of the shared `m_argv.c` helpers used during startup.

use std::ffi::CStr;
use std::io::{self, Write};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

const MAXARGVS: usize = 100;

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResponseError {
    TooManyArgumentsUpToResponseFile,
    TooManyArgumentsInResponseFile,
    TooManyArgumentsFollowingResponseFile,
    QuotesUnclosed,
    ReadFailed,
}

fn ascii_lower(ch: c_char) -> c_char {
    if ch >= b'A' as c_char && ch <= b'Z' as c_char {
        ch + 32
    } else {
        ch
    }
}

unsafe fn c_str_eq_ignore_ascii_case(a: *const c_char, b: *const c_char) -> bool {
    let mut index = 0;

    loop {
        let a_ch = unsafe { *a.add(index) };
        let b_ch = unsafe { *b.add(index) };

        if ascii_lower(a_ch) != ascii_lower(b_ch) {
            return false;
        }

        if a_ch == 0 {
            return true;
        }

        index += 1;
    }
}

pub unsafe fn check_parm_with_args(
    argc: c_int,
    argv: *const *const c_char,
    check: *const c_char,
    num_args: c_int,
) -> c_int {
    let limit = argc - num_args;
    let mut i = 1;

    while i < limit {
        let arg = unsafe { *argv.add(i as usize) };

        if arg.is_null() {
            break;
        }

        if unsafe { c_str_eq_ignore_ascii_case(check, arg) } {
            return i;
        }

        i += 1;
    }

    0
}

pub unsafe fn check_parm(argc: c_int, argv: *const *const c_char, check: *const c_char) -> c_int {
    unsafe { check_parm_with_args(argc, argv, check, 0) }
}

pub unsafe fn parm_exists(argc: c_int, argv: *const *const c_char, check: *const c_char) -> bool {
    unsafe { check_parm(argc, argv, check) != 0 }
}

fn is_c_space(ch: u8) -> bool {
    matches!(ch, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

pub fn parse_response_bytes(file: &[u8]) -> Result<Vec<Vec<u8>>, ResponseError> {
    let size = file.len();
    let mut args = Vec::new();
    let mut k = 0;

    while k < size {
        while k < size && is_c_space(file[k]) {
            k += 1;
        }

        if k >= size {
            break;
        }

        if file[k] == b'"' {
            k += 1;
            let argstart = k;

            while k < size && file[k] != b'"' && file[k] != b'\n' {
                k += 1;
            }

            if k >= size || file[k] == b'\n' {
                return Err(ResponseError::QuotesUnclosed);
            }

            if args.len() >= MAXARGVS {
                return Err(ResponseError::TooManyArgumentsInResponseFile);
            }

            args.push(file[argstart..k].to_vec());
            k += 1;
        } else {
            let argstart = k;

            while k < size && !is_c_space(file[k]) {
                k += 1;
            }

            if args.len() >= MAXARGVS {
                return Err(ResponseError::TooManyArgumentsInResponseFile);
            }

            args.push(file[argstart..k].to_vec());
            k += usize::from(k < size);
        }
    }

    Ok(args)
}

unsafe fn duplicate_bytes(bytes: &[u8]) -> *mut c_char {
    let result = unsafe { malloc(bytes.len() + 1) } as *mut u8;

    if result.is_null() {
        std::process::abort();
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result, bytes.len());
        *result.add(bytes.len()) = 0;
    }

    result as *mut c_char
}

unsafe fn report_response_error(error: ResponseError, filename: *const c_char) -> ! {
    let filename = unsafe { CStr::from_ptr(filename) }.to_string_lossy();

    match error {
        ResponseError::TooManyArgumentsUpToResponseFile => {
            eprintln!("Too many arguments up to the response file!");
        }
        ResponseError::TooManyArgumentsInResponseFile => {
            eprintln!("Too many arguments in the response file!");
        }
        ResponseError::TooManyArgumentsFollowingResponseFile => {
            eprintln!("Too many arguments following the response file!");
        }
        ResponseError::QuotesUnclosed => {
            eprintln!("Quotes unclosed in response file '{filename}'");
        }
        ResponseError::ReadFailed => {
            eprintln!("Failed to read full contents of '{filename}'");
        }
    }

    std::process::exit(1);
}

fn read_response_file(filename: *const c_char) -> Result<Vec<u8>, ResponseError> {
    let c_filename = unsafe { CStr::from_ptr(filename) };

    #[cfg(unix)]
    let path = std::path::Path::new(std::ffi::OsStr::from_bytes(c_filename.to_bytes()));

    #[cfg(not(unix))]
    let path = std::path::Path::new(&c_filename.to_string_lossy());

    std::fs::read(path).map_err(|_| ResponseError::ReadFailed)
}

unsafe fn load_response_file(
    argc_ptr: *mut c_int,
    argv_ptr: *mut *mut *mut c_char,
    argv_index: c_int,
    filename: *const c_char,
) {
    let file = match read_response_file(filename) {
        Ok(file) => file,
        Err(ResponseError::ReadFailed) => {
            print!("\nNo such response file!");
            let _ = io::stdout().flush();
            std::process::exit(1);
        }
        Err(error) => unsafe { report_response_error(error, filename) },
    };

    println!(
        "Found response file {}!",
        unsafe { CStr::from_ptr(filename) }.to_string_lossy()
    );

    let response_args = match parse_response_bytes(&file) {
        Ok(args) => args,
        Err(error) => unsafe { report_response_error(error, filename) },
    };

    let old_argc = unsafe { *argc_ptr };
    let old_argv = unsafe { *argv_ptr };

    let newargv =
        unsafe { malloc(std::mem::size_of::<*mut c_char>() * MAXARGVS) } as *mut *mut c_char;

    if newargv.is_null() {
        std::process::abort();
    }

    for i in 0..MAXARGVS {
        unsafe {
            *newargv.add(i) = ptr::null_mut();
        }
    }

    let mut newargc: c_int = 0;

    if argv_index as usize >= MAXARGVS {
        unsafe { report_response_error(ResponseError::TooManyArgumentsUpToResponseFile, filename) };
    }

    for i in 0..argv_index {
        unsafe {
            *newargv.add(i as usize) = *old_argv.add(i as usize);
            *old_argv.add(i as usize) = ptr::null_mut();
        }
        newargc += 1;
    }

    for arg in &response_args {
        if newargc as usize >= MAXARGVS {
            unsafe {
                report_response_error(ResponseError::TooManyArgumentsInResponseFile, filename)
            };
        }

        unsafe {
            *newargv.add(newargc as usize) = duplicate_bytes(arg);
        }
        newargc += 1;
    }

    if newargc + old_argc - (argv_index + 1) >= MAXARGVS as c_int {
        unsafe {
            report_response_error(
                ResponseError::TooManyArgumentsFollowingResponseFile,
                filename,
            )
        };
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
                free(arg.cast());
                *old_argv.add(i as usize) = ptr::null_mut();
            }
        }
    }

    unsafe {
        free(old_argv.cast());
        *argv_ptr = newargv;
        *argc_ptr = newargc;
    }
}

pub unsafe fn find_response_file(argc_ptr: *mut c_int, argv_ptr: *mut *mut *mut c_char) {
    let mut i = 1;

    while i < unsafe { *argc_ptr } {
        let argv = unsafe { *argv_ptr };
        let arg = unsafe { *argv.add(i as usize) };

        if !arg.is_null() && unsafe { *arg } == b'@' as c_char {
            unsafe { load_response_file(argc_ptr, argv_ptr, i, arg.add(1)) };
        }

        i += 1;
    }

    loop {
        let argv = unsafe { *argv_ptr } as *const *const c_char;
        let i = unsafe { check_parm_with_args(*argc_ptr, argv, c"-response".as_ptr(), 1) };

        if i <= 0 {
            break;
        }

        let argv = unsafe { *argv_ptr };

        unsafe {
            free((*argv.add(i as usize)).cast());
            *argv.add(i as usize) = duplicate_bytes(b"-_");
            load_response_file(argc_ptr, argv_ptr, i + 1, *argv.add((i + 1) as usize));
        }
    }
}

fn last_separator(bytes: &[u8]) -> Option<usize> {
    let slash = bytes.iter().rposition(|&ch| ch == b'/');

    #[cfg(windows)]
    {
        let backslash = bytes.iter().rposition(|&ch| ch == b'\\');
        return slash.max(backslash);
    }

    #[cfg(not(windows))]
    {
        slash
    }
}

pub unsafe fn get_executable_name(path: *const c_char) -> *const c_char {
    let bytes = unsafe { CStr::from_ptr(path) }.to_bytes();

    match last_separator(bytes) {
        Some(index) => unsafe { path.add(index + 1) },
        None => path,
    }
}

pub unsafe fn set_exe_dir(path: *const c_char, exedir: *mut *mut c_char) {
    let bytes = unsafe { CStr::from_ptr(path) }.to_bytes();
    let mut dirname = match last_separator(bytes) {
        Some(index) => bytes[..index].to_vec(),
        None => b".".to_vec(),
    };

    #[cfg(windows)]
    dirname.push(b'\\');

    #[cfg(not(windows))]
    dirname.push(b'/');

    unsafe {
        *exedir = duplicate_bytes(&dirname);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_response_file_like_c() {
        let args = parse_response_bytes(b"  -iwad \"long path.wad\"\n-skill\t1\r\n").unwrap();

        assert_eq!(
            args,
            vec![
                b"-iwad".to_vec(),
                b"long path.wad".to_vec(),
                b"-skill".to_vec(),
                b"1".to_vec(),
            ]
        );
    }

    #[test]
    fn rejects_unclosed_quoted_argument() {
        assert_eq!(
            parse_response_bytes(b"\"unterminated\n").unwrap_err(),
            ResponseError::QuotesUnclosed
        );
    }
}
