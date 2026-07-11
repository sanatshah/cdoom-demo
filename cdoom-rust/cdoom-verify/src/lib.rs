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

    const FRACUNIT: i32 = 1 << 16;

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
    fn fixed_mul_matches_fixed_point_inputs() {
        let cases = [
            (FRACUNIT, 2 * FRACUNIT, 2 * FRACUNIT),
            (-(FRACUNIT + FRACUNIT / 2), 2 * FRACUNIT, -3 * FRACUNIT),
            (i32::MAX, FRACUNIT, i32::MAX),
            (i32::MIN, FRACUNIT, i32::MIN),
        ];

        for (a, b, expected) in cases {
            assert_eq!(cdoom_core::m_fixed::fixed_mul(a, b), expected);
            assert_eq!(cdoom_core::cdoom_rust_fixed_mul(a, b), expected);
        }
    }

    #[test]
    fn fixed_div_matches_fractional_and_saturation_inputs() {
        let cases = [
            (3 * FRACUNIT, 2 * FRACUNIT, FRACUNIT + FRACUNIT / 2),
            (-3 * FRACUNIT, 2 * FRACUNIT, -(FRACUNIT + FRACUNIT / 2)),
            (0, 0, i32::MAX),
            (0x4000_0000, 1, i32::MAX),
            (-0x4000_0000, 1, i32::MIN),
            (0x4000_0000, -1, i32::MIN),
        ];

        for (a, b, expected) in cases {
            assert_eq!(cdoom_core::m_fixed::fixed_div(a, b), expected);
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), expected);
        }
    }

    #[test]
    fn angle_to_fine_index_matches_tables_shift() {
        let cases = [
            (0x0000_0000, 0),
            (0x2000_0000, 1024),
            (0x4000_0000, 2048),
            (0x8000_0000, 4096),
            (0xc000_0000, 6144),
            (0xffff_ffff, 8191),
        ];

        for (angle, expected) in cases {
            assert_eq!(cdoom_core::m_fixed::angle_to_fine_index(angle), expected);
            assert_eq!(cdoom_core::cdoom_rust_angle_to_fine_index(angle), expected);
        }
    }

    #[test]
    fn clear_box_matches_c_sentinel_layout() {
        let mut box_ = [0; 4];

        cdoom_core::m_bbox::clear_box(&mut box_);
        assert_eq!(box_, [i32::MIN, i32::MAX, i32::MAX, i32::MIN]);

        let mut ffi_box = [0; 4];
        cdoom_core::cdoom_rust_m_clear_box(ffi_box.as_mut_ptr());
        assert_eq!(ffi_box, box_);
    }

    #[test]
    fn add_to_box_matches_c_else_if_update_order() {
        let mut box_ = [i32::MIN, i32::MAX, i32::MAX, i32::MIN];

        cdoom_core::m_bbox::add_to_box(&mut box_, 10, -20);
        assert_eq!(box_, [i32::MIN, -20, 10, i32::MIN]);

        cdoom_core::m_bbox::add_to_box(&mut box_, 5, -25);
        assert_eq!(box_, [i32::MIN, -25, 5, i32::MIN]);

        cdoom_core::m_bbox::add_to_box(&mut box_, 200, 300);
        assert_eq!(box_, [300, -25, 5, 200]);

        let mut ffi_box = [i32::MIN, i32::MAX, i32::MAX, i32::MIN];
        cdoom_core::cdoom_rust_m_add_to_box(ffi_box.as_mut_ptr(), 10, -20);
        cdoom_core::cdoom_rust_m_add_to_box(ffi_box.as_mut_ptr(), 5, -25);
        cdoom_core::cdoom_rust_m_add_to_box(ffi_box.as_mut_ptr(), 200, 300);
        assert_eq!(ffi_box, box_);
    }
}
