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
    fn reject_padding_truncates_header_for_short_buffers() {
        let mut bytes = [0xaa; 5];

        cdoom_core::p_rejectpad::pad_reject_array(&mut bytes, 3, false);

        assert_eq!(bytes, [36, 0, 0, 0, 0]);
    }

    #[test]
    fn reject_padding_matches_vanilla_header_words() {
        let mut bytes = [0xaa; 16];

        cdoom_core::p_rejectpad::pad_reject_array(&mut bytes, 3, false);

        assert_eq!(
            bytes,
            [
                36, 0, 0, 0, // Size
                0, 0, 0, 0, // z_zone header word
                50, 0, 0, 0, // PU_LEVEL
                0x11, 0x4a, 0x1d, 0x00, // DOOM_CONST_ZONEID
            ]
        );
    }

    #[test]
    fn reject_padding_fills_long_buffers_with_zero_or_ff() {
        let mut zero_padded = [0xaa; 20];
        let mut ff_padded = [0xaa; 20];

        cdoom_core::p_rejectpad::pad_reject_array(&mut zero_padded, 3, false);
        cdoom_core::p_rejectpad::pad_reject_array(&mut ff_padded, 3, true);

        assert_eq!(&zero_padded[16..], &[0, 0, 0, 0]);
        assert_eq!(&ff_padded[16..], &[0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn reject_padding_preserves_c_negative_totallines_behavior() {
        let mut bytes = [0xaa; 4];

        cdoom_core::p_rejectpad::pad_reject_array(&mut bytes, -1, false);

        assert_eq!(bytes, [20, 0, 0, 0]);
    }

    #[test]
    fn reject_padding_ffi_matches_safe_api() {
        let mut safe_bytes = [0xaa; 20];
        let mut ffi_bytes = [0xaa; 20];

        cdoom_core::p_rejectpad::pad_reject_array(&mut safe_bytes, 31, true);
        unsafe {
            cdoom_core::cdoom_rust_pad_reject_array(
                ffi_bytes.as_mut_ptr(),
                ffi_bytes.len() as u32,
                31,
                1,
            );
        }

        assert_eq!(ffi_bytes, safe_bytes);
    }
}
