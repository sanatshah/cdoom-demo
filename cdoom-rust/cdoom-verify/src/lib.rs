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
    use cdoom_core::{bbox, fixed};

    fn c_fixed_mul(a: i32, b: i32) -> i32 {
        (((a as i64) * (b as i64)) >> fixed::FRACBITS) as i32
    }

    fn c_abs(value: i32) -> i32 {
        if value < 0 {
            value.wrapping_neg()
        } else {
            value
        }
    }

    fn c_fixed_div(a: i32, b: i32) -> i32 {
        if b == 0 || ((c_abs(a) >> 14) >= c_abs(b)) {
            if (a ^ b) < 0 {
                i32::MIN
            } else {
                i32::MAX
            }
        } else {
            (((a as i64) << fixed::FRACBITS) / (b as i64)) as i32
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
    fn fixed_mul_matches_c_vectors() {
        let vectors = [
            (fixed::FRACUNIT, fixed::FRACUNIT),
            (fixed::FRACUNIT / 2, fixed::FRACUNIT / 2),
            (-fixed::FRACUNIT, fixed::FRACUNIT / 4),
            (12345, -6789),
            (i32::MAX, 2),
            (i32::MIN + 1, -1),
        ];

        for (a, b) in vectors {
            assert_eq!(fixed::fixed_mul(a, b), c_fixed_mul(a, b));
            assert_eq!(cdoom_core::cdoom_rust_fixed_mul(a, b), c_fixed_mul(a, b));
        }
    }

    #[test]
    fn fixed_div_matches_c_vectors() {
        let vectors = [
            (fixed::FRACUNIT, fixed::FRACUNIT),
            (fixed::FRACUNIT / 2, fixed::FRACUNIT),
            (-fixed::FRACUNIT, fixed::FRACUNIT / 2),
            (12345, -6789),
            (0, fixed::FRACUNIT),
            (fixed::FRACUNIT, 0),
            (i32::MAX, 1),
            (i32::MIN + 1, 1),
        ];

        for (a, b) in vectors {
            assert_eq!(fixed::fixed_div(a, b), c_fixed_div(a, b));
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), c_fixed_div(a, b));
        }
    }

    #[test]
    fn angle_to_fine_index_matches_c_shift() {
        let vectors = [
            0x0000_0000,
            0x0008_0000,
            0x2000_0000,
            0x4000_0000,
            0x8000_0000,
            0xc000_0000,
            0xffff_ffff,
        ];

        for angle in vectors {
            let expected = ((angle >> fixed::ANGLETOFINESHIFT) & fixed::FINEMASK) as i32;
            assert_eq!(fixed::angle_to_fine_index(angle), expected);
            assert_eq!(cdoom_core::cdoom_rust_angle_to_fine_index(angle), expected);
        }
    }

    #[test]
    fn bbox_clear_matches_c_sentinels() {
        let mut box_ = [0; bbox::BOX_LEN];

        bbox::clear_box(&mut box_);

        assert_eq!(box_[bbox::BOXTOP], i32::MIN);
        assert_eq!(box_[bbox::BOXRIGHT], i32::MIN);
        assert_eq!(box_[bbox::BOXBOTTOM], i32::MAX);
        assert_eq!(box_[bbox::BOXLEFT], i32::MAX);
    }

    #[test]
    fn bbox_add_preserves_c_else_if_update_order() {
        let mut box_ = [0; bbox::BOX_LEN];
        bbox::clear_box(&mut box_);

        bbox::add_to_box(&mut box_, 10, -20);
        assert_eq!(box_[bbox::BOXLEFT], 10);
        assert_eq!(box_[bbox::BOXBOTTOM], -20);
        assert_eq!(box_[bbox::BOXRIGHT], i32::MIN);
        assert_eq!(box_[bbox::BOXTOP], i32::MIN);

        bbox::add_to_box(&mut box_, 30, 40);
        assert_eq!(box_[bbox::BOXLEFT], 10);
        assert_eq!(box_[bbox::BOXBOTTOM], -20);
        assert_eq!(box_[bbox::BOXRIGHT], 30);
        assert_eq!(box_[bbox::BOXTOP], 40);

        bbox::add_to_box(&mut box_, -100, 5);
        bbox::add_to_box(&mut box_, 15, -200);
        assert_eq!(
            box_,
            [40, -200, -100, 30],
            "order is [BOXTOP, BOXBOTTOM, BOXLEFT, BOXRIGHT]"
        );
    }

    #[test]
    fn bbox_ffi_updates_c_buffer() {
        let mut box_ = [123; bbox::BOX_LEN];

        cdoom_core::cdoom_rust_m_clear_box(box_.as_mut_ptr());
        assert_eq!(box_, [i32::MIN, i32::MAX, i32::MAX, i32::MIN]);

        cdoom_core::cdoom_rust_m_add_to_box(box_.as_mut_ptr(), -7, 11);
        cdoom_core::cdoom_rust_m_add_to_box(box_.as_mut_ptr(), 3, 19);
        assert_eq!(box_, [19, 11, -7, 3]);
    }
}
