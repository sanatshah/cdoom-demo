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
    const ANG90: u32 = 0x4000_0000;
    const ANG180: u32 = 0x8000_0000;

    fn c_style_fixed_mul(a: i32, b: i32) -> i32 {
        ((i64::from(a) * i64::from(b)) >> 16) as i32
    }

    fn assert_fixed_mul_parity(a: i32, b: i32) {
        let expected = c_style_fixed_mul(a, b);

        assert_eq!(cdoom_core::m_fixed::fixed_mul(a, b), expected);
        assert_eq!(cdoom_core::cdoom_rust_fixed_mul(a, b), expected);
    }

    fn assert_fixed_div_parity(a: i32, b: i32, expected: i32) {
        assert_eq!(cdoom_core::m_fixed::fixed_div(a, b), expected);
        assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), expected);
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
    fn fixed_mul_matches_c_16_16_semantics() {
        assert_fixed_mul_parity(FRACUNIT, FRACUNIT);
        assert_fixed_mul_parity(FRACUNIT + (FRACUNIT / 2), 2 * FRACUNIT);
        assert_fixed_mul_parity(-(FRACUNIT + (FRACUNIT / 2)), 2 * FRACUNIT);
        assert_fixed_mul_parity(i32::MAX, 2);
        assert_fixed_mul_parity(i32::MIN, -1);
    }

    #[test]
    fn fixed_div_matches_c_16_16_semantics() {
        assert_fixed_div_parity(FRACUNIT, 2 * FRACUNIT, FRACUNIT / 2);
        assert_fixed_div_parity(-3 * FRACUNIT, 2 * FRACUNIT, -(FRACUNIT + (FRACUNIT / 2)));
        assert_fixed_div_parity(i32::MAX, 1, i32::MAX);
        assert_fixed_div_parity(i32::MIN + 1, 1, i32::MIN);
        assert_fixed_div_parity(0, 0, i32::MAX);
        assert_fixed_div_parity(-1, 0, i32::MIN);
    }

    #[test]
    fn angle_to_fine_index_matches_c_shift() {
        assert_eq!(cdoom_core::m_fixed::angle_to_fine_index(0), 0);
        assert_eq!(cdoom_core::m_fixed::angle_to_fine_index(ANG90), 2048);
        assert_eq!(cdoom_core::m_fixed::angle_to_fine_index(ANG180), 4096);
        assert_eq!(cdoom_core::m_fixed::angle_to_fine_index(u32::MAX), 8191);

        assert_eq!(cdoom_core::cdoom_rust_angle_to_fine_index(ANG90), 2048);
        assert_eq!(cdoom_core::cdoom_rust_angle_to_fine_index(u32::MAX), 8191);
    }

    #[test]
    fn bbox_clear_matches_c_sentinels() {
        let mut bbox = [0; cdoom_core::m_bbox::BOX_SIZE];

        cdoom_core::m_bbox::clear_box(&mut bbox);

        assert_eq!(bbox[cdoom_core::m_bbox::BOXTOP], i32::MIN);
        assert_eq!(bbox[cdoom_core::m_bbox::BOXBOTTOM], i32::MAX);
        assert_eq!(bbox[cdoom_core::m_bbox::BOXLEFT], i32::MAX);
        assert_eq!(bbox[cdoom_core::m_bbox::BOXRIGHT], i32::MIN);

        unsafe {
            cdoom_core::cdoom_rust_m_clear_box(bbox.as_mut_ptr());
        }

        assert_eq!(bbox, [i32::MIN, i32::MAX, i32::MAX, i32::MIN]);
    }

    #[test]
    fn bbox_add_to_box_matches_c_update_order() {
        let mut bbox = [0; cdoom_core::m_bbox::BOX_SIZE];

        cdoom_core::m_bbox::clear_box(&mut bbox);
        cdoom_core::m_bbox::add_to_box(&mut bbox, 10, 20);
        assert_eq!(bbox, [i32::MIN, 20, 10, i32::MIN]);

        cdoom_core::m_bbox::add_to_box(&mut bbox, 10, 20);
        assert_eq!(bbox, [20, 20, 10, 10]);

        unsafe {
            cdoom_core::cdoom_rust_m_add_to_box(bbox.as_mut_ptr(), 5, 30);
        }
        assert_eq!(bbox, [30, 20, 5, 10]);

        unsafe {
            cdoom_core::cdoom_rust_m_add_to_box(bbox.as_mut_ptr(), 40, 10);
        }
        assert_eq!(bbox, [30, 10, 5, 40]);
    }
}
