//! Rust helpers for `m_config`.
//!
//! The C module still owns the defaults tables and bound variable pointers.
//! These helpers move deterministic parsing and key translation behind the
//! Rust ABI while preserving Chocolate Doom's config file format.

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::slice;

const KEY_BACKSPACE: c_int = 0x7f;
const KEY_PAUSE: c_int = 0xff;
const KEY_RSHIFT: c_int = 0x80 + 0x36;
const KEY_RCTRL: c_int = 0x80 + 0x1d;
const KEY_RALT: c_int = 0x80 + 0x38;
const KEY_CAPSLOCK: c_int = 0x80 + 0x3a;
const KEY_F1: c_int = 0x80 + 0x3b;
const KEY_F2: c_int = 0x80 + 0x3c;
const KEY_F3: c_int = 0x80 + 0x3d;
const KEY_F4: c_int = 0x80 + 0x3e;
const KEY_F5: c_int = 0x80 + 0x3f;
const KEY_F6: c_int = 0x80 + 0x40;
const KEY_F7: c_int = 0x80 + 0x41;
const KEY_F8: c_int = 0x80 + 0x42;
const KEY_F9: c_int = 0x80 + 0x43;
const KEY_F10: c_int = 0x80 + 0x44;
const KEY_F11: c_int = 0x80 + 0x57;
const KEY_F12: c_int = 0x80 + 0x58;
const KEY_SCRLCK: c_int = 0x80 + 0x46;
const KEY_PRTSCR: c_int = 0x80 + 0x59;
const KEY_HOME: c_int = 0x80 + 0x47;
const KEY_END: c_int = 0x80 + 0x4f;
const KEY_PGUP: c_int = 0x80 + 0x49;
const KEY_PGDN: c_int = 0x80 + 0x51;
const KEY_INS: c_int = 0x80 + 0x52;
const KEY_DEL: c_int = 0x80 + 0x53;
const KEY_RIGHTARROW: c_int = 0xae;
const KEY_LEFTARROW: c_int = 0xac;
const KEY_UPARROW: c_int = 0xad;
const KEY_DOWNARROW: c_int = 0xaf;
const KEYP_5: c_int = 0x80 + 0x4c;
const KEYP_MULTIPLY: c_int = b'*' as c_int;
const KEYP_PLUS: c_int = b'+' as c_int;
const KEY_MINUS: c_int = b'-' as c_int;

pub const SCAN_TO_KEY: [c_int; 128] = [
    0,
    27,
    b'1' as c_int,
    b'2' as c_int,
    b'3' as c_int,
    b'4' as c_int,
    b'5' as c_int,
    b'6' as c_int,
    b'7' as c_int,
    b'8' as c_int,
    b'9' as c_int,
    b'0' as c_int,
    b'-' as c_int,
    b'=' as c_int,
    KEY_BACKSPACE,
    9,
    b'q' as c_int,
    b'w' as c_int,
    b'e' as c_int,
    b'r' as c_int,
    b't' as c_int,
    b'y' as c_int,
    b'u' as c_int,
    b'i' as c_int,
    b'o' as c_int,
    b'p' as c_int,
    b'[' as c_int,
    b']' as c_int,
    13,
    KEY_RCTRL,
    b'a' as c_int,
    b's' as c_int,
    b'd' as c_int,
    b'f' as c_int,
    b'g' as c_int,
    b'h' as c_int,
    b'j' as c_int,
    b'k' as c_int,
    b'l' as c_int,
    b';' as c_int,
    b'\'' as c_int,
    b'`' as c_int,
    KEY_RSHIFT,
    b'\\' as c_int,
    b'z' as c_int,
    b'x' as c_int,
    b'c' as c_int,
    b'v' as c_int,
    b'b' as c_int,
    b'n' as c_int,
    b'm' as c_int,
    b',' as c_int,
    b'.' as c_int,
    b'/' as c_int,
    KEY_RSHIFT,
    KEYP_MULTIPLY,
    KEY_RALT,
    b' ' as c_int,
    KEY_CAPSLOCK,
    KEY_F1,
    KEY_F2,
    KEY_F3,
    KEY_F4,
    KEY_F5,
    KEY_F6,
    KEY_F7,
    KEY_F8,
    KEY_F9,
    KEY_F10,
    KEY_PAUSE,
    KEY_SCRLCK,
    KEY_HOME,
    KEY_UPARROW,
    KEY_PGUP,
    KEY_MINUS,
    KEY_LEFTARROW,
    KEYP_5,
    KEY_RIGHTARROW,
    KEYP_PLUS,
    KEY_END,
    KEY_DOWNARROW,
    KEY_PGDN,
    KEY_INS,
    KEY_DEL,
    0,
    0,
    0,
    KEY_F11,
    KEY_F12,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    KEY_PRTSCR,
    0,
];

