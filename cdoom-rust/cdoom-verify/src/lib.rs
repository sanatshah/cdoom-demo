//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::path::Path;

use cdoom_core::m_bbox;
use cdoom_core::m_fixed;

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

    const FRACUNIT: i32 = m_fixed::FRACUNIT;

    fn reference_fixed_mul(a: i32, b: i32) -> i32 {
        (((a as i64) * (b as i64)) >> m_fixed::FRACBITS) as i32
    }

    fn c_abs_i32(value: i32) -> i32 {
        if value == i32::MIN {
            i32::MIN
        } else {
            value.abs()
        }
    }

    fn reference_fixed_div(a: i32, b: i32) -> i32 {
        if (c_abs_i32(a) >> 14) >= c_abs_i32(b) {
            if (a ^ b) < 0 {
                i32::MIN
            } else {
                i32::MAX
            }
        } else {
            (((a as i64) << m_fixed::FRACBITS) / (b as i64)) as i32
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
    fn fixed_mul_matches_c_reference_on_fixed_inputs() {
        let cases = [
            (0, 0),
            (FRACUNIT, FRACUNIT),
            (2 * FRACUNIT, 3 * FRACUNIT),
            (-2 * FRACUNIT, 3 * FRACUNIT),
            (i32::MAX, 2),
            (i32::MAX, i32::MAX),
            (i32::MIN, FRACUNIT),
        ];

        for (a, b) in cases {
            assert_eq!(m_fixed::fixed_mul(a, b), reference_fixed_mul(a, b));
            assert_eq!(
                cdoom_core::cdoom_rust_fixed_mul(a, b),
                reference_fixed_mul(a, b)
            );
        }
    }

    #[test]
    fn fixed_div_matches_c_reference_on_fixed_inputs() {
        let cases = [
            (0, FRACUNIT),
            (6 * FRACUNIT, 2 * FRACUNIT),
            (FRACUNIT, 2 * FRACUNIT),
            (-6 * FRACUNIT, 2 * FRACUNIT),
            (i32::MAX, 1),
            (i32::MAX, -1),
            (0, 0),
            (i32::MIN, i32::MIN),
        ];

        for (a, b) in cases {
            assert_eq!(m_fixed::fixed_div(a, b), reference_fixed_div(a, b));
            assert_eq!(
                cdoom_core::cdoom_rust_fixed_div(a, b),
                reference_fixed_div(a, b)
            );
        }
    }

    #[test]
    fn angle_to_fine_index_matches_c_shift() {
        let cases = [
            (0x00000000, 0),
            (0x20000000, 1024),
            (0x40000000, 2048),
            (0x80000000, 4096),
            (0xffffffff, 8191),
        ];

        for (angle, fine_index) in cases {
            assert_eq!(m_fixed::angle_to_fine_index(angle), fine_index);
            assert_eq!(
                cdoom_core::cdoom_rust_angle_to_fine_index(angle),
                fine_index
            );
        }
    }

    #[test]
    fn bbox_helpers_match_c_sentinel_and_update_order() {
        let mut bbox = [0; m_bbox::BOX_SIZE];
        m_bbox::clear_box(&mut bbox);
        assert_eq!(bbox, [i32::MIN, i32::MAX, i32::MAX, i32::MIN]);

        m_bbox::add_to_box(&mut bbox, 10, 20);
        assert_eq!(bbox, [i32::MIN, 20, 10, i32::MIN]);

        m_bbox::add_to_box(&mut bbox, -5, 30);
        assert_eq!(bbox, [30, 20, -5, i32::MIN]);

        m_bbox::add_to_box(&mut bbox, 40, -10);
        assert_eq!(bbox, [30, -10, -5, 40]);
    }

    #[allow(unsafe_code)]
    #[test]
    fn bbox_ffi_matches_c_sentinel_and_update_order() {
        let mut bbox = [0; m_bbox::BOX_SIZE];

        unsafe {
            cdoom_core::cdoom_rust_m_clear_box(bbox.as_mut_ptr());
        }
        assert_eq!(bbox, [i32::MIN, i32::MAX, i32::MAX, i32::MIN]);

        unsafe {
            cdoom_core::cdoom_rust_m_add_to_box(bbox.as_mut_ptr(), 10, 20);
        }
        assert_eq!(bbox, [i32::MIN, 20, 10, i32::MIN]);

        unsafe {
            cdoom_core::cdoom_rust_m_add_to_box(bbox.as_mut_ptr(), -5, 30);
        }
        assert_eq!(bbox, [30, 20, -5, i32::MIN]);

        unsafe {
            cdoom_core::cdoom_rust_m_add_to_box(bbox.as_mut_ptr(), 40, -10);
        }
        assert_eq!(bbox, [30, -10, -5, 40]);
    }
}
