//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::path::Path;

const FRACUNIT: i32 = 1 << 16;
const BOXTOP: usize = 0;
const BOXBOTTOM: usize = 1;
const BOXLEFT: usize = 2;
const BOXRIGHT: usize = 3;

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
    fn fixed_mul_matches_c_fixed_inputs() {
        let cases = [
            (FRACUNIT, FRACUNIT, FRACUNIT),
            (2 * FRACUNIT, -3 * FRACUNIT, -6 * FRACUNIT),
            (FRACUNIT / 2, FRACUNIT / 4, FRACUNIT / 8),
            (i32::MAX, 2 * FRACUNIT, -2),
            (i32::MIN, FRACUNIT / 2, i32::MIN / 2),
        ];

        for (a, b, expected) in cases {
            assert_eq!(cdoom_core::cdoom_rust_fixed_mul(a, b), expected);
        }
    }

    #[test]
    fn fixed_div_matches_c_regular_inputs() {
        let cases = [
            (FRACUNIT, 2 * FRACUNIT, FRACUNIT / 2),
            (-3 * FRACUNIT, 2 * FRACUNIT, -(3 * FRACUNIT / 2)),
            (FRACUNIT / 8, FRACUNIT / 2, FRACUNIT / 4),
        ];

        for (a, b, expected) in cases {
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), expected);
        }
    }

    #[test]
    fn fixed_div_matches_c_saturation_inputs() {
        let cases = [
            (i32::MAX, 1, i32::MAX),
            (i32::MAX, -1, i32::MIN),
            (-i32::MAX, 1, i32::MIN),
            (0, 0, i32::MAX),
        ];

        for (a, b, expected) in cases {
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), expected);
        }
    }

    #[test]
    fn bbox_clear_matches_c_sentinels() {
        let mut box_ = [0; 4];

        unsafe {
            cdoom_core::cdoom_rust_m_clear_box(box_.as_mut_ptr());
        }

        assert_eq!(box_[BOXTOP], i32::MIN);
        assert_eq!(box_[BOXBOTTOM], i32::MAX);
        assert_eq!(box_[BOXLEFT], i32::MAX);
        assert_eq!(box_[BOXRIGHT], i32::MIN);
    }

    #[test]
    fn bbox_add_matches_c_else_if_sentinel_behavior() {
        let mut box_ = [0; 4];

        unsafe {
            cdoom_core::cdoom_rust_m_clear_box(box_.as_mut_ptr());
            cdoom_core::cdoom_rust_m_add_to_box(box_.as_mut_ptr(), 10, 20);
        }

        assert_eq!(box_[BOXTOP], i32::MIN);
        assert_eq!(box_[BOXBOTTOM], 20);
        assert_eq!(box_[BOXLEFT], 10);
        assert_eq!(box_[BOXRIGHT], i32::MIN);

        unsafe {
            cdoom_core::cdoom_rust_m_add_to_box(box_.as_mut_ptr(), 5, 15);
            cdoom_core::cdoom_rust_m_add_to_box(box_.as_mut_ptr(), 30, 40);
        }

        assert_eq!(box_[BOXTOP], 40);
        assert_eq!(box_[BOXBOTTOM], 15);
        assert_eq!(box_[BOXLEFT], 5);
        assert_eq!(box_[BOXRIGHT], 30);
    }
}
