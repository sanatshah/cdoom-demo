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
    use cdoom_core::{m_bbox, m_fixed};

    fn reference_fixed_mul(a: m_fixed::Fixed, b: m_fixed::Fixed) -> m_fixed::Fixed {
        (((a as i64) * (b as i64)) >> m_fixed::FRACBITS) as m_fixed::Fixed
    }

    fn reference_fixed_div(a: m_fixed::Fixed, b: m_fixed::Fixed) -> m_fixed::Fixed {
        if (a.wrapping_abs() >> 14) >= b.wrapping_abs() {
            if (a ^ b) < 0 {
                i32::MIN
            } else {
                i32::MAX
            }
        } else {
            (((a as i64) << m_fixed::FRACBITS) / (b as i64)) as m_fixed::Fixed
        }
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
    fn fixed_mul_matches_c_reference_values() {
        let values = [
            (0, m_fixed::FRACUNIT),
            (m_fixed::FRACUNIT, m_fixed::FRACUNIT),
            (3 * m_fixed::FRACUNIT, -2 * m_fixed::FRACUNIT),
            (m_fixed::FRACUNIT / 2, m_fixed::FRACUNIT / 2),
            (i32::MAX, m_fixed::FRACUNIT),
            (i32::MIN, m_fixed::FRACUNIT),
            (i32::MAX, i32::MAX),
            (i32::MIN, i32::MIN),
        ];

        for (a, b) in values {
            let expected = reference_fixed_mul(a, b);
            assert_eq!(m_fixed::fixed_mul(a, b), expected, "FixedMul({a}, {b})");
            assert_eq!(
                cdoom_core::cdoom_rust_fixed_mul(a, b),
                expected,
                "FFI FixedMul({a}, {b})"
            );
        }
    }

    #[test]
    fn fixed_div_matches_c_reference_values() {
        let values = [
            (0, m_fixed::FRACUNIT),
            (m_fixed::FRACUNIT, m_fixed::FRACUNIT),
            (-3 * m_fixed::FRACUNIT, 2 * m_fixed::FRACUNIT),
            (m_fixed::FRACUNIT / 2, 2 * m_fixed::FRACUNIT),
            (i32::MAX, 1),
            (i32::MIN + 1, -1),
            (0, 0),
            (-m_fixed::FRACUNIT, 0),
        ];

        for (a, b) in values {
            let expected = reference_fixed_div(a, b);
            assert_eq!(m_fixed::fixed_div(a, b), expected, "FixedDiv({a}, {b})");
            assert_eq!(
                cdoom_core::cdoom_rust_fixed_div(a, b),
                expected,
                "FFI FixedDiv({a}, {b})"
            );
        }
    }

    #[test]
    fn bbox_helpers_match_c_layout() {
        let mut box_ = [123, 456, 789, 101112];

        m_bbox::clear_box(&mut box_);
        assert_eq!(
            box_,
            [i32::MIN, i32::MAX, i32::MAX, i32::MIN],
            "M_ClearBox sentinel layout"
        );

        m_bbox::add_to_box(&mut box_, 10, -20);
        assert_eq!(box_, [i32::MIN, -20, 10, i32::MIN]);

        m_bbox::add_to_box(&mut box_, -30, 40);
        assert_eq!(box_, [40, -20, -30, i32::MIN]);

        m_bbox::add_to_box(&mut box_, 10, 15);
        assert_eq!(box_, [40, -20, -30, 10]);
    }

    #[test]
    fn bbox_ffi_updates_four_fixed_slots() {
        let mut box_ = [0; m_bbox::BOX_COUNT];

        cdoom_core::cdoom_rust_m_clear_box(box_.as_mut_ptr());
        assert_eq!(box_, [i32::MIN, i32::MAX, i32::MAX, i32::MIN]);

        cdoom_core::cdoom_rust_m_add_to_box(box_.as_mut_ptr(), 7, 9);
        cdoom_core::cdoom_rust_m_add_to_box(box_.as_mut_ptr(), -3, 11);
        cdoom_core::cdoom_rust_m_add_to_box(box_.as_mut_ptr(), 7, 9);
        assert_eq!(box_, [11, 9, -3, 7]);
    }
}
