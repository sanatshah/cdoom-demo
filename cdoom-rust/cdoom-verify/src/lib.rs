//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
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
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_void};

    unsafe extern "C" {
        fn free(ptr: *mut c_void);
    }

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
    fn m_misc_string_helpers_match_c_semantics() {
        let mut parsed = 0;
        let input = CString::new(" 0x2a").unwrap();
        assert_eq!(
            unsafe { cdoom_core::cdoom_rust_m_misc_str_to_int(input.as_ptr(), &mut parsed) },
            1
        );
        assert_eq!(parsed, 42);

        let input = CString::new("/tmp//doom\\\\config/").unwrap();
        let mut bytes = input.into_bytes_with_nul();
        unsafe { cdoom_core::cdoom_rust_m_misc_normalize_slashes(bytes.as_mut_ptr().cast()) };
        assert_eq!(
            CStr::from_bytes_until_nul(&bytes).unwrap().to_bytes(),
            b"/tmp/doom/config"
        );

        let haystack = CString::new("Chocolate Doom").unwrap();
        let needle = CString::new("doom").unwrap();
        let found = unsafe {
            cdoom_core::cdoom_rust_m_misc_str_case_str(haystack.as_ptr(), needle.as_ptr())
        };
        assert!(!found.is_null());
        let offset = (found as usize) - (haystack.as_ptr() as usize);
        assert_eq!(offset, "Chocolate ".len());
    }

    #[test]
    fn m_misc_file_helpers_round_trip_bytes() {
        let path = std::env::temp_dir().join(format!(
            "cdoom-rust-m-misc-{}-{}.tmp",
            std::process::id(),
            42
        ));
        let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let payload = b"config-bytes";

        assert_eq!(
            unsafe {
                cdoom_core::cdoom_rust_m_misc_write_file(
                    path_c.as_ptr(),
                    payload.as_ptr().cast(),
                    payload.len() as i32,
                )
            },
            1
        );
        assert_eq!(
            unsafe { cdoom_core::cdoom_rust_m_misc_file_exists(path_c.as_ptr()) },
            1
        );
        assert_eq!(std::fs::read(&path).unwrap(), payload);

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn m_config_key_and_value_round_trip_parity() {
        let key = cdoom_core::cdoom_rust_m_config_key_from_scan(29);
        assert_eq!(key, 0x80 + 0x1d);
        assert_eq!(cdoom_core::cdoom_rust_m_config_scan_from_key(key), 29);

        let right_shift = 0x80 + 0x36;
        assert_eq!(
            cdoom_core::cdoom_rust_m_config_scan_from_key(right_shift),
            54
        );

        let mut value = *b"\"music-packs\"\r\0\0\0\0\0\0\0\0";
        unsafe {
            cdoom_core::cdoom_rust_m_config_clean_config_value(
                value.as_mut_ptr().cast::<c_char>(),
                value.len(),
            )
        };
        assert_eq!(
            CStr::from_bytes_until_nul(&value).unwrap().to_bytes(),
            b"music-packs"
        );

        let int_value = CString::new("077").unwrap();
        assert_eq!(
            unsafe { cdoom_core::cdoom_rust_m_config_parse_int_parameter(int_value.as_ptr()) },
            63
        );

        let float_value = CString::new("-1,5").unwrap();
        assert_eq!(
            unsafe { cdoom_core::cdoom_rust_m_config_parse_float_parameter(float_value.as_ptr()) },
            -1.5
        );
    }

    #[test]
    fn m_misc_allocated_strings_can_be_freed_by_c_allocator() {
        let path = CString::new("/tmp/cdoom.cfg").unwrap();
        let dirname = unsafe { cdoom_core::cdoom_rust_m_misc_dir_name(path.as_ptr()) };
        assert!(!dirname.is_null());
        let dirname_str = unsafe { CStr::from_ptr(dirname) };
        assert_eq!(dirname_str.to_bytes(), b"/tmp");
        unsafe { free(dirname.cast::<c_void>()) };
    }
}