unsafe fn bytes_from_cstr<'a>(ptr: *const c_char) -> &'a [u8] {
    if ptr.is_null() {
        b""
    } else {
        CStr::from_ptr(ptr).to_bytes()
    }
}

fn parse_c_int_bytes(bytes: &[u8]) -> c_int {
    let trimmed = bytes
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .map_or(&[][..], |pos| &bytes[pos..]);

    let (radix, digits) = if trimmed.starts_with(b"0x") {
        (16, &trimmed[2..])
    } else {
        (0, trimmed)
    };

    if radix == 16 {
        parse_digits(digits, 16, false).unwrap_or(0)
    } else {
        parse_i_base_zero(digits).unwrap_or(0)
    }
}

fn parse_digits(bytes: &[u8], radix: u32, allow_sign: bool) -> Option<c_int> {
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

    (pos != start).then_some(value.wrapping_mul(sign) as c_int)
}

fn parse_i_base_zero(bytes: &[u8]) -> Option<c_int> {
    let mut pos = 0;
    let mut sign = 1i64;

    if pos < bytes.len() {
        if bytes[pos] == b'-' {
            sign = -1;
            pos += 1;
        } else if bytes[pos] == b'+' {
            pos += 1;
        }
    }

    if pos >= bytes.len() {
        return None;
    }

    let (radix, start) = if bytes[pos] == b'0'
        && pos + 1 < bytes.len()
        && (bytes[pos + 1] == b'x' || bytes[pos + 1] == b'X')
    {
        (16, pos + 2)
    } else if bytes[pos] == b'0' {
        (8, pos + 1)
    } else {
        (10, pos)
    };

    if start == bytes.len() && radix == 8 {
        return Some(0);
    }

    parse_digits(&bytes[start..], radix, false).map(|value| value.wrapping_mul(sign as c_int))
}

pub unsafe fn parse_int_parameter(value: *const c_char) -> c_int {
    parse_c_int_bytes(bytes_from_cstr(value))
}

pub fn key_from_scan(scan: c_int) -> c_int {
    if (0..SCAN_TO_KEY.len() as c_int).contains(&scan) {
        SCAN_TO_KEY[scan as usize]
    } else {
        0
    }
}

pub fn scan_from_key(key: c_int) -> c_int {
    if key == KEY_RSHIFT {
        return 54;
    }

    SCAN_TO_KEY
        .iter()
        .position(|candidate| *candidate == key)
        .map_or(key, |scan| scan as c_int)
}

pub unsafe fn parse_float_parameter(value: *const c_char) -> f32 {
    let mut bytes = bytes_from_cstr(value).to_vec();
    let mut pos = 0;

    if matches!(bytes.first(), Some(b'-' | b'+')) {
        pos = 1;
    }

    while pos < bytes.len() {
        if !bytes[pos].is_ascii_digit() {
            bytes[pos] = b'.';
            break;
        }
        pos += 1;
    }

    String::from_utf8_lossy(&bytes)
        .parse::<f32>()
        .unwrap_or(0.0)
}

pub unsafe fn clean_config_value(buffer: *mut c_char, buffer_size: usize) {
    if buffer.is_null() || buffer_size == 0 {
        return;
    }

    let len = CStr::from_ptr(buffer).to_bytes().len().min(buffer_size - 1);
    let bytes = slice::from_raw_parts_mut(buffer.cast::<u8>(), buffer_size);
    let mut end = len;

    while end > 0 && !(0x20..=0x7e).contains(&bytes[end - 1]) {
        end -= 1;
    }
    bytes[end] = 0;

    if end >= 2 && bytes[0] == b'"' && bytes[end - 1] == b'"' {
        let inner_len = end - 2;
        bytes.copy_within(1..1 + inner_len, 0);
        bytes[inner_len] = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn parses_config_int_like_c() {
        let cases = [
            ("42", 42),
            ("  -42", -42),
            ("077", 63),
            ("0x2a", 42),
            ("0X2a", 42),
            ("123junk", 123),
        ];

        for (input, expected) in cases {
            let input = CString::new(input).unwrap();
            assert_eq!(unsafe { parse_int_parameter(input.as_ptr()) }, expected);
        }
    }

    #[test]
    fn translates_key_scans_round_trip() {
        assert_eq!(key_from_scan(29), KEY_RCTRL);
        assert_eq!(key_from_scan(54), KEY_RSHIFT);
        assert_eq!(scan_from_key(KEY_RSHIFT), 54);
        assert_eq!(scan_from_key(KEY_RCTRL), 29);
        assert_eq!(scan_from_key(b'z' as c_int), 44);
    }

    #[test]
    fn cleans_quoted_config_values() {
        let mut bytes = *b"\"hello world\"\r\0\0\0\0\0\0\0";
        unsafe { clean_config_value(bytes.as_mut_ptr().cast::<c_char>(), bytes.len()) };
        let value = CStr::from_bytes_until_nul(&bytes).unwrap();
        assert_eq!(value.to_bytes(), b"hello world");
    }
}
