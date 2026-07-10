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

    const FRACUNIT: i32 = cdoom_core::m_fixed::FRACUNIT;

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
            (FRACUNIT + (FRACUNIT / 2), 2 * FRACUNIT, 3 * FRACUNIT),
            (-FRACUNIT, 2 * FRACUNIT, -2 * FRACUNIT),
            (12345, -67890, -12789),
            (i32::MAX, 2 * FRACUNIT, -2),
            (i32::MIN, 2 * FRACUNIT, 0),
        ];

        for (a, b, expected) in cases {
            assert_eq!(cdoom_core::m_fixed::fixed_mul(a, b), expected);
            assert_eq!(cdoom_core::cdoom_rust_fixed_mul(a, b), expected);
        }
    }

    #[test]
    fn fixed_div_matches_c_fixed_inputs() {
        let cases = [
            (FRACUNIT, FRACUNIT, FRACUNIT),
            (3 * FRACUNIT, 2 * FRACUNIT, FRACUNIT + (FRACUNIT / 2)),
            (-3 * FRACUNIT, 2 * FRACUNIT, -(FRACUNIT + (FRACUNIT / 2))),
            (FRACUNIT, -2 * FRACUNIT, -(FRACUNIT / 2)),
            (i32::MAX, 1, i32::MAX),
            (-i32::MAX, 1, i32::MIN),
            (0, 0, i32::MAX),
            (FRACUNIT, 0, i32::MAX),
            (-FRACUNIT, 0, i32::MIN),
            (i32::MIN, FRACUNIT, i32::MIN),
        ];

        for (a, b, expected) in cases {
            assert_eq!(cdoom_core::m_fixed::fixed_div(a, b), expected);
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), expected);
        }
    }

    #[test]
    fn bbox_clear_matches_c_sentinels() {
        let mut box_ = [0; cdoom_core::m_bbox::BOX_COORDS];

        cdoom_core::m_bbox::clear_box(&mut box_);

        assert_eq!(box_, [i32::MIN, i32::MAX, i32::MAX, i32::MIN,]);
    }

    #[test]
    fn bbox_add_to_box_preserves_c_else_if_sentinel_behavior() {
        let mut box_ = [0; cdoom_core::m_bbox::BOX_COORDS];

        cdoom_core::m_bbox::clear_box(&mut box_);
        cdoom_core::m_bbox::add_to_box(&mut box_, 10 * FRACUNIT, -3 * FRACUNIT);
        assert_eq!(box_, [i32::MIN, -3 * FRACUNIT, 10 * FRACUNIT, i32::MIN,]);

        cdoom_core::m_bbox::add_to_box(&mut box_, 12 * FRACUNIT, 2 * FRACUNIT);
        assert_eq!(
            box_,
            [2 * FRACUNIT, -3 * FRACUNIT, 10 * FRACUNIT, 12 * FRACUNIT,]
        );

        cdoom_core::m_bbox::add_to_box(&mut box_, 4 * FRACUNIT, -5 * FRACUNIT);
        assert_eq!(
            box_,
            [2 * FRACUNIT, -5 * FRACUNIT, 4 * FRACUNIT, 12 * FRACUNIT,]
        );
    }

    #[test]
    fn bbox_ffi_mutates_c_layout_box() {
        let mut box_ = [0; cdoom_core::m_bbox::BOX_COORDS];

        cdoom_core::cdoom_rust_m_clear_box(box_.as_mut_ptr());
        cdoom_core::cdoom_rust_m_add_to_box(box_.as_mut_ptr(), -FRACUNIT, FRACUNIT);
        cdoom_core::cdoom_rust_m_add_to_box(box_.as_mut_ptr(), 3 * FRACUNIT, 4 * FRACUNIT);

        assert_eq!(box_, [4 * FRACUNIT, FRACUNIT, -FRACUNIT, 3 * FRACUNIT,]);
    }
}
