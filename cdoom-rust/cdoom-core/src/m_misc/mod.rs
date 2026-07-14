//! Rust implementations for selected `m_misc` helpers.
//!
//! These functions intentionally mirror the C utility semantics instead of
//! exposing a more idiomatic Rust API; callers are still C code.

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;

const DIR_SEPARATOR: u8 = b'/';

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

unsafe fn bytes_from_cstr<'a>(ptr: *const c_char) -> &'a [u8] {
    if ptr.is_null() {
        b""
    } else {
        CStr::from_ptr(ptr).to_bytes()
    }
}

unsafe fn mut_bytes_from_cstr<'a>(ptr: *mut c_char) -> &'a mut [u8] {
    let len = CStr::from_ptr(ptr).to_bytes().len();
    slice::from_raw_parts_mut(ptr.cast::<u8>(), len)
}

fn alloc_bytes(bytes: &[u8]) -> *mut c_char {
    unsafe {
        let result = malloc(bytes.len() + 1).cast::<u8>();
        if result.is_null() {
            return ptr::null_mut();
        }

        ptr::copy_nonoverlapping(bytes.as_ptr(), result, bytes.len());
        *result.add(bytes.len()) = 0;
        result.cast::<c_char>()
    }
}

fn parse_digits(bytes: &[u8], radix: u32, allow_sign: bool) -> Option<i32> {
    let mut pos = 0;
    let mut sign = 1i64;

    if allow_sign && pos < bytes.len() {
        if bytes[pos] == b'-' {
            sign = -1;
            pos += 1;
        } else if bytes[pos] == b'+' {
            pos += 1;
        }
    }

    let start = pos;
    let mut value = 0i64;

    while pos < bytes.len() {
        match (bytes[pos] as char).to_digit(radix) {
            Some(digit) => {
                value = value.wrapping_mul(radix as i64).wrapping_add(digit as i64);
                pos += 1;
            }
            None => break,
        }
    }

    if pos == start {
        None
    } else {
        Some(value.wrapping_mul(sign) as i32)
    }
}

pub unsafe fn str_to_int(str_ptr: *const c_char, result: *mut c_int) -> c_int {
    if result.is_null() {
        return 0;
    }

    let bytes = bytes_from_cstr(str_ptr);
    let trimmed = bytes
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .map_or(&[][..], |pos| &bytes[pos..]);

    let parsed = if trimmed.starts_with(b"0x") || trimmed.starts_with(b"0X") {
        parse_digits(&trimmed[2..], 16, false)
    } else if trimmed.starts_with(b"0") {
        if trimmed.len() == 1 {
            Some(0)
        } else {
            parse_digits(&trimmed[1..], 8, false).or(Some(0))
        }
    } else {
        parse_digits(trimmed, 10, true)
    };

    match parsed {
        Some(value) => {
            *result = value;
            1
        }
        None => 0,
    }
}

pub unsafe fn dir_name(path: *const c_char) -> *mut c_char {
    let bytes = bytes_from_cstr(path);
    match bytes.iter().rposition(|b| *b == b'/') {
        Some(pos) => alloc_bytes(&bytes[..pos]),
        None => alloc_bytes(b"."),
    }
}

pub unsafe fn base_name(path: *const c_char) -> *const c_char {
    if path.is_null() {
        return ptr::null();
    }

    let bytes = bytes_from_cstr(path);
    match bytes.iter().rposition(|b| *b == b'/') {
        Some(pos) => path.cast::<u8>().add(pos + 1).cast::<c_char>(),
        None => path,
    }
}

pub unsafe fn extract_file_base(path: *const c_char, dest: *mut c_char) {
    if dest.is_null() {
        return;
    }

    let bytes = bytes_from_cstr(path);
    ptr::write_bytes(dest, 0, 8);

    let start = bytes
        .iter()
        .rposition(|b| *b == DIR_SEPARATOR)
        .map_or(0, |pos| pos + 1);

    for (idx, b) in bytes[start..]
        .iter()
        .take_while(|b| **b != b'.')
        .take(8)
        .enumerate()
    {
        *dest.add(idx) = b.to_ascii_uppercase() as c_char;
    }
}

pub unsafe fn force_uppercase(text: *mut c_char) {
    if text.is_null() {
        return;
    }

    for b in mut_bytes_from_cstr(text) {
        *b = b.to_ascii_uppercase();
    }
}

pub unsafe fn force_lowercase(text: *mut c_char) {
    if text.is_null() {
        return;
    }

    for b in mut_bytes_from_cstr(text) {
        *b = b.to_ascii_lowercase();
    }
}

pub unsafe fn str_case_str(haystack: *const c_char, needle: *const c_char) -> *const c_char {
    if haystack.is_null() || needle.is_null() {
        return ptr::null();
    }

    let haystack_bytes = bytes_from_cstr(haystack);
    let needle_bytes = bytes_from_cstr(needle);

    if needle_bytes.len() > haystack_bytes.len() {
        return ptr::null();
    }

    for i in 0..=haystack_bytes.len() - needle_bytes.len() {
        if haystack_bytes[i..i + needle_bytes.len()].eq_ignore_ascii_case(needle_bytes) {
            return haystack.cast::<u8>().add(i).cast::<c_char>();
        }
    }

    ptr::null()
}

pub unsafe fn string_duplicate(orig: *const c_char) -> *mut c_char {
    alloc_bytes(bytes_from_cstr(orig))
}

