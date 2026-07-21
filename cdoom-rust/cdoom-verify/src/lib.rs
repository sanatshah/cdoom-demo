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

    const REJECT_HEADER_FOR_TEN_LINES: [u8; 16] =
        [64, 0, 0, 0, 0, 0, 0, 0, 50, 0, 0, 0, 0x11, 0x4a, 0x1d, 0];

    fn padded(len: usize, totallines: i32, pad_with_ff: bool) -> Vec<u8> {
        let mut bytes = vec![0xaa; len];
        cdoom_core::p_rejectpad::pad_reject_array(&mut bytes, totallines, pad_with_ff);
        bytes
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
    fn reject_padding_truncates_to_available_bytes() {
        assert_eq!(padded(3, 10, false), vec![64, 0, 0]);
    }

    #[test]
    fn reject_padding_writes_vanilla_zone_header_bytes() {
        assert_eq!(padded(16, 10, false), REJECT_HEADER_FOR_TEN_LINES);
    }

    #[test]
    fn reject_padding_zero_fills_beyond_header_by_default() {
        let mut expected = REJECT_HEADER_FOR_TEN_LINES.to_vec();
        expected.extend([0, 0, 0, 0]);

        assert_eq!(padded(20, 10, false), expected);
    }

    #[test]
    fn reject_padding_can_ff_fill_beyond_header() {
        let mut expected = REJECT_HEADER_FOR_TEN_LINES.to_vec();
        expected.extend([0xff, 0xff, 0xff, 0xff]);

        assert_eq!(padded(20, 10, true), expected);
    }

    #[test]
    fn reject_padding_matches_c_integer_behavior_for_negative_totallines() {
        let expected = [20, 0, 0, 0, 0, 0, 0, 0, 50, 0, 0, 0, 0x11, 0x4a, 0x1d, 0];

        assert_eq!(padded(16, -1, false), expected);
    }

    #[test]
    fn reject_padding_ffi_matches_safe_helper() {
        let mut via_ffi = vec![0xaa; 20];
        let mut via_helper = via_ffi.clone();

        cdoom_core::cdoom_rust_pad_reject_array(
            via_ffi.as_mut_ptr(),
            via_ffi.len() as u32,
            10,
            true,
        );
        cdoom_core::p_rejectpad::pad_reject_array(&mut via_helper, 10, true);

        assert_eq!(via_ffi, via_helper);
    }
}
