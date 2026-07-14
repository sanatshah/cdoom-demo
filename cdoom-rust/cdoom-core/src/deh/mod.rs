//! Rust implementation of shared Dehacked/BEX parser mechanics.
//!
//! The C side still owns file/lump allocation and game-specific section
//! callbacks. This module mirrors the old C data layouts and moves the common
//! parser, line reader, string replacement table, and mapping helpers behind a
//! stable ABI.

#![allow(unsafe_code)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::ptr;
use std::sync::{Mutex, OnceLock};

const MAX_MAPPING_ENTRIES: usize = 32;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DehInputType {
    File = 0,
    Lump = 1,
}

#[repr(C)]
#[derive(Debug)]
pub struct DehContext {
    pub input_type: DehInputType,
    pub filename: *mut c_char,
    pub input_buffer: *mut u8,
    pub input_buffer_len: usize,
    pub input_buffer_pos: c_uint,
    pub lumpnum: c_int,
    pub stream: *mut c_void,
    pub linenum: c_int,
    pub last_was_newline: c_int,
    pub readbuffer: *mut c_char,
    pub readbuffer_size: c_int,
    pub had_error: c_int,
}

pub type SectionInitFn = Option<unsafe extern "C" fn()>;
pub type SectionStartFn =
    Option<unsafe extern "C" fn(context: *mut c_void, line: *mut c_char) -> *mut c_void>;
pub type SectionLineParserFn =
    Option<unsafe extern "C" fn(context: *mut c_void, line: *mut c_char, tag: *mut c_void)>;
pub type SectionEndFn = Option<unsafe extern "C" fn(context: *mut c_void, tag: *mut c_void)>;
pub type SectionSha1HashFn = Option<unsafe extern "C" fn(context: *mut c_void)>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DehSection {
    pub name: *const c_char,
    pub init: SectionInitFn,
    pub start: SectionStartFn,
    pub line_parser: SectionLineParserFn,
    pub end: SectionEndFn,
    pub sha1_hash: SectionSha1HashFn,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DehMappingEntry {
    pub name: *const c_char,
    pub location: *mut c_void,
    pub size: c_int,
    pub is_string: c_int,
}

#[repr(C)]
pub struct DehMapping {
    pub base: *mut c_void,
    pub entries: [DehMappingEntry; MAX_MAPPING_ENTRIES],
}

pub type DehRawReadCharFn = Option<unsafe extern "C" fn(context: *mut c_void) -> c_int>;
pub type DehRawUnreadCharFn = Option<unsafe extern "C" fn(context: *mut c_void, ch: c_int)>;
pub type DehGetCharFn = Option<unsafe extern "C" fn(context: *mut c_void) -> c_int>;
pub type DehIncreaseReadBufferFn = Option<unsafe extern "C" fn(context: *mut c_void)>;
pub type DehReadLineFn =
    Option<unsafe extern "C" fn(context: *mut c_void, extended: c_int) -> *mut c_char>;
pub type DehHadErrorFn = Option<unsafe extern "C" fn(context: *mut c_void) -> c_int>;
pub type DehContextOnlyFn = Option<unsafe extern "C" fn(context: *mut c_void)>;
pub type DehContextNameFn = Option<unsafe extern "C" fn(context: *mut c_void, name: *const c_char)>;
pub type DehStringCopyFn =
    Option<unsafe extern "C" fn(dest: *mut c_char, src: *const c_char, dest_size: usize) -> c_int>;
pub type DehSha1UpdateInt32Fn = Option<unsafe extern "C" fn(context: *mut c_void, value: c_uint)>;
pub type DehNameOnlyFn = Option<unsafe extern "C" fn(name: *const c_char)>;
pub type DehAddStringReplacementFn =
    Option<unsafe extern "C" fn(from_text: *const c_char, to_text: *const c_char)>;

#[derive(Default)]
struct StringReplacements {
    entries: Vec<(CString, CString)>,
}

fn replacements() -> &'static Mutex<StringReplacements> {
    static REPLACEMENTS: OnceLock<Mutex<StringReplacements>> = OnceLock::new();
    REPLACEMENTS.get_or_init(|| Mutex::new(StringReplacements::default()))
}

