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
    fn reject_padding_writes_vanilla_header_words() {
        let mut actual = [0xcc; 16];

        cdoom_core::p_rejectpad::pad_reject_array(&mut actual, 7, false);

        assert_eq!(
            actual,
            [
                0x34, 0x00, 0x00, 0x00, // computed zone block size
                0x00, 0x00, 0x00, 0x00, // zone block header
                0x32, 0x00, 0x00, 0x00, // PU_LEVEL
                0x11, 0x4a, 0x1d, 0x00, // DOOM_CONST_ZONEID
            ]
        );
    }

    #[test]
    fn reject_padding_truncates_to_requested_length() {
        let mut actual = [0xcc; 3];

        cdoom_core::p_rejectpad::pad_reject_array(&mut actual, 0, false);

        assert_eq!(actual, [0x18, 0x00, 0x00]);
    }

    #[test]
    fn reject_padding_fills_extra_bytes_with_zero_or_ff() {
        let mut zero_padded = [0xcc; 20];
        let mut ff_padded = [0xcc; 20];

        cdoom_core::p_rejectpad::pad_reject_array(&mut zero_padded, 1, false);
        cdoom_core::p_rejectpad::pad_reject_array(&mut ff_padded, 1, true);

        assert_eq!(&zero_padded[..4], &[0x1c, 0x00, 0x00, 0x00]);
        assert_eq!(&zero_padded[16..], &[0x00, 0x00, 0x00, 0x00]);
        assert_eq!(&ff_padded[..4], &[0x1c, 0x00, 0x00, 0x00]);
        assert_eq!(&ff_padded[16..], &[0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn reject_padding_matches_negative_totallines_wrapping() {
        let mut minus_one = [0xcc; 4];
        let mut minus_seven = [0xcc; 4];

        cdoom_core::p_rejectpad::pad_reject_array(&mut minus_one, -1, false);
        cdoom_core::p_rejectpad::pad_reject_array(&mut minus_seven, -7, false);

        assert_eq!(minus_one, [0x14, 0x00, 0x00, 0x00]);
        assert_eq!(minus_seven, [0xfc, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn reject_padding_ffi_routes_to_rust_implementation() {
        let mut actual = [0xcc; 18];

        unsafe {
            cdoom_core::cdoom_rust_pad_reject_array(actual.as_mut_ptr(), actual.len() as u32, 2, 1);
        }

        assert_eq!(&actual[..4], &[0x20, 0x00, 0x00, 0x00]);
        assert_eq!(&actual[16..], &[0xff, 0xff]);
    }
}
