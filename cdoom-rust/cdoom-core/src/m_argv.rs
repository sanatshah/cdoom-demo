//! Behavioral port of Chocolate Doom's shared `m_argv` helpers.

use std::ffi::CStr;
#[cfg(unix)]
use std::ffi::OsStr;
use std::mem;
use std::os::raw::{c_char, c_int, c_void};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::path::Path;
use std::ptr;

const DIR_SEPARATOR: u8 = b'/';
const MAXARGVS: usize = 100;

unsafe extern "C" {
    fn exit(status: c_int) -> !;
    fn free(ptr: *mut c_void);
    fn malloc(size: usize) -> *mut c_void;
}

#[no_mangle]
pub static mut myargc: c_int = 0;

#[no_mangle]
pub static mut myargv: *mut *mut c_char = ptr::null_mut();

#[no_mangle]
pub static mut exedir: *mut c_char = ptr::null_mut();

unsafe fn fatal(message: &str) -> ! {
    eprintln!("{message}");
    exit(1);
}

unsafe fn c_arg(index: c_int) -> *mut c_char {
    *myargv.add(index as usize)
}

unsafe fn set_c_arg(index: c_int, value: *mut c_char) {
    *myargv.add(index as usize) = value;
}

unsafe fn c_bytes(ptr: *const c_char) -> &'static [u8] {
    CStr::from_ptr(ptr).to_bytes()
}

fn eq_ignore_ascii_case(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(&a, &b)| a.eq_ignore_ascii_case(&b))
}

unsafe fn malloc_bytes(len: usize) -> *mut u8 {
    let ptr = malloc(len).cast::<u8>();

    if ptr.is_null() {
        fatal("Failed to allocate memory in m_argv");
    }

    ptr
}

unsafe fn duplicate_bytes(bytes: &[u8]) -> *mut c_char {
    let dest = malloc_bytes(bytes.len() + 1);

    if !bytes.is_empty() {
        ptr::copy_nonoverlapping(bytes.as_ptr(), dest, bytes.len());
    }

    *dest.add(bytes.len()) = 0;
    dest.cast::<c_char>()
}

unsafe fn malloc_argv() -> *mut *mut c_char {
    let bytes = MAXARGVS * mem::size_of::<*mut c_char>();
    let argv = malloc_bytes(bytes).cast::<*mut c_char>();

    ptr::write_bytes(argv, 0, MAXARGVS);

    argv
}

fn read_response_file(filename: *const c_char) -> Result<Vec<u8>, std::io::Error> {
    let filename_bytes = unsafe { c_bytes(filename) };

    #[cfg(unix)]
    {
        std::fs::read(Path::new(OsStr::from_bytes(filename_bytes)))
    }

    #[cfg(not(unix))]
    {
        std::fs::read(String::from_utf8_lossy(filename_bytes).as_ref())
    }
}

unsafe fn response_filename_for_display(filename: *const c_char) -> String {
    String::from_utf8_lossy(c_bytes(filename)).into_owned()
}

unsafe fn load_response_file(argv_index: c_int, filename: *const c_char) {
    let mut file = match read_response_file(filename) {
        Ok(file) => file,
        Err(_) => {
            print!("\nNo such response file!");
            exit(1);
        }
    };

    println!(
        "Found response file {}!",
        response_filename_for_display(filename)
    );

    let size = file.len();
    file.push(0);

    let newargv = malloc_argv();
    let mut newargc: c_int = 0;

    if argv_index as usize >= MAXARGVS {
        fatal("Too many arguments up to the response file!");
    }

    for i in 0..argv_index {
        *newargv.add(i as usize) = c_arg(i);
        set_c_arg(i, ptr::null_mut());
        newargc += 1;
    }

    let mut k = 0usize;

    while k < size {
        while k < size && file[k].is_ascii_whitespace() {
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
                fatal(&format!(
                    "Quotes unclosed in response file '{}'",
                    response_filename_for_display(filename)
                ));
            }

            if newargc as usize >= MAXARGVS {
                fatal("Too many arguments in the response file!");
            }

            *newargv.add(newargc as usize) = duplicate_bytes(&file[argstart..k]);
            newargc += 1;
            k += 1;
        } else {
            let argstart = k;

            while k < size && !file[k].is_ascii_whitespace() {
                k += 1;
            }

            if newargc as usize >= MAXARGVS {
                fatal("Too many arguments in the response file!");
            }

            *newargv.add(newargc as usize) = duplicate_bytes(&file[argstart..k]);
            newargc += 1;
            k += 1;
        }
    }

    if newargc + myargc - (argv_index + 1) >= MAXARGVS as c_int {
        fatal("Too many arguments following the response file!");
    }

    for i in (argv_index + 1)..myargc {
        *newargv.add(newargc as usize) = c_arg(i);
        set_c_arg(i, ptr::null_mut());
        newargc += 1;
    }

    for i in 0..myargc {
        let arg = c_arg(i);

        if !arg.is_null() {
            free(arg.cast::<c_void>());
            set_c_arg(i, ptr::null_mut());
        }
    }

    free(myargv.cast::<c_void>());
    myargv = newargv;
    myargc = newargc;
}

