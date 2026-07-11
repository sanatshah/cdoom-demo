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
    const ANG270: u32 = 0xc000_0000;
    const ANG_MAX: u32 = 0xffff_ffff;

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
            (FRACUNIT, FRACUNIT),
            (2 * FRACUNIT, FRACUNIT / 2),
            (-3 * FRACUNIT, FRACUNIT + FRACUNIT / 2),
            (0x1234_5678, -0x0001_0000),
            (i32::MAX, 2),
            (i32::MIN + 1, -2),
        ];

        for (a, b) in cases {
            let expected = (((a as i64) * (b as i64)) >> 16) as i32;
            assert_eq!(cdoom_core::cdoom_rust_fixed_mul(a, b), expected);
        }
    }

    #[test]
    fn fixed_div_matches_c_inputs() {
        let cases = [
            (FRACUNIT, 2 * FRACUNIT, FRACUNIT / 2),
            (3 * FRACUNIT, 2 * FRACUNIT, FRACUNIT + FRACUNIT / 2),
            (-3 * FRACUNIT, 2 * FRACUNIT, -(FRACUNIT + FRACUNIT / 2)),
            (i32::MAX, 1, i32::MAX),
            (-i32::MAX, 1, i32::MIN),
            (i32::MAX, -1, i32::MIN),
            (0, 0, i32::MAX),
        ];

        for (a, b, expected) in cases {
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), expected);
        }
    }

    #[test]
    fn angle_to_fine_index_matches_shift() {
        let cases = [
            (0, 0),
            (ANG90, 2048),
            (ANG180, 4096),
            (ANG270, 6144),
            (ANG_MAX, 8191),
        ];

        for (angle, expected) in cases {
            assert_eq!(cdoom_core::cdoom_rust_angle_to_fine_index(angle), expected);
        }
    }

    #[test]
    fn bbox_clear_matches_c_sentinels() {
        let mut bbox = [0; 4];

        unsafe { cdoom_core::cdoom_rust_m_clear_box(bbox.as_mut_ptr()) };

        assert_eq!(bbox, [i32::MIN, i32::MAX, i32::MAX, i32::MIN]);
    }

    #[test]
    fn bbox_add_matches_c_update_order() {
        let mut bbox = [0; 4];

        unsafe {
            cdoom_core::cdoom_rust_m_clear_box(bbox.as_mut_ptr());
            cdoom_core::cdoom_rust_m_add_to_box(bbox.as_mut_ptr(), 10, 20);
        }

        assert_eq!(bbox, [i32::MIN, 20, 10, i32::MIN]);

        unsafe {
            cdoom_core::cdoom_rust_m_add_to_box(bbox.as_mut_ptr(), 12, 25);
            cdoom_core::cdoom_rust_m_add_to_box(bbox.as_mut_ptr(), -5, -7);
        }

        assert_eq!(bbox, [25, -7, -5, 12]);
    }
}
