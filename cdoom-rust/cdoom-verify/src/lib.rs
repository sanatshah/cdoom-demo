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
    const BOXTOP: usize = 0;
    const BOXBOTTOM: usize = 1;
    const BOXLEFT: usize = 2;
    const BOXRIGHT: usize = 3;

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
    fn fixed_mul_matches_c_inputs() {
        let cases = [
            (FRACUNIT, FRACUNIT, FRACUNIT),
            (3 * FRACUNIT, -2 * FRACUNIT, -6 * FRACUNIT),
            (0x12345, 0x23456, 164_373),
            (-0x76543, 0x13579, -585_913),
            (i32::MAX, 2 * FRACUNIT, -2),
        ];

        for (a, b, expected) in cases {
            assert_eq!(cdoom_core::m_fixed::fixed_mul(a, b), expected);
            assert_eq!(cdoom_core::cdoom_rust_fixed_mul(a, b), expected);
        }
    }

    #[test]
    fn fixed_div_matches_c_inputs() {
        let cases = [
            (FRACUNIT, FRACUNIT, FRACUNIT),
            (3 * FRACUNIT, 2 * FRACUNIT, 98_304),
            (-3 * FRACUNIT, 2 * FRACUNIT, -98_304),
            (123_456_789, 30_000, 269_695_470),
            (-123_456_789, 30_000, -269_695_470),
        ];

        for (a, b, expected) in cases {
            assert_eq!(cdoom_core::m_fixed::fixed_div(a, b), expected);
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), expected);
        }
    }

    #[test]
    fn fixed_div_saturates_like_c() {
        let cases = [
            (0x4000_0000, 1, i32::MAX),
            (0x4000_0000, -1, i32::MIN),
            (0, 0, i32::MAX),
        ];

        for (a, b, expected) in cases {
            assert_eq!(cdoom_core::m_fixed::fixed_div(a, b), expected);
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), expected);
        }
    }

    #[test]
    fn bbox_clear_matches_c_sentinels() {
        let mut box_ = [0; 4];

        cdoom_core::m_bbox::clear_box(&mut box_);
        assert_eq!(box_, [i32::MIN, i32::MAX, i32::MAX, i32::MIN]);

        let mut ffi_box = [0; 4];
        unsafe {
            cdoom_core::cdoom_rust_m_clear_box(ffi_box.as_mut_ptr());
        }
        assert_eq!(ffi_box, box_);
    }

    #[test]
    fn bbox_add_matches_c_ordering() {
        let mut box_ = [0; 4];
        cdoom_core::m_bbox::clear_box(&mut box_);

        cdoom_core::m_bbox::add_to_box(&mut box_, 10 * FRACUNIT, -3 * FRACUNIT);
        assert_eq!(box_[BOXTOP], i32::MIN);
        assert_eq!(box_[BOXBOTTOM], -3 * FRACUNIT);
        assert_eq!(box_[BOXLEFT], 10 * FRACUNIT);
        assert_eq!(box_[BOXRIGHT], i32::MIN);

        cdoom_core::m_bbox::add_to_box(&mut box_, 4 * FRACUNIT, 5 * FRACUNIT);
        assert_eq!(box_, [5 * FRACUNIT, -3 * FRACUNIT, 4 * FRACUNIT, i32::MIN]);

        cdoom_core::m_bbox::add_to_box(&mut box_, 20 * FRACUNIT, 2 * FRACUNIT);
        assert_eq!(
            box_,
            [5 * FRACUNIT, -3 * FRACUNIT, 4 * FRACUNIT, 20 * FRACUNIT]
        );

        let mut ffi_box = [0; 4];
        unsafe {
            cdoom_core::cdoom_rust_m_clear_box(ffi_box.as_mut_ptr());
            cdoom_core::cdoom_rust_m_add_to_box(ffi_box.as_mut_ptr(), 10 * FRACUNIT, -3 * FRACUNIT);
            cdoom_core::cdoom_rust_m_add_to_box(ffi_box.as_mut_ptr(), 4 * FRACUNIT, 5 * FRACUNIT);
            cdoom_core::cdoom_rust_m_add_to_box(ffi_box.as_mut_ptr(), 20 * FRACUNIT, 2 * FRACUNIT);
        }
        assert_eq!(ffi_box, box_);
    }
}
