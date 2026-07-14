use std::ffi::CStr;
use std::fs::File;
use std::io::Read;
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
use std::ptr;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

const MAXARGVS: usize = 100;

unsafe extern "C" {
    fn free(ptr: *mut c_void);
    fn malloc(size: usize) -> *mut c_void;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResponseError {
    MissingFile(Vec<u8>),
    ReadFailed(Vec<u8>),
    TooManyArguments(&'static str),
    UnclosedQuote(Vec<u8>),
    InteriorNul,
    AllocationFailed,
}

pub fn check_parm(argv: &[Option<&[u8]>], check: &[u8], num_args: c_int) -> c_int {
    let argc = argv.len() as c_int;
    let mut i = 1;

    while i < argc - num_args {
        let Some(arg) = argv[i as usize] else {
            break;
        };

        if ascii_case_eq(check, arg) {
            return i;
        }

        i += 1;
    }

    0
}

pub fn parse_response_bytes(bytes: &[u8], filename: &[u8]) -> Result<Vec<Vec<u8>>, ResponseError> {
    let mut result = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        while index < bytes.len() && is_c_space(bytes[index]) {
            index += 1;
        }

        if index >= bytes.len() {
            break;
        }

        let arg = if bytes[index] == b'"' {
            index += 1;
            let start = index;

            while index < bytes.len() && bytes[index] != b'"' && bytes[index] != b'\n' {
                index += 1;
            }

            if index >= bytes.len() || bytes[index] == b'\n' {
                return Err(ResponseError::UnclosedQuote(filename.to_vec()));
            }

            let arg = bytes[start..index].to_vec();
            index += 1;
            arg
        } else {
            let start = index;

            while index < bytes.len() && !is_c_space(bytes[index]) {
                index += 1;
            }

            bytes[start..index].to_vec()
        };

        if result.len() >= MAXARGVS {
            return Err(ResponseError::TooManyArguments(
                "Too many arguments in the response file!",
            ));
        }

        result.push(arg);
    }

    Ok(result)
}

pub fn expand_response_files<F>(
    mut args: Vec<Vec<u8>>,
    mut load_file: F,
) -> Result<Vec<Vec<u8>>, ResponseError>
where
    F: FnMut(&[u8]) -> Result<Vec<u8>, ResponseError>,
{
    let mut i = 1;

    while i < args.len() {
        if args[i].first() == Some(&b'@') {
            let filename = args[i][1..].to_vec();
            let contents = load_file(&filename)?;
            let tokens = parse_response_bytes(&contents, &filename)?;
            args = replace_response_argument(args, i, tokens)?;
        }

        i += 1;
    }

    loop {
        let view = args
            .iter()
            .map(|arg| Some(arg.as_slice()))
            .collect::<Vec<_>>();
        let i = check_parm(&view, b"-response", 1);

        if i <= 0 {
            break;
        }

        let response_index = i as usize;
        args[response_index] = b"-_".to_vec();
        let filename = args[response_index + 1].clone();
        let contents = load_file(&filename)?;
        let tokens = parse_response_bytes(&contents, &filename)?;
        args = replace_response_argument(args, response_index + 1, tokens)?;
    }

    Ok(args)
}

pub fn basename_offset(path: &[u8]) -> usize {
    path.iter()
        .rposition(|&ch| ch == b'/' || cfg!(windows) && ch == b'\\')
        .map(|index| index + 1)
        .unwrap_or(0)
}

pub fn exe_dir_bytes(path: &[u8]) -> Vec<u8> {
    let dirname = path
        .iter()
        .rposition(|&ch| ch == b'/' || cfg!(windows) && ch == b'\\')
        .map(|index| path[..index].to_vec())
        .unwrap_or_else(|| b".".to_vec());

    let mut result = dirname;
    result.push(if cfg!(windows) { b'\\' } else { b'/' });
    result
}

pub unsafe fn check_parm_with_c_args(
    argc: c_int,
    argv: *mut *mut c_char,
    check: *const c_char,
    num_args: c_int,
) -> c_int {
    if argc <= 0 || argv.is_null() || check.is_null() {
        return 0;
    }

    let check = CStr::from_ptr(check).to_bytes();
    let mut args = Vec::with_capacity(argc as usize);

    for i in 0..argc as usize {
        let arg = *argv.add(i);
        if arg.is_null() {
            args.push(None);
        } else {
            args.push(Some(CStr::from_ptr(arg).to_bytes()));
        }
    }

    check_parm(&args, check, num_args)
}

pub unsafe fn find_response_file_c(
    argc: *mut c_int,
    argv: *mut *mut *mut c_char,
) -> Result<(), ResponseError> {
    if argc.is_null() || argv.is_null() || (*argv).is_null() || *argc < 0 {
        return Err(ResponseError::AllocationFailed);
    }

    let current_argc = *argc as usize;
    let current_argv = *argv;
    let mut args = Vec::with_capacity(current_argc);

    for i in 0..current_argc {
        let arg = *current_argv.add(i);
        if arg.is_null() {
            args.push(Vec::new());
        } else {
            args.push(CStr::from_ptr(arg).to_bytes().to_vec());
        }
    }

    let expanded = expand_response_files(args, read_response_file)?;
    let new_argv = allocate_argv(&expanded)?;

    for i in 0..current_argc {
        let arg = *current_argv.add(i);
        if !arg.is_null() {
            free(arg.cast::<c_void>());
        }
    }

    free(current_argv.cast::<c_void>());
    *argv = new_argv;
    *argc = expanded.len() as c_int;

    Ok(())
}

pub unsafe fn duplicate_error_message(error: &ResponseError) -> *mut c_char {
    let message = match error {
        ResponseError::MissingFile(_) => "No such response file!".to_string(),
        ResponseError::ReadFailed(filename) => {
            format!(
                "Failed to read full contents of '{}'",
                String::from_utf8_lossy(filename)
            )
        }
        ResponseError::TooManyArguments(message) => (*message).to_string(),
        ResponseError::UnclosedQuote(filename) => {
            format!(
                "Quotes unclosed in response file '{}'",
                String::from_utf8_lossy(filename)
            )
        }
        ResponseError::InteriorNul => "Argument contains an interior NUL byte.".to_string(),
        ResponseError::AllocationFailed => "Failed to allocate argv storage.".to_string(),
    };

    duplicate_bytes(message.as_bytes()).unwrap_or(ptr::null_mut())
}

pub unsafe fn duplicate_exe_dir(argv0: *const c_char) -> Result<*mut c_char, ResponseError> {
    if argv0.is_null() {
        return duplicate_bytes(b"./");
    }

    duplicate_bytes(&exe_dir_bytes(CStr::from_ptr(argv0).to_bytes()))
}

pub unsafe fn executable_name(argv0: *const c_char) -> *const c_char {
    if argv0.is_null() {
        return ptr::null();
    }

    let bytes = CStr::from_ptr(argv0).to_bytes();
    argv0.add(basename_offset(bytes))
}

fn ascii_case_eq(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

fn is_c_space(value: u8) -> bool {
    matches!(value, b' ' | b'\x0c' | b'\n' | b'\r' | b'\t' | b'\x0b')
}

fn replace_response_argument(
    args: Vec<Vec<u8>>,
    argv_index: usize,
    tokens: Vec<Vec<u8>>,
) -> Result<Vec<Vec<u8>>, ResponseError> {
    if argv_index >= MAXARGVS {
        return Err(ResponseError::TooManyArguments(
            "Too many arguments up to the response file!",
        ));
    }

    let mut result = Vec::with_capacity(MAXARGVS);

    for arg in args.iter().take(argv_index) {
        result.push(arg.clone());
    }

    for token in tokens {
        if result.len() >= MAXARGVS {
            return Err(ResponseError::TooManyArguments(
                "Too many arguments in the response file!",
            ));
        }

        result.push(token);
    }

    if result.len() + args.len() - (argv_index + 1) >= MAXARGVS {
        return Err(ResponseError::TooManyArguments(
            "Too many arguments following the response file!",
        ));
    }

    for arg in args.into_iter().skip(argv_index + 1) {
        result.push(arg);
    }

    Ok(result)
}

fn read_response_file(filename: &[u8]) -> Result<Vec<u8>, ResponseError> {
    let path = path_from_c_bytes(filename);
    let mut file = File::open(&path).map_err(|_| ResponseError::MissingFile(filename.to_vec()))?;
    let mut contents = Vec::new();

    file.read_to_end(&mut contents)
        .map_err(|_| ResponseError::ReadFailed(filename.to_vec()))?;

    println!("Found response file {}!", String::from_utf8_lossy(filename));

    Ok(contents)
}

fn path_from_c_bytes(bytes: &[u8]) -> PathBuf {
    #[cfg(unix)]
    {
        std::ffi::OsStr::from_bytes(bytes).into()
    }

    #[cfg(not(unix))]
    {
        String::from_utf8_lossy(bytes).into_owned().into()
    }
}

unsafe fn allocate_argv(args: &[Vec<u8>]) -> Result<*mut *mut c_char, ResponseError> {
    let capacity = MAXARGVS.max(args.len());
    let size = capacity * std::mem::size_of::<*mut c_char>();
    let argv = malloc(size).cast::<*mut c_char>();

    if argv.is_null() {
        return Err(ResponseError::AllocationFailed);
    }

    ptr::write_bytes(argv, 0, capacity);

    for (index, arg) in args.iter().enumerate() {
        *argv.add(index) = match duplicate_bytes(arg) {
            Ok(ptr) => ptr,
            Err(error) => {
                for cleanup in 0..index {
                    let ptr = *argv.add(cleanup);
                    if !ptr.is_null() {
                        free(ptr.cast::<c_void>());
                    }
                }

                free(argv.cast::<c_void>());
                return Err(error);
            }
        };
    }

    Ok(argv)
}

unsafe fn duplicate_bytes(bytes: &[u8]) -> Result<*mut c_char, ResponseError> {
    if bytes.contains(&0) {
        return Err(ResponseError::InteriorNul);
    }

    let ptr = malloc(bytes.len() + 1).cast::<u8>();

    if ptr.is_null() {
        return Err(ResponseError::AllocationFailed);
    }

    ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
    *ptr.add(bytes.len()) = 0;

    Ok(ptr.cast::<c_char>())
}
