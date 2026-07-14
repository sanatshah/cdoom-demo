//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::net;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};
use std::sync::OnceLock;

static VERSION: OnceLock<CString> = OnceLock::new();
static PROTOCOL_CHOCOLATE_DOOM_0_CSTR: &[u8] = b"CHOCOLATE_DOOM_0\0";

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
pub extern "C" fn cdoom_rust_net_expand_tic_num(relative: c_uint, b: c_uint) -> c_uint {
    net::expand_tic_num(relative, b)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_net_read_int8(
    data: *const u8,
    len: usize,
    pos: *mut c_uint,
    out: *mut c_uint,
) -> c_int {
    read_unsigned(data, len, pos, out, net::read_u8)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_net_read_int16(
    data: *const u8,
    len: usize,
    pos: *mut c_uint,
    out: *mut c_uint,
) -> c_int {
    read_unsigned(data, len, pos, out, net::read_u16)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_net_read_int32(
    data: *const u8,
    len: usize,
    pos: *mut c_uint,
    out: *mut c_uint,
) -> c_int {
    read_unsigned(data, len, pos, out, net::read_u32)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_net_read_sint8(
    data: *const u8,
    len: usize,
    pos: *mut c_uint,
    out: *mut c_int,
) -> c_int {
    read_signed(data, len, pos, out, net::read_i8)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_net_read_sint16(
    data: *const u8,
    len: usize,
    pos: *mut c_uint,
    out: *mut c_int,
) -> c_int {
    read_signed(data, len, pos, out, net::read_i16)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_net_read_sint32(
    data: *const u8,
    len: usize,
    pos: *mut c_uint,
    out: *mut c_int,
) -> c_int {
    read_signed(data, len, pos, out, net::read_i32)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_net_write_int8(
    data: *mut u8,
    available: usize,
    value: c_uint,
) -> c_int {
    write_unsigned(data, available, value, net::write_u8)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_net_write_int16(
    data: *mut u8,
    available: usize,
    value: c_uint,
) -> c_int {
    write_unsigned(data, available, value, net::write_u16)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_net_write_int32(
    data: *mut u8,
    available: usize,
    value: c_uint,
) -> c_int {
    write_unsigned(data, available, value, net::write_u32)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_net_parse_protocol_name(name: *const c_char) -> c_int {
    if name.is_null() {
        return net::NET_PROTOCOL_UNKNOWN;
    }

    let bytes = unsafe { CStr::from_ptr(name) }.to_bytes();
    net::parse_protocol_name(bytes)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_net_protocol_name(protocol: c_int) -> *const c_char {
    match net::protocol_name(protocol) {
        Some(_) => PROTOCOL_CHOCOLATE_DOOM_0_CSTR.as_ptr().cast(),
        None => std::ptr::null(),
    }
}

fn read_unsigned(
    data: *const u8,
    len: usize,
    pos: *mut c_uint,
    out: *mut c_uint,
    read: fn(&[u8], &mut u32) -> Option<u32>,
) -> c_int {
    if data.is_null() || pos.is_null() || out.is_null() {
        return 0;
    }

    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    let mut rust_pos = unsafe { *pos };
    match read(bytes, &mut rust_pos) {
        Some(value) => {
            unsafe {
                *pos = rust_pos;
                *out = value;
            }
            1
        }
        None => 0,
    }
}

fn read_signed(
    data: *const u8,
    len: usize,
    pos: *mut c_uint,
    out: *mut c_int,
    read: fn(&[u8], &mut u32) -> Option<i32>,
) -> c_int {
    if data.is_null() || pos.is_null() || out.is_null() {
        return 0;
    }

    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    let mut rust_pos = unsafe { *pos };
    match read(bytes, &mut rust_pos) {
        Some(value) => {
            unsafe {
                *pos = rust_pos;
                *out = value;
            }
            1
        }
        None => 0,
    }
}

fn write_unsigned(
    data: *mut u8,
    available: usize,
    value: c_uint,
    write: fn(&mut [u8], u32) -> bool,
) -> c_int {
    if data.is_null() {
        return 0;
    }

    let bytes = unsafe { std::slice::from_raw_parts_mut(data, available) };
    if write(bytes, value) {
        1
    } else {
        0
    }
}
