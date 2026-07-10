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

    const FRACBITS: i32 = 16;
    const FRACUNIT: i32 = 1 << FRACBITS;
    const BOXTOP: usize = 0;
    const BOXBOTTOM: usize = 1;
    const BOXLEFT: usize = 2;
    const BOXRIGHT: usize = 3;

    fn c_fixed_mul(a: i32, b: i32) -> i32 {
        (((a as i64) * (b as i64)) >> FRACBITS) as i32
    }

    fn c_abs(value: i32) -> i32 {
        value.wrapping_abs()
    }

    fn c_fixed_div(a: i32, b: i32) -> i32 {
        if (c_abs(a) >> 14) >= c_abs(b) {
            if (a ^ b) < 0 {
                i32::MIN
            } else {
                i32::MAX
            }
        } else {
            (((a as i64) << FRACBITS) / (b as i64)) as i32
        }
    }

    fn c_clear_box(box_: &mut [i32; 4]) {
        box_[BOXTOP] = i32::MIN;
        box_[BOXRIGHT] = i32::MIN;
        box_[BOXBOTTOM] = i32::MAX;
        box_[BOXLEFT] = i32::MAX;
    }

    fn c_add_to_box(box_: &mut [i32; 4], x: i32, y: i32) {
        if x < box_[BOXLEFT] {
            box_[BOXLEFT] = x;
        } else if x > box_[BOXRIGHT] {
            box_[BOXRIGHT] = x;
        }

        if y < box_[BOXBOTTOM] {
            box_[BOXBOTTOM] = y;
        } else if y > box_[BOXTOP] {
            box_[BOXTOP] = y;
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
    fn fixed_and_bbox_constants_match_c_headers() {
        assert_eq!(cdoom_core::m_fixed::FRACBITS, FRACBITS);
        assert_eq!(cdoom_core::m_fixed::FRACUNIT, FRACUNIT);
        assert_eq!(cdoom_core::m_bbox::BOXTOP, BOXTOP);
        assert_eq!(cdoom_core::m_bbox::BOXBOTTOM, BOXBOTTOM);
        assert_eq!(cdoom_core::m_bbox::BOXLEFT, BOXLEFT);
        assert_eq!(cdoom_core::m_bbox::BOXRIGHT, BOXRIGHT);
    }

    #[test]
    fn fixed_mul_matches_c_inputs() {
        let cases = [
            (0, FRACUNIT),
            (FRACUNIT, FRACUNIT),
            (2 * FRACUNIT, 3 * FRACUNIT),
            (-2 * FRACUNIT, 3 * FRACUNIT),
            (FRACUNIT / 2, FRACUNIT / 4),
            (12345, -67890),
            (i32::MAX, 2),
            (i32::MIN + 1, -3),
        ];

        for (a, b) in cases {
            assert_eq!(cdoom_core::cdoom_rust_fixed_mul(a, b), c_fixed_mul(a, b));
        }
    }

    #[test]
    fn fixed_div_matches_c_inputs_and_saturation() {
        let cases = [
            (FRACUNIT, FRACUNIT),
            (3 * FRACUNIT, 2 * FRACUNIT),
            (-3 * FRACUNIT, 2 * FRACUNIT),
            (FRACUNIT / 4, FRACUNIT / 2),
            (123456, -7890),
            (i32::MAX, 1),
            (i32::MIN + 1, 1),
        ];

        for (a, b) in cases {
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), c_fixed_div(a, b));
        }
    }

    #[test]
    fn angle_helpers_match_tables_constants() {
        assert_eq!(cdoom_core::m_fixed::ANG45, 0x20000000);
        assert_eq!(cdoom_core::m_fixed::ANG90, 0x40000000);
        assert_eq!(cdoom_core::m_fixed::ANG180, 0x80000000);
        assert_eq!(cdoom_core::m_fixed::ANG270, 0xc0000000);
        assert_eq!(cdoom_core::m_fixed::ANG_MAX, 0xffffffff);
        assert_eq!(
            cdoom_core::cdoom_rust_angle_to_fine_index(cdoom_core::m_fixed::ANG90),
            cdoom_core::m_fixed::ANG90 >> 19
        );
    }

    #[test]
    fn bbox_matches_c_sentinel_and_update_order() {
        let mut expected = [0; 4];
        let mut actual = [0; 4];

        c_clear_box(&mut expected);
        cdoom_core::cdoom_rust_m_clear_box(actual.as_mut_ptr());
        assert_eq!(actual, expected);

        let points = [(128, -64), (64, -128), (256, 32), (-32, 16), (512, -512)];

        for (x, y) in points {
            c_add_to_box(&mut expected, x, y);
            cdoom_core::cdoom_rust_m_add_to_box(actual.as_mut_ptr(), x, y);
            assert_eq!(actual, expected);
        }

        assert_eq!(actual[BOXTOP], 32);
        assert_eq!(actual[BOXBOTTOM], -512);
        assert_eq!(actual[BOXLEFT], -32);
        assert_eq!(actual[BOXRIGHT], 512);
    }
}
