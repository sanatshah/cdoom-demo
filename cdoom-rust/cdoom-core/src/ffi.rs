//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::deh;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};
use std::sync::OnceLock;

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

/// Parse a Dehacked `name = value` assignment in place.
///
/// Returns `1` on success and `0` when no assignment separator is present.
///
/// # Safety
///
/// `line` must point to mutable NUL-terminated storage. `variable_name` and
/// `value` must be valid out-pointers.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_deh_parse_assignment(
    line: *mut c_char,
    variable_name: *mut *mut c_char,
    value: *mut *mut c_char,
) -> c_int {
    deh::parse_assignment(line, variable_name, value)
}

/// Look up a Dehacked string replacement.
///
/// # Safety
///
/// `s` must be a valid NUL-terminated string. Returned pointers are either the
/// original pointer or storage owned by the Rust replacement table.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_deh_string(s: *const c_char) -> *const c_char {
    deh::string_replacement(s)
}

/// Add or replace a Dehacked string substitution.
///
/// # Safety
///
/// Both arguments must be valid NUL-terminated strings.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_deh_add_string_replacement(
    from_text: *const c_char,
    to_text: *const c_char,
) {
    deh::add_string_replacement(from_text, to_text);
}

/// Read one character from a Dehacked context, matching `DEH_GetChar`.
///
/// # Safety
///
/// `context` must point to a C `deh_context_t` and callbacks must operate on
/// the same context layout.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_deh_get_char(
    context: *mut c_void,
    read_raw_char: deh::DehRawReadCharFn,
    unread_raw_char: deh::DehRawUnreadCharFn,
) -> c_int {
    deh::get_char(context, read_raw_char, unread_raw_char)
}

/// Read one logical line from a Dehacked context.
///
/// # Safety
///
/// `context` must point to a C `deh_context_t` and callbacks must operate on
/// the same context layout.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_deh_read_line(
    context: *mut c_void,
    extended: c_int,
    get_char: deh::DehGetCharFn,
    increase_read_buffer: deh::DehIncreaseReadBufferFn,
) -> *mut c_char {
    deh::read_line(context, extended, get_char, increase_read_buffer)
}

/// Parse a Dehacked file/lump context through C section callbacks.
///
/// # Safety
///
/// Pointers must reference Chocolate Doom's live Dehacked globals and callback
/// functions for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_deh_parse_context(
    context: *mut c_void,
    section_types: *mut c_void,
    signatures: *const c_void,
    allow_long_strings: *mut c_int,
    allow_long_cheats: *mut c_int,
    allow_extended_strings: *mut c_int,
    read_line: deh::DehReadLineFn,
    had_error: deh::DehHadErrorFn,
    invalid_patch_error: deh::DehContextOnlyFn,
) {
    deh::parse_context(
        context,
        section_types,
        signatures,
        allow_long_strings,
        allow_long_cheats,
        allow_extended_strings,
        read_line,
        had_error,
        invalid_patch_error,
    );
}

/// Maximum replacement string length allowed by vanilla Dehacked.
#[no_mangle]
pub extern "C" fn cdoom_rust_deh_max_string_length(len: c_int) -> c_int {
    deh::max_string_length(len)
}

/// Start and parse a `Text` replacement section.
///
/// # Safety
///
/// Pointers and callbacks must reference valid Chocolate Doom Dehacked state.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_deh_text_start(
    context: *mut c_void,
    line: *mut c_char,
    allow_long_strings: c_int,
    get_char: deh::DehGetCharFn,
    warn_parse_error: deh::DehContextOnlyFn,
    error_replacement_too_long: deh::DehContextOnlyFn,
    add_string_replacement: deh::DehAddStringReplacementFn,
) -> *mut c_void {
    deh::text_start(
        context,
        line,
        allow_long_strings,
        get_char,
        warn_parse_error,
        error_replacement_too_long,
        add_string_replacement,
    )
}

/// Set an integer field in a mapped Dehacked structure.
///
/// # Safety
///
/// Pointers must refer to C `deh_mapping_t`, target structure, and C strings.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_deh_set_mapping(
    context: *mut c_void,
    mapping: *mut c_void,
    structptr: *mut c_void,
    name: *mut c_char,
    value: c_int,
    warn_unsupported: deh::DehContextNameFn,
    warn_not_found: deh::DehContextNameFn,
    error_int_as_string: deh::DehContextNameFn,
    error_unknown_field_size: deh::DehContextNameFn,
) -> c_int {
    deh::set_mapping(
        context,
        mapping,
        structptr,
        name,
        value,
        warn_unsupported,
        warn_not_found,
        error_int_as_string,
        error_unknown_field_size,
    )
}

/// Set a string field in a mapped Dehacked structure.
///
/// # Safety
///
/// Pointers must refer to C `deh_mapping_t`, target structure, and C strings.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_deh_set_string_mapping(
    context: *mut c_void,
    mapping: *mut c_void,
    structptr: *mut c_void,
    name: *mut c_char,
    value: *mut c_char,
    warn_unsupported: deh::DehContextNameFn,
    warn_not_found: deh::DehContextNameFn,
    error_string_as_int: deh::DehContextNameFn,
    copy_string: deh::DehStringCopyFn,
) -> c_int {
    deh::set_string_mapping(
        context,
        mapping,
        structptr,
        name,
        value,
        warn_unsupported,
        warn_not_found,
        error_string_as_int,
        copy_string,
    )
}

/// Add mapped structure fields into a SHA1 context.
///
/// # Safety
///
/// Pointers must refer to C `sha1_context_t`, `deh_mapping_t`, and target
/// structure storage.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_deh_struct_sha1_sum(
    sha1_context: *mut c_void,
    mapping: *mut c_void,
    structptr: *mut c_void,
    update_int32: deh::DehSha1UpdateInt32Fn,
    fatal_unknown_field_size: deh::DehNameOnlyFn,
) {
    deh::struct_sha1_sum(
        sha1_context,
        mapping,
        structptr,
        update_int32,
        fatal_unknown_field_size,
    );
}