pub unsafe fn check_parm_with_args(check: *const c_char, num_args: c_int) -> c_int {
    if check.is_null() || myargv.is_null() {
        return 0;
    }

    let check_bytes = c_bytes(check);
    let mut i = 1;

    while i < myargc - num_args {
        let arg = c_arg(i);

        if arg.is_null() {
            break;
        }

        if eq_ignore_ascii_case(check_bytes, c_bytes(arg)) {
            return i;
        }

        i += 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_check_parm_with_args(
    check: *const c_char,
    num_args: c_int,
) -> c_int {
    check_parm_with_args(check, num_args)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_check_parm(check: *const c_char) -> c_int {
    check_parm_with_args(check, 0)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_parm_exists(check: *const c_char) -> c_int {
    (check_parm_with_args(check, 0) != 0) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_find_response_file() {
    let mut i = 1;

    while i < myargc {
        let arg = c_arg(i);

        if !arg.is_null() && *arg.cast::<u8>() == b'@' {
            load_response_file(i, arg.add(1));
        }

        i += 1;
    }

    loop {
        let response = b"-response\0";
        let i = check_parm_with_args(response.as_ptr().cast::<c_char>(), 1);

        if i <= 0 {
            break;
        }

        free(c_arg(i).cast::<c_void>());
        set_c_arg(i, duplicate_bytes(b"-_"));
        load_response_file(i + 1, c_arg(i + 1));
    }
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_get_executable_name() -> *const c_char {
    if myargv.is_null() || myargc <= 0 || c_arg(0).is_null() {
        return ptr::null();
    }

    let path = c_bytes(c_arg(0));
    let basename = path
        .iter()
        .rposition(|&ch| ch == DIR_SEPARATOR)
        .map_or(0, |index| index + 1);

    c_arg(0).add(basename).cast::<c_char>()
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_set_exe_dir() {
    if myargv.is_null() || myargc <= 0 || c_arg(0).is_null() {
        exedir = duplicate_bytes(b"./");
        return;
    }

    let path = c_bytes(c_arg(0));
    let dirname_end = path.iter().rposition(|&ch| ch == DIR_SEPARATOR);
    let mut dirname = Vec::new();

    match dirname_end {
        Some(index) => dirname.extend_from_slice(&path[..index]),
        None => dirname.extend_from_slice(b"."),
    }

    dirname.push(DIR_SEPARATOR);
    exedir = duplicate_bytes(&dirname);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn basename_points_inside_executable_path() {
        let arg0 = CString::new("/usr/local/bin/chocolate-doom").unwrap();
        let args = [arg0.as_ptr().cast_mut()];
        let old_argc = unsafe { myargc };
        let old_argv = unsafe { myargv };

        unsafe {
            myargc = args.len() as c_int;
            myargv = args.as_ptr().cast_mut();
            assert_eq!(
                CStr::from_ptr(cdoom_rust_m_get_executable_name())
                    .to_str()
                    .unwrap(),
                "chocolate-doom"
            );
            myargc = old_argc;
            myargv = old_argv;
        }
    }
}
