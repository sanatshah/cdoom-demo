//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::path::Path;

/// Expected Chocolate Doom package version vendored in this repo.
pub const CHOCOLATE_DOOM_VERSION: &str = "3.1.1";

/// Returns `true` when a timedemo baseline can run (binary + IWAD present).
pub fn timedemo_baseline_available(root: &Path) -> bool {
    let binary = root.join("chocolate-doom/build/src/chocolate-doom");
    let wad = root.join("wads/freedoom1.wad");
    binary.is_file() && wad.is_file()
}

/// Reads the exported Rust version string from the C ABI.
pub fn rust_version_from_ffi() -> String {
    let ptr = cdoom_core::cdoom_rust_version();
    assert!(!ptr.is_null());
    // SAFETY: cdoom_rust_version returns a static NUL-terminated string.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cdoom_core::deh::{DehContext, DehInputType, DehMapping, DehMappingEntry};
    use std::ptr;

    #[test]
    fn ffi_version_matches_crate() {
        assert_eq!(rust_version_from_ffi(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn init_succeeds() {
        assert_eq!(cdoom_core::cdoom_rust_init(), 0);
    }

    #[test]
    fn version_string_is_non_empty() {
        assert!(!cdoom_core::version_string().is_empty());
    }

    #[test]
    fn deh_assignment_parser_trims_and_splits_in_place() {
        let mut line = CString::new("  Hit points  =  -42  ")
            .unwrap()
            .into_bytes_with_nul();
        let mut variable_name: *mut c_char = ptr::null_mut();
        let mut value: *mut c_char = ptr::null_mut();

        let parsed = unsafe {
            cdoom_core::cdoom_rust_deh_parse_assignment(
                line.as_mut_ptr() as *mut c_char,
                &mut variable_name,
                &mut value,
            )
        };

        assert_eq!(parsed, 1);
        assert_eq!(
            unsafe { CStr::from_ptr(variable_name) }.to_str().unwrap(),
            "Hit points"
        );
        assert_eq!(unsafe { CStr::from_ptr(value) }.to_str().unwrap(), "-42");

        let mut not_assignment = CString::new("No separator").unwrap().into_bytes_with_nul();
        let parsed = unsafe {
            cdoom_core::cdoom_rust_deh_parse_assignment(
                not_assignment.as_mut_ptr() as *mut c_char,
                &mut variable_name,
                &mut value,
            )
        };
        assert_eq!(parsed, 0);
    }

    #[test]
    fn deh_string_replacements_preserve_latest_value() {
        cdoom_core::deh::clear_string_replacements_for_tests();

        let key = CString::new("GOTARMOR").unwrap();
        let first = CString::new("Picked up armor.").unwrap();
        let second = CString::new("Armor acquired.").unwrap();

        let original = unsafe { cdoom_core::cdoom_rust_deh_string(key.as_ptr()) };
        assert_eq!(original, key.as_ptr());

        unsafe {
            cdoom_core::cdoom_rust_deh_add_string_replacement(key.as_ptr(), first.as_ptr());
        }
        assert_eq!(
            unsafe { CStr::from_ptr(cdoom_core::cdoom_rust_deh_string(key.as_ptr())) }
                .to_str()
                .unwrap(),
            "Picked up armor."
        );

        unsafe {
            cdoom_core::cdoom_rust_deh_add_string_replacement(key.as_ptr(), second.as_ptr());
        }
        assert_eq!(
            unsafe { CStr::from_ptr(cdoom_core::cdoom_rust_deh_string(key.as_ptr())) }
                .to_str()
                .unwrap(),
            "Armor acquired."
        );
    }

    unsafe extern "C" fn test_read_raw_char(context: *mut c_void) -> c_int {
        let context = context as *mut DehContext;

        if (*context).input_buffer_pos as usize >= (*context).input_buffer_len {
            return -1;
        }

        let result = *(*context)
            .input_buffer
            .add((*context).input_buffer_pos as usize);
        (*context).input_buffer_pos += 1;
        result as c_int
    }

    unsafe extern "C" fn test_unread_raw_char(context: *mut c_void, _ch: c_int) {
        let context = context as *mut DehContext;
        (*context).input_buffer_pos -= 1;
    }

    unsafe extern "C" fn test_get_char(context: *mut c_void) -> c_int {
        cdoom_core::cdoom_rust_deh_get_char(
            context,
            Some(test_read_raw_char),
            Some(test_unread_raw_char),
        )
    }

    unsafe extern "C" fn test_increase_read_buffer(_context: *mut c_void) {
        panic!("test read buffer should be large enough");
    }

    fn test_context(input: &mut [u8], readbuffer: &mut [c_char]) -> DehContext {
        DehContext {
            input_type: DehInputType::Lump,
            filename: ptr::null_mut(),
            input_buffer: input.as_mut_ptr(),
            input_buffer_len: input.len(),
            input_buffer_pos: 0,
            lumpnum: 0,
            stream: ptr::null_mut(),
            linenum: 0,
            last_was_newline: 1,
            readbuffer: readbuffer.as_mut_ptr(),
            readbuffer_size: readbuffer.len() as c_int,
            had_error: 0,
        }
    }

    #[test]
    fn deh_read_line_matches_crlf_and_extended_bex_rules() {
        let mut input = b"first\r\n  second\nA\\\n   B\\nC\n".to_vec();
        let mut readbuffer = vec![0 as c_char; 128];
        let mut context = test_context(&mut input, &mut readbuffer);

        let line = unsafe {
            cdoom_core::cdoom_rust_deh_read_line(
                &mut context as *mut _ as *mut c_void,
                0,
                Some(test_get_char),
                Some(test_increase_read_buffer),
            )
        };
        assert_eq!(unsafe { CStr::from_ptr(line) }.to_str().unwrap(), "first");

        let line = unsafe {
            cdoom_core::cdoom_rust_deh_read_line(
                &mut context as *mut _ as *mut c_void,
                0,
                Some(test_get_char),
                Some(test_increase_read_buffer),
            )
        };
        assert_eq!(
            unsafe { CStr::from_ptr(line) }.to_str().unwrap(),
            "  second"
        );

        let line = unsafe {
            cdoom_core::cdoom_rust_deh_read_line(
                &mut context as *mut _ as *mut c_void,
                1,
                Some(test_get_char),
                Some(test_increase_read_buffer),
            )
        };
        assert_eq!(unsafe { CStr::from_ptr(line) }.to_str().unwrap(), "AB\nC");
    }

    #[repr(C)]
    #[derive(Default)]
    struct MappingStruct {
        small: u8,
        medium: u16,
        large: u32,
        label: [c_char; 5],
    }

    unsafe extern "C" fn warn_counter(context: *mut c_void, _name: *const c_char) {
        *(context as *mut c_int) += 1;
    }

    unsafe extern "C" fn error_counter(context: *mut c_void, _name: *const c_char) {
        *(context as *mut c_int) += 1;
    }

    unsafe extern "C" fn test_string_copy(
        dest: *mut c_char,
        src: *const c_char,
        dest_size: usize,
    ) -> c_int {
        let src = CStr::from_ptr(src).to_bytes_with_nul();
        let copy_len = src.len().min(dest_size);
        ptr::copy_nonoverlapping(src.as_ptr() as *const c_char, dest, copy_len);

        if copy_len == dest_size {
            *dest.add(dest_size - 1) = 0;
        }

        1
    }

    unsafe extern "C" fn record_sha1_value(context: *mut c_void, value: c_uint) {
        (*(context as *mut Vec<c_uint>)).push(value);
    }

    unsafe extern "C" fn fatal_unknown(_name: *const c_char) {
        panic!("unexpected unknown mapping field size");
    }

    #[test]
    fn deh_mapping_sets_fields_and_hashes_in_mapping_order() {
        let name_small = CString::new("Small").unwrap();
        let name_medium = CString::new("Medium").unwrap();
        let name_large = CString::new("Large").unwrap();
        let name_label = CString::new("Label").unwrap();
        let unsupported = CString::new("Unsupported").unwrap();
        let unknown = CString::new("Unknown").unwrap();

        let mut base = MappingStruct::default();
        let mut target = MappingStruct::default();
        let null_entry = DehMappingEntry {
            name: ptr::null(),
            location: ptr::null_mut(),
            size: -1,
            is_string: 0,
        };
        let mut entries = [null_entry; 32];

        entries[0] = DehMappingEntry {
            name: name_small.as_ptr(),
            location: &mut base.small as *mut _ as *mut c_void,
            size: 1,
            is_string: 0,
        };
        entries[1] = DehMappingEntry {
            name: name_medium.as_ptr(),
            location: &mut base.medium as *mut _ as *mut c_void,
            size: 2,
            is_string: 0,
        };
        entries[2] = DehMappingEntry {
            name: name_large.as_ptr(),
            location: &mut base.large as *mut _ as *mut c_void,
            size: 4,
            is_string: 0,
        };
        entries[3] = DehMappingEntry {
            name: name_label.as_ptr(),
            location: &mut base.label as *mut _ as *mut c_void,
            size: base.label.len() as c_int,
            is_string: 1,
        };
        entries[4] = DehMappingEntry {
            name: unsupported.as_ptr(),
            location: ptr::null_mut(),
            size: -1,
            is_string: 0,
        };

        let mut mapping = DehMapping {
            base: &mut base as *mut _ as *mut c_void,
            entries,
        };
        let mut diagnostics = 0;

        assert_eq!(
            unsafe {
                cdoom_core::cdoom_rust_deh_set_mapping(
                    &mut diagnostics as *mut _ as *mut c_void,
                    &mut mapping as *mut _ as *mut c_void,
                    &mut target as *mut _ as *mut c_void,
                    name_small.as_ptr() as *mut c_char,
                    0x123,
                    Some(warn_counter),
                    Some(warn_counter),
                    Some(error_counter),
                    Some(error_counter),
                )
            },
            1
        );
        assert_eq!(target.small, 0x23);

        assert_eq!(
            unsafe {
                cdoom_core::cdoom_rust_deh_set_mapping(
                    &mut diagnostics as *mut _ as *mut c_void,
                    &mut mapping as *mut _ as *mut c_void,
                    &mut target as *mut _ as *mut c_void,
                    name_medium.as_ptr() as *mut c_char,
                    0x12345,
                    Some(warn_counter),
                    Some(warn_counter),
                    Some(error_counter),
                    Some(error_counter),
                )
            },
            1
        );
        assert_eq!(target.medium, 0x2345);

        let replacement = CString::new("abcdef").unwrap();
        assert_eq!(
            unsafe {
                cdoom_core::cdoom_rust_deh_set_string_mapping(
                    &mut diagnostics as *mut _ as *mut c_void,
                    &mut mapping as *mut _ as *mut c_void,
                    &mut target as *mut _ as *mut c_void,
                    name_label.as_ptr() as *mut c_char,
                    replacement.as_ptr() as *mut c_char,
                    Some(warn_counter),
                    Some(warn_counter),
                    Some(error_counter),
                    Some(test_string_copy),
                )
            },
            1
        );
        assert_eq!(
            unsafe { CStr::from_ptr(target.label.as_ptr()) }
                .to_str()
                .unwrap(),
            "abcd"
        );

        assert_eq!(
            unsafe {
                cdoom_core::cdoom_rust_deh_set_mapping(
                    &mut diagnostics as *mut _ as *mut c_void,
                    &mut mapping as *mut _ as *mut c_void,
                    &mut target as *mut _ as *mut c_void,
                    unsupported.as_ptr() as *mut c_char,
                    7,
                    Some(warn_counter),
                    Some(warn_counter),
                    Some(error_counter),
                    Some(error_counter),
                )
            },
            0
        );
        assert_eq!(
            unsafe {
                cdoom_core::cdoom_rust_deh_set_mapping(
                    &mut diagnostics as *mut _ as *mut c_void,
                    &mut mapping as *mut _ as *mut c_void,
                    &mut target as *mut _ as *mut c_void,
                    unknown.as_ptr() as *mut c_char,
                    7,
                    Some(warn_counter),
                    Some(warn_counter),
                    Some(error_counter),
                    Some(error_counter),
                )
            },
            0
        );
        assert_eq!(diagnostics, 2);

        let mut values = Vec::new();
        unsafe {
            cdoom_core::cdoom_rust_deh_struct_sha1_sum(
                &mut values as *mut _ as *mut c_void,
                &mut mapping as *mut _ as *mut c_void,
                &mut target as *mut _ as *mut c_void,
                Some(record_sha1_value),
                Some(fatal_unknown),
            );
        }

        assert_eq!(values[0], target.small as c_uint);
        assert_eq!(values[1], target.medium as c_uint);
        assert_eq!(values[2], target.large as c_uint);
        assert_eq!(values.len(), 4);
    }
}
