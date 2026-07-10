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
    use cdoom_core::m_bbox::{BOXBOTTOM, BOXLEFT, BOXRIGHT, BOXTOP};
    use cdoom_core::m_fixed::FRACUNIT;

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
    fn fixed_mul_matches_known_16_16_vectors() {
        let vectors = [
            (0, 3 * FRACUNIT, 0),
            (2 * FRACUNIT, 3 * FRACUNIT, 6 * FRACUNIT),
            (-2 * FRACUNIT, 3 * FRACUNIT, -6 * FRACUNIT),
            (FRACUNIT / 2, FRACUNIT / 4, FRACUNIT / 8),
            (12345, -6789, -1279),
        ];

        for (a, b, expected) in vectors {
            assert_eq!(cdoom_core::m_fixed::fixed_mul(a, b), expected);
            assert_eq!(cdoom_core::cdoom_rust_fixed_mul(a, b), expected);
        }
    }

    #[test]
    fn fixed_div_matches_known_16_16_vectors() {
        let vectors = [
            (6 * FRACUNIT, 2 * FRACUNIT, 3 * FRACUNIT),
            (-6 * FRACUNIT, 2 * FRACUNIT, -3 * FRACUNIT),
            (FRACUNIT / 2, FRACUNIT / 4, 2 * FRACUNIT),
            (1, 2, FRACUNIT / 2),
            ((1 << 14) - 1, 1, 1_073_676_288),
        ];

        for (a, b, expected) in vectors {
            assert_eq!(cdoom_core::m_fixed::fixed_div(a, b), expected);
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), expected);
        }
    }

    #[test]
    fn fixed_div_saturates_like_c() {
        let vectors = [
            (i32::MAX, 1, i32::MAX),
            (i32::MAX, -1, i32::MIN),
            (1 << 14, 1, i32::MAX),
            (-(1 << 14), 1, i32::MIN),
            (0, 0, i32::MAX),
        ];

        for (a, b, expected) in vectors {
            assert_eq!(cdoom_core::m_fixed::fixed_div(a, b), expected);
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), expected);
        }
    }

    #[test]
    fn bbox_clear_and_add_match_c_layout() {
        let mut bbox = [17, 23, 42, 99];

        cdoom_core::m_bbox::clear_box(&mut bbox);
        assert_eq!(bbox[BOXTOP], i32::MIN);
        assert_eq!(bbox[BOXRIGHT], i32::MIN);
        assert_eq!(bbox[BOXBOTTOM], i32::MAX);
        assert_eq!(bbox[BOXLEFT], i32::MAX);

        cdoom_core::m_bbox::add_to_box(&mut bbox, 10, 20);
        assert_eq!(bbox, [20, 20, 10, 10]);

        cdoom_core::m_bbox::add_to_box(&mut bbox, -5, 30);
        assert_eq!(bbox, [30, 20, -5, 10]);

        cdoom_core::m_bbox::add_to_box(&mut bbox, 15, 15);
        assert_eq!(bbox, [30, 15, -5, 15]);
    }

    #[test]
    fn bbox_ffi_updates_c_array_in_place() {
        let mut bbox = [0; 4];

        unsafe {
            cdoom_core::cdoom_rust_m_clear_box(bbox.as_mut_ptr());
            cdoom_core::cdoom_rust_m_add_to_box(bbox.as_mut_ptr(), 100, -50);
            cdoom_core::cdoom_rust_m_add_to_box(bbox.as_mut_ptr(), -25, 75);
        }

        assert_eq!(bbox[BOXTOP], 75);
        assert_eq!(bbox[BOXBOTTOM], -50);
        assert_eq!(bbox[BOXLEFT], -25);
        assert_eq!(bbox[BOXRIGHT], 100);
    }
}