fn c_str_bytes(ptr: *const c_char) -> Option<&'static [u8]> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: Callers pass C strings from Chocolate Doom. We only borrow the
        // bytes before the first NUL and never retain this borrowed slice.
        Some(unsafe { CStr::from_ptr(ptr).to_bytes() })
    }
}

fn ascii_eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn ascii_starts_ignore_case(a: &[u8], prefix: &[u8]) -> bool {
    a.len() >= prefix.len() && a[..prefix.len()].eq_ignore_ascii_case(prefix)
}

fn is_c_space(ch: u8) -> bool {
    matches!(ch, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

fn trim_assignment_part(mut start: *mut c_char) -> *mut c_char {
    // SAFETY: `start` points into a mutable NUL-terminated C string.
    unsafe {
        while *start != 0 && is_c_space(*start as u8) {
            start = start.add(1);
        }

        let mut len = 0usize;
        while *start.add(len) != 0 {
            len += 1;
        }

        if len == 0 {
            return start;
        }

        let mut end = start.add(len - 1);
        while end >= start && is_c_space(*end as u8) {
            *end = 0;
            if end == start {
                break;
            }
            end = end.sub(1);
        }

        start
    }
}

pub unsafe fn parse_assignment(
    line: *mut c_char,
    variable_name: *mut *mut c_char,
    value: *mut *mut c_char,
) -> c_int {
    if line.is_null() || variable_name.is_null() || value.is_null() {
        return 0;
    }

    let mut p = line;
    while *p != 0 {
        if *p == b'=' as c_char {
            *p = 0;
            *variable_name = trim_assignment_part(line);
            *value = trim_assignment_part(p.add(1));
            return 1;
        }
        p = p.add(1);
    }

    0
}

pub unsafe fn add_string_replacement(from_text: *const c_char, to_text: *const c_char) {
    let Some(from_bytes) = c_str_bytes(from_text) else {
        return;
    };
    let Some(to_bytes) = c_str_bytes(to_text) else {
        return;
    };

    let mut table = replacements()
        .lock()
        .expect("deh replacements mutex poisoned");

    if let Some((_, existing_to)) = table
        .entries
        .iter_mut()
        .find(|(from, _)| from.as_bytes() == from_bytes)
    {
        *existing_to = CString::new(to_bytes).expect("CStr bytes never contain an interior NUL");
        return;
    }

    table.entries.push((
        CString::new(from_bytes).expect("CStr bytes never contain an interior NUL"),
        CString::new(to_bytes).expect("CStr bytes never contain an interior NUL"),
    ));
}

pub unsafe fn string_replacement(s: *const c_char) -> *const c_char {
    let Some(bytes) = c_str_bytes(s) else {
        return ptr::null();
    };

    let table = replacements()
        .lock()
        .expect("deh replacements mutex poisoned");

    if let Some((_, to_text)) = table
        .entries
        .iter()
        .find(|(from, _)| from.as_bytes() == bytes)
    {
        to_text.as_ptr()
    } else {
        s
    }
}

#[doc(hidden)]
pub fn clear_string_replacements_for_tests() {
    replacements()
        .lock()
        .expect("deh replacements mutex poisoned")
        .entries
        .clear();
}

pub unsafe fn get_char(
    context: *mut c_void,
    read_raw_char: DehRawReadCharFn,
    unread_raw_char: DehRawUnreadCharFn,
) -> c_int {
    let Some(read_raw_char) = read_raw_char else {
        return -1;
    };
    let Some(unread_raw_char) = unread_raw_char else {
        return -1;
    };

    let context_fields = context as *mut DehContext;

    if (*context_fields).last_was_newline != 0 {
        (*context_fields).linenum += 1;
    }

    let mut last_was_cr = false;
    let mut result;

    loop {
        result = read_raw_char(context);

        if last_was_cr && result != b'\n' as c_int {
            unread_raw_char(context, result);
            return b'\r' as c_int;
        }

        last_was_cr = result == b'\r' as c_int;
        if !last_was_cr {
            break;
        }
    }

    (*context_fields).last_was_newline = (result == b'\n' as c_int) as c_int;

    result
}

pub unsafe fn read_line(
    context: *mut c_void,
    extended: c_int,
    get_char: DehGetCharFn,
    increase_read_buffer: DehIncreaseReadBufferFn,
) -> *mut c_char {
    let Some(get_char) = get_char else {
        return ptr::null_mut();
    };
    let Some(increase_read_buffer) = increase_read_buffer else {
        return ptr::null_mut();
    };

    let context_fields = context as *mut DehContext;
    let mut pos = 0isize;
    let mut escaped = false;

    loop {
        let mut ch = get_char(context);

        if ch < 0 && pos == 0 {
            return ptr::null_mut();
        }

        if pos >= (*context_fields).readbuffer_size as isize {
            increase_read_buffer(context);
        }

        if extended != 0 && ch == b'\\' as c_int {
            ch = get_char(context);

            if ch == b'n' as c_int {
                *(*context_fields).readbuffer.offset(pos) = b'\n' as c_char;
                pos += 1;
                continue;
            }

            if ch == b'\n' as c_int {
                escaped = true;
                continue;
            }
        }

        if escaped && ch >= 0 && is_c_space(ch as u8) && ch != b'\n' as c_int {
            continue;
        } else {
            escaped = false;
        }

        if ch == b'\n' as c_int || ch < 0 {
            *(*context_fields).readbuffer.offset(pos) = 0;
            break;
        } else if ch != 0 {
            *(*context_fields).readbuffer.offset(pos) = ch as c_char;
            pos += 1;
        }
    }

    (*context_fields).readbuffer
}

fn section_name_from_line(line: *mut c_char) -> Vec<u8> {
    if line.is_null() {
        return Vec::new();
    }

    let bytes = unsafe { CStr::from_ptr(line).to_bytes() };
    bytes
        .iter()
        .copied()
        .take_while(|ch| !is_c_space(*ch))
        .take(19)
        .collect()
}

unsafe fn section_name(section: *mut DehSection) -> &'static [u8] {
    if section.is_null() || (*section).name.is_null() {
        b""
    } else {
        CStr::from_ptr((*section).name).to_bytes()
    }
}

unsafe fn get_section_by_name(
    section_types: *mut *mut DehSection,
    name: &[u8],
    allow_extended_strings: c_int,
) -> *mut DehSection {
    if allow_extended_strings == 0 && ascii_starts_ignore_case(name, b"[STRINGS]") {
        return ptr::null_mut();
    }

    let mut index = 0usize;
    loop {
        let section = *section_types.add(index);
        if section.is_null() {
            return ptr::null_mut();
        }

        if ascii_eq_ignore_case(section_name(section), name) {
            return section;
        }

        index += 1;
    }
}

unsafe fn check_signatures(
    context: *mut c_void,
    signatures: *const *const c_char,
    read_line: unsafe extern "C" fn(context: *mut c_void, extended: c_int) -> *mut c_char,
) -> bool {
    let line = read_line(context, 0);

    if line.is_null() {
        return false;
    }

    let line_bytes = CStr::from_ptr(line).to_bytes();
    let mut index = 0usize;

    loop {
        let signature = *signatures.add(index);
        if signature.is_null() {
            return false;
        }

        if CStr::from_ptr(signature).to_bytes() == line_bytes {
            return true;
        }

        index += 1;
    }
}

unsafe fn parse_comment(
    comment: *mut c_char,
    allow_long_strings: *mut c_int,
    allow_long_cheats: *mut c_int,
    allow_extended_strings: *mut c_int,
) {
    if comment.is_null() {
        return;
    }

    let bytes = CStr::from_ptr(comment).to_bytes();

    if bytes
        .windows(b"*allow-long-strings*".len())
        .any(|w| w == b"*allow-long-strings*")
    {
        *allow_long_strings = 1;
    }

    if bytes
        .windows(b"*allow-long-cheats*".len())
        .any(|w| w == b"*allow-long-cheats*")
    {
        *allow_long_cheats = 1;
    }

    if bytes
        .windows(b"*allow-extended-strings*".len())
        .any(|w| w == b"*allow-extended-strings*")
    {
        *allow_extended_strings = 1;
    }
}

unsafe fn skip_leading_whitespace(mut line: *mut c_char) -> *mut c_char {
    while *line != 0 && is_c_space(*line as u8) {
        line = line.add(1);
    }

    line
}

unsafe fn is_whitespace(mut line: *mut c_char) -> bool {
    while *line != 0 {
        if !is_c_space(*line as u8) {
            return false;
        }
        line = line.add(1);
    }

    true
}

pub unsafe fn parse_context(
    context: *mut c_void,
    section_types: *mut c_void,
    signatures: *const c_void,
    allow_long_strings: *mut c_int,
    allow_long_cheats: *mut c_int,
    allow_extended_strings: *mut c_int,
    read_line: DehReadLineFn,
    had_error: DehHadErrorFn,
    invalid_patch_error: DehContextOnlyFn,
) {
    let Some(read_line) = read_line else {
        return;
    };
    let Some(had_error) = had_error else {
        return;
    };
    let Some(invalid_patch_error) = invalid_patch_error else {
        return;
    };

    let section_types = section_types as *mut *mut DehSection;
    let signatures = signatures as *const *const c_char;

    if !check_signatures(context, signatures, read_line) {
        invalid_patch_error(context);
    }

    let mut current_section: *mut DehSection = ptr::null_mut();
    let mut tag: *mut c_void = ptr::null_mut();

    while had_error(context) == 0 {
        let extended = if current_section.is_null() {
            0
        } else {
            ascii_eq_ignore_case(section_name(current_section), b"[STRINGS]") as c_int
        };
        let mut line = read_line(context, extended);

        if line.is_null() {
            return;
        }

        line = skip_leading_whitespace(line);

        if *line == b'#' as c_char {
            parse_comment(
                line,
                allow_long_strings,
                allow_long_cheats,
                allow_extended_strings,
            );
            continue;
        }

        if is_whitespace(line) {
            if !current_section.is_null() {
                if let Some(end) = (*current_section).end {
                    end(context, tag);
                }
                current_section = ptr::null_mut();
            }
            continue;
        }

        if !current_section.is_null() {
            if let Some(line_parser) = (*current_section).line_parser {
                line_parser(context, line, tag);
            }
        } else {
            let section_name = section_name_from_line(line);
            current_section =
                get_section_by_name(section_types, &section_name, *allow_extended_strings);

            if !current_section.is_null() {
                if let Some(start) = (*current_section).start {
                    tag = start(context, line);
                } else {
                    tag = ptr::null_mut();
                }
            }
        }
    }
}

fn parse_c_int(bytes: &[u8]) -> Option<(c_int, usize)> {
    let mut index = 0usize;
    let mut sign = 1i64;

    if bytes.get(index) == Some(&b'+') {
        index += 1;
    } else if bytes.get(index) == Some(&b'-') {
        sign = -1;
        index += 1;
    }

    let mut base = 10u32;

    if bytes.get(index) == Some(&b'0') {
        if matches!(bytes.get(index + 1), Some(b'x' | b'X')) {
            base = 16;
            index += 2;
        } else {
            base = 8;
            index += 1;
        }
    }

    let digits_start = index;
    let mut value = 0i64;

    while let Some(ch) = bytes.get(index).copied() {
        let digit = match ch {
            b'0'..=b'9' => (ch - b'0') as u32,
            b'a'..=b'f' => (ch - b'a' + 10) as u32,
            b'A'..=b'F' => (ch - b'A' + 10) as u32,
            _ => break,
        };

        if digit >= base {
            break;
        }

        value = value.wrapping_mul(base as i64).wrapping_add(digit as i64);
        index += 1;
    }

    let parsed_digits = if base == 8 && digits_start > 0 {
        index > digits_start || bytes.get(digits_start - 1) == Some(&b'0')
    } else {
        index > digits_start
    };

    if !parsed_digits {
        return None;
    }

    Some(((value.wrapping_mul(sign)) as c_int, index))
}

fn scan_text_lengths(line: *const c_char) -> Option<(c_int, c_int)> {
    if line.is_null() {
        return None;
    }

    let bytes = unsafe { CStr::from_ptr(line).to_bytes() };
    let mut index = 0usize;

    if !bytes
        .get(index..index + 4)
        .is_some_and(|prefix| prefix == b"Text")
    {
        return None;
    }
    index += 4;

    if !bytes.get(index).is_some_and(|ch| is_c_space(*ch)) {
        return None;
    }
    while bytes.get(index).is_some_and(|ch| is_c_space(*ch)) {
        index += 1;
    }

    let (from_len, consumed) = parse_c_int(&bytes[index..])?;
    index += consumed;

    if !bytes.get(index).is_some_and(|ch| is_c_space(*ch)) {
        return None;
    }
    while bytes.get(index).is_some_and(|ch| is_c_space(*ch)) {
        index += 1;
    }

    let (to_len, _) = parse_c_int(&bytes[index..])?;

    Some((from_len, to_len))
}

pub fn max_string_length(mut len: c_int) -> c_int {
    len += 1;
    len += (4 - (len % 4)) % 4;
    len - 1
}

pub unsafe fn text_start(
    context: *mut c_void,
    line: *mut c_char,
    allow_long_strings: c_int,
    get_char: DehGetCharFn,
    warn_parse_error: DehContextOnlyFn,
    error_replacement_too_long: DehContextOnlyFn,
    add_string_replacement: DehAddStringReplacementFn,
) -> *mut c_void {
    let Some(get_char) = get_char else {
        return ptr::null_mut();
    };
    let Some(warn_parse_error) = warn_parse_error else {
        return ptr::null_mut();
    };
    let Some(error_replacement_too_long) = error_replacement_too_long else {
        return ptr::null_mut();
    };
    let Some(add_string_replacement) = add_string_replacement else {
        return ptr::null_mut();
    };

    let Some((from_len, to_len)) = scan_text_lengths(line) else {
        warn_parse_error(context);
        return ptr::null_mut();
    };

    if from_len < 0 || to_len < 0 {
        warn_parse_error(context);
        return ptr::null_mut();
    }

    if allow_long_strings == 0 && to_len > max_string_length(from_len) {
        error_replacement_too_long(context);
        return ptr::null_mut();
    }

    let mut from_text = Vec::with_capacity(from_len as usize + 1);
    for _ in 0..from_len {
        from_text.push(get_char(context) as u8);
    }
    from_text.push(0);

    let mut to_text = Vec::with_capacity(to_len as usize + 1);
    for _ in 0..to_len {
        to_text.push(get_char(context) as u8);
    }
    to_text.push(0);

    add_string_replacement(
        from_text.as_ptr() as *const c_char,
        to_text.as_ptr() as *const c_char,
    );

    ptr::null_mut()
}

unsafe fn mapping_entries(mapping: *mut DehMapping) -> impl Iterator<Item = *mut DehMappingEntry> {
    (0..MAX_MAPPING_ENTRIES).map(move |i| (*mapping).entries.as_mut_ptr().add(i))
}

unsafe fn mapping_entry_by_name(
    context: *mut c_void,
    mapping: *mut DehMapping,
    name: *const c_char,
    warn_unsupported: DehContextNameFn,
    warn_not_found: DehContextNameFn,
) -> *mut DehMappingEntry {
    for entry in mapping_entries(mapping) {
        if (*entry).name.is_null() {
            break;
        }

        if c_str_bytes((*entry).name)
            .zip(c_str_bytes(name))
            .is_some_and(|(entry_name, query)| ascii_eq_ignore_case(entry_name, query))
        {
            if (*entry).location.is_null() {
                if let Some(warn_unsupported) = warn_unsupported {
                    warn_unsupported(context, name);
                }
                return ptr::null_mut();
            }

            return entry;
        }
    }

    if let Some(warn_not_found) = warn_not_found {
        warn_not_found(context, name);
    }

    ptr::null_mut()
}

unsafe fn struct_field(
    structptr: *mut c_void,
    mapping: *mut DehMapping,
    entry: *mut DehMappingEntry,
) -> *mut c_void {
    let offset = ((*entry).location as usize).wrapping_sub((*mapping).base as usize);
    (structptr as *mut u8).add(offset) as *mut c_void
}

pub unsafe fn set_mapping(
    context: *mut c_void,
    mapping: *mut c_void,
    structptr: *mut c_void,
    name: *mut c_char,
    value: c_int,
    warn_unsupported: DehContextNameFn,
    warn_not_found: DehContextNameFn,
    error_int_as_string: DehContextNameFn,
    error_unknown_field_size: DehContextNameFn,
) -> c_int {
    let mapping = mapping as *mut DehMapping;
    let entry = mapping_entry_by_name(context, mapping, name, warn_unsupported, warn_not_found);

    if entry.is_null() {
        return 0;
    }

    if (*entry).is_string != 0 {
        if let Some(error_int_as_string) = error_int_as_string {
            error_int_as_string(context, name);
        }
        return 0;
    }

    let location = struct_field(structptr, mapping, entry);

    match (*entry).size {
        1 => ptr::write(location as *mut u8, value as u8),
        2 => ptr::write_unaligned(location as *mut u16, value as u16),
        4 => ptr::write_unaligned(location as *mut u32, value as u32),
        _ => {
            if let Some(error_unknown_field_size) = error_unknown_field_size {
                error_unknown_field_size(context, name);
            }
            return 0;
        }
    }

    1
}

pub unsafe fn set_string_mapping(
    context: *mut c_void,
    mapping: *mut c_void,
    structptr: *mut c_void,
    name: *mut c_char,
    value: *mut c_char,
    warn_unsupported: DehContextNameFn,
    warn_not_found: DehContextNameFn,
    error_string_as_int: DehContextNameFn,
    copy_string: DehStringCopyFn,
) -> c_int {
    let mapping = mapping as *mut DehMapping;
    let entry = mapping_entry_by_name(context, mapping, name, warn_unsupported, warn_not_found);

    if entry.is_null() {
        return 0;
    }

    if (*entry).is_string == 0 {
        if let Some(error_string_as_int) = error_string_as_int {
            error_string_as_int(context, name);
        }
        return 0;
    }

    if let Some(copy_string) = copy_string {
        copy_string(
            struct_field(structptr, mapping, entry) as *mut c_char,
            value,
            (*entry).size as usize,
        );
        1
    } else {
        0
    }
}

pub unsafe fn struct_sha1_sum(
    sha1_context: *mut c_void,
    mapping: *mut c_void,
    structptr: *mut c_void,
    update_int32: DehSha1UpdateInt32Fn,
    fatal_unknown_field_size: DehNameOnlyFn,
) {
    let mapping = mapping as *mut DehMapping;
    let Some(update_int32) = update_int32 else {
        return;
    };

    for entry in mapping_entries(mapping) {
        if (*entry).name.is_null() {
            break;
        }

        if (*entry).location.is_null() {
            continue;
        }

        let location = struct_field(structptr, mapping, entry);

        match (*entry).size {
            1 => update_int32(sha1_context, ptr::read(location as *mut u8) as c_uint),
            2 => update_int32(
                sha1_context,
                ptr::read_unaligned(location as *mut u16) as c_uint,
            ),
            4 => update_int32(
                sha1_context,
                ptr::read_unaligned(location as *mut u32) as c_uint,
            ),
            _ => {
                if let Some(fatal_unknown_field_size) = fatal_unknown_field_size {
                    fatal_unknown_field_size((*entry).name);
                }
            }
        }
    }
}
