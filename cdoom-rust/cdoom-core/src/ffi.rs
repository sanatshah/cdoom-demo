//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};
use std::sync::OnceLock;

use crate::m_controls::{BindIntVariableFn, ControlBinding};

static VERSION: OnceLock<CString> = OnceLock::new();

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
pub unsafe extern "C" fn cdoom_rust_m_misc_str_to_int(
    str_ptr: *const c_char,
    result: *mut c_int,
) -> c_int {
    crate::m_misc::str_to_int(str_ptr, result)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_dir_name(path: *const c_char) -> *mut c_char {
    crate::m_misc::dir_name(path)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_base_name(path: *const c_char) -> *const c_char {
    crate::m_misc::base_name(path)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_extract_file_base(
    path: *const c_char,
    dest: *mut c_char,
) {
    crate::m_misc::extract_file_base(path, dest);
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_force_uppercase(text: *mut c_char) {
    crate::m_misc::force_uppercase(text);
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_force_lowercase(text: *mut c_char) {
    crate::m_misc::force_lowercase(text);
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_str_case_str(
    haystack: *const c_char,
    needle: *const c_char,
) -> *const c_char {
    crate::m_misc::str_case_str(haystack, needle)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_string_duplicate(orig: *const c_char) -> *mut c_char {
    crate::m_misc::string_duplicate(orig)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_string_copy(
    dest: *mut c_char,
    src: *const c_char,
    dest_size: usize,
) -> c_int {
    crate::m_misc::string_copy(dest, src, dest_size)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_string_concat(
    dest: *mut c_char,
    src: *const c_char,
    dest_size: usize,
) -> c_int {
    crate::m_misc::string_concat(dest, src, dest_size)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_string_replace(
    haystack: *const c_char,
    needle: *const c_char,
    replacement: *const c_char,
) -> *mut c_char {
    crate::m_misc::string_replace(haystack, needle, replacement)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_string_starts_with(
    s: *const c_char,
    prefix: *const c_char,
) -> c_int {
    crate::m_misc::string_starts_with(s, prefix)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_string_ends_with(
    s: *const c_char,
    suffix: *const c_char,
) -> c_int {
    crate::m_misc::string_ends_with(s, suffix)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_normalize_slashes(str_ptr: *mut c_char) {
    crate::m_misc::normalize_slashes(str_ptr);
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_write_file(
    name: *const c_char,
    source: *const c_void,
    length: c_int,
) -> c_int {
    crate::m_misc::write_file(name, source, length)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_make_directory(path: *const c_char) {
    crate::m_misc::make_directory(path);
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_temp_file(name: *const c_char) -> *mut c_char {
    crate::m_misc::temp_file(name)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_file_exists(filename: *const c_char) -> c_int {
    crate::m_misc::file_exists(filename)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_misc_file_case_exists(path: *const c_char) -> *mut c_char {
    crate::m_misc::file_case_exists(path)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_config_parse_int_parameter(value: *const c_char) -> c_int {
    crate::m_config::parse_int_parameter(value)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_m_config_key_from_scan(scan: c_int) -> c_int {
    crate::m_config::key_from_scan(scan)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_m_config_scan_from_key(key: c_int) -> c_int {
    crate::m_config::scan_from_key(key)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_config_parse_float_parameter(value: *const c_char) -> f32 {
    crate::m_config::parse_float_parameter(value)
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_config_clean_config_value(
    buffer: *mut c_char,
    buffer_size: usize,
) {
    crate::m_config::clean_config_value(buffer, buffer_size);
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_m_controls_bind_ints(
    bindings: *const c_void,
    count: usize,
    bind_int_variable: BindIntVariableFn,
) {
    crate::m_controls::bind_ints(bindings.cast::<ControlBinding>(), count, bind_int_variable);
}