pub unsafe fn string_copy(dest: *mut c_char, src: *const c_char, dest_size: usize) -> c_int {
    if dest.is_null() || dest_size == 0 {
        return 0;
    }

    let src_bytes = bytes_from_cstr(src);
    let copy_len = src_bytes.len().min(dest_size - 1);
    ptr::copy_nonoverlapping(src_bytes.as_ptr(), dest.cast::<u8>(), copy_len);
    *dest.cast::<u8>().add(copy_len) = 0;

    (copy_len == src_bytes.len()) as c_int
}

pub unsafe fn string_concat(dest: *mut c_char, src: *const c_char, dest_size: usize) -> c_int {
    if dest.is_null() {
        return 0;
    }

    let offset = bytes_from_cstr(dest).len().min(dest_size);
    string_copy(dest.add(offset), src, dest_size.saturating_sub(offset))
}

pub unsafe fn string_replace(
    haystack: *const c_char,
    needle: *const c_char,
    replacement: *const c_char,
) -> *mut c_char {
    let haystack = bytes_from_cstr(haystack);
    let needle = bytes_from_cstr(needle);
    let replacement = bytes_from_cstr(replacement);

    if needle.is_empty() {
        return alloc_bytes(haystack);
    }

    let mut result = Vec::with_capacity(haystack.len() + 1);
    let mut pos = 0;

    while pos < haystack.len() {
        if haystack[pos..].starts_with(needle) {
            result.extend_from_slice(replacement);
            pos += needle.len();
        } else {
            result.push(haystack[pos]);
            pos += 1;
        }
    }

    alloc_bytes(&result)
}

pub unsafe fn string_starts_with(s: *const c_char, prefix: *const c_char) -> c_int {
    bytes_from_cstr(s).starts_with(bytes_from_cstr(prefix)) as c_int
}

pub unsafe fn string_ends_with(s: *const c_char, suffix: *const c_char) -> c_int {
    bytes_from_cstr(s).ends_with(bytes_from_cstr(suffix)) as c_int
}

pub unsafe fn normalize_slashes(str_ptr: *mut c_char) {
    if str_ptr.is_null() {
        return;
    }

    let bytes = mut_bytes_from_cstr(str_ptr);

    for b in bytes.iter_mut() {
        if *b == b'\\' {
            *b = DIR_SEPARATOR;
        }
    }

    let mut read = 0;
    let mut write = 0;
    let mut previous_was_slash = false;

    while read < bytes.len() {
        let b = bytes[read];
        if b == DIR_SEPARATOR {
            if !previous_was_slash {
                bytes[write] = b;
                write += 1;
            }
            previous_was_slash = true;
        } else {
            bytes[write] = b;
            write += 1;
            previous_was_slash = false;
        }
        read += 1;
    }

    while write > 0 && bytes[write - 1] == DIR_SEPARATOR {
        write -= 1;
    }

    *str_ptr.cast::<u8>().add(write) = 0;
}

pub unsafe fn write_file(name: *const c_char, source: *const c_void, length: c_int) -> c_int {
    if name.is_null() || source.is_null() || length < 0 {
        return 0;
    }

    let path = match CStr::from_ptr(name).to_str() {
        Ok(path) => path,
        Err(_) => return 0,
    };
    let bytes = slice::from_raw_parts(source.cast::<u8>(), length as usize);

    std::fs::write(path, bytes).is_ok() as c_int
}

pub unsafe fn make_directory(path: *const c_char) {
    if path.is_null() {
        return;
    }

    if let Ok(path) = CStr::from_ptr(path).to_str() {
        let _ = std::fs::create_dir(path);
    }
}

pub unsafe fn temp_file(name: *const c_char) -> *mut c_char {
    let tempdir = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
    let name = bytes_from_cstr(name);
    let mut result = tempdir.into_bytes();
    result.push(DIR_SEPARATOR);
    result.extend_from_slice(name);
    alloc_bytes(&result)
}

pub unsafe fn file_exists(filename: *const c_char) -> c_int {
    if filename.is_null() {
        return 0;
    }

    match CStr::from_ptr(filename).to_str() {
        Ok(path) => std::fs::metadata(path).is_ok() as c_int,
        Err(_) => 0,
    }
}

pub unsafe fn file_case_exists(path: *const c_char) -> *mut c_char {
    let original = bytes_from_cstr(path);
    let candidate = alloc_bytes(original);

    if file_exists(candidate) != 0 {
        return candidate;
    }

    free(candidate.cast::<c_void>());

    let slash = original
        .iter()
        .rposition(|b| *b == DIR_SEPARATOR)
        .map_or(0, |pos| pos + 1);

    let mut lower = original.to_vec();
    for b in &mut lower[slash..] {
        *b = b.to_ascii_lowercase();
    }
    let lower_ptr = alloc_bytes(&lower);
    if file_exists(lower_ptr) != 0 {
        return lower_ptr;
    }
    free(lower_ptr.cast::<c_void>());

    let mut upper = original.to_vec();
    for b in &mut upper[slash..] {
        *b = b.to_ascii_uppercase();
    }
    let upper_ptr = alloc_bytes(&upper);
    if file_exists(upper_ptr) != 0 {
        return upper_ptr;
    }
    free(upper_ptr.cast::<c_void>());

    ptr::null_mut()
}
