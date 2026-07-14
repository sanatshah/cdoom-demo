//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::m_argv::{self, ResponseParseError};
use crate::m_cheat::{self, CheatSeq};
use std::ffi::CStr;
use std::ffi::CString;
use std::mem;
use std::os::raw::{c_char, c_int, c_long, c_ulong, c_void};
use std::ptr;
use std::sync::OnceLock;

static VERSION: OnceLock<CString> = OnceLock::new();

const READ_BINARY: &[u8] = b"rb\0";
const NO_SUCH_RESPONSE_FILE: &[u8] = b"\nNo such response file!\0";
const FOUND_RESPONSE_FILE: &[u8] = b"Found response file %s!\n\0";
const TOO_MANY_PREFIX_ARGS: &[u8] = b"Too many arguments up to the response file!\0";
const TOO_MANY_RESPONSE_ARGS: &[u8] = b"Too many arguments in the response file!\0";
const TOO_MANY_SUFFIX_ARGS: &[u8] = b"Too many arguments following the response file!\0";
const UNCLOSED_QUOTES: &[u8] = b"Quotes unclosed in response file '%s'\0";
const READ_FAILED: &[u8] = b"Failed to read full contents of '%s'\0";
const RESPONSE_PARM: &[u8] = b"-response\0";
const RESPONSE_REPLACEMENT: &[u8] = b"-_\0";

unsafe extern "C" {
    static mut myargc: c_int;
    static mut myargv: *mut *mut c_char;

    fn M_fopen(filename: *const c_char, mode: *const c_char) -> *mut c_void;
    fn M_FileLength(handle: *mut c_void) -> c_long;
    fn I_Error(error: *const c_char, ...) -> !;

    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut c_void) -> c_ulong;
    fn fclose(stream: *mut c_void) -> c_int;
    fn printf(format: *const c_char, ...) -> c_int;
    fn exit(status: c_int) -> !;
}

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
    check: *const c_char,
    num_args: c_int,
) -> c_int {
    if check.is_null() {
        return 0;
    }

    let argc = myargc;
    let limit = argc - num_args;
    let check = CStr::from_ptr(check).to_bytes();
    let mut index = 1;

    while index < limit {
        let arg = *myargv.add(index as usize);

        if arg.is_null() {
            break;
        }

        let arg = CStr::from_ptr(arg).to_bytes();

        if arg.len() == check.len()
            && arg
                .iter()
                .zip(check)
                .all(|(&a, &b)| a.eq_ignore_ascii_case(&b))
        {
            return index;
        }

        index += 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_check_parm(check: *const c_char) -> c_int {
    cdoom_rust_m_check_parm_with_args(check, 0)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_parm_exists(check: *const c_char) -> c_int {
    (cdoom_rust_m_check_parm(check) != 0) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_find_response_file() {
    let mut index = 1;

    while index < myargc {
        let arg = *myargv.add(index as usize);

        if !arg.is_null() && *arg == b'@' as c_char {
            load_response_file(index, arg.add(1));
        }

        index += 1;
    }

    loop {
        let response_index = cdoom_rust_m_check_parm_with_args(RESPONSE_PARM.as_ptr().cast(), 1);

        if response_index <= 0 {
            break;
        }

        let response_slot = myargv.add(response_index as usize);
        free((*response_slot).cast());
        *response_slot = duplicate_bytes(&RESPONSE_REPLACEMENT[..RESPONSE_REPLACEMENT.len() - 1]);

        load_response_file(
            response_index + 1,
            *myargv.add((response_index + 1) as usize),
        );
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

    let count = (*cheat).parameter_chars.max(0) as usize;
    let buffer = std::slice::from_raw_parts_mut(buffer, count);
    m_cheat::get_param(&*cheat, buffer);
}

unsafe fn load_response_file(argv_index: c_int, filename: *const c_char) {
    let handle = M_fopen(filename, READ_BINARY.as_ptr().cast());

    if handle.is_null() {
        printf(NO_SUCH_RESPONSE_FILE.as_ptr().cast());
        exit(1);
    }

    printf(FOUND_RESPONSE_FILE.as_ptr().cast(), filename);

    let size = M_FileLength(handle);
    let size = if size < 0 { 0 } else { size as usize };
    let mut file = vec![0_u8; size + 1];
    let mut bytes_read = 0;

    while bytes_read < size {
        let chunk = fread(
            file.as_mut_ptr().add(bytes_read).cast(),
            1,
            size - bytes_read,
            handle,
        ) as usize;

        if chunk == 0 {
            I_Error(READ_FAILED.as_ptr().cast(), filename);
        }

        bytes_read += chunk;
    }

    fclose(handle);

    let response_args = match m_argv::parse_response_file(&file[..size]) {
        Ok(args) => args,
        Err(ResponseParseError::UnclosedQuote) => {
            I_Error(UNCLOSED_QUOTES.as_ptr().cast(), filename)
        }
        Err(ResponseParseError::TooManyArguments) => {
            I_Error(TOO_MANY_RESPONSE_ARGS.as_ptr().cast())
        }
    };

    let pointer_size = mem::size_of::<*mut c_char>();
    let newargv = malloc(pointer_size * m_argv::MAXARGVS) as *mut *mut c_char;
    ptr::write_bytes(newargv, 0, m_argv::MAXARGVS);
    let mut newargc = 0;

    if argv_index as usize >= m_argv::MAXARGVS {
        I_Error(TOO_MANY_PREFIX_ARGS.as_ptr().cast());
    }

    for old_index in 0..argv_index {
        *newargv.add(old_index as usize) = *myargv.add(old_index as usize);
        *myargv.add(old_index as usize) = ptr::null_mut();
        newargc += 1;
    }

    for arg in response_args {
        if newargc as usize >= m_argv::MAXARGVS {
            I_Error(TOO_MANY_RESPONSE_ARGS.as_ptr().cast());
        }

        *newargv.add(newargc as usize) = duplicate_bytes(&arg);
        newargc += 1;
    }

    if newargc + myargc - (argv_index + 1) >= m_argv::MAXARGVS as c_int {
        I_Error(TOO_MANY_SUFFIX_ARGS.as_ptr().cast());
    }

    for old_index in argv_index + 1..myargc {
        *newargv.add(newargc as usize) = *myargv.add(old_index as usize);
        *myargv.add(old_index as usize) = ptr::null_mut();
        newargc += 1;
    }

    for old_index in 0..myargc {
        let arg = *myargv.add(old_index as usize);

        if !arg.is_null() {
            free(arg.cast());
            *myargv.add(old_index as usize) = ptr::null_mut();
        }
    }

    free(myargv.cast());
    myargv = newargv;
    myargc = newargc;
}

unsafe fn duplicate_bytes(bytes: &[u8]) -> *mut c_char {
    let ptr = malloc(bytes.len() + 1) as *mut c_char;

    if ptr.is_null() {
        I_Error(TOO_MANY_RESPONSE_ARGS.as_ptr().cast());
    }

    ptr::copy_nonoverlapping(bytes.as_ptr().cast(), ptr, bytes.len());
    *ptr.add(bytes.len()) = 0;
    ptr
}
