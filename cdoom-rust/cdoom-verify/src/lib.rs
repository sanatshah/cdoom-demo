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
    const FINEANGLES: u32 = 8192;
    const FINEMASK: u32 = FINEANGLES - 1;
    const ANGLETOFINESHIFT: u32 = 19;

    const BOXTOP: usize = 0;
    const BOXBOTTOM: usize = 1;
    const BOXLEFT: usize = 2;
    const BOXRIGHT: usize = 3;

    fn c_fixed_mul(a: i32, b: i32) -> i32 {
        (((a as i64) * (b as i64)) >> FRACBITS) as i32
    }

    fn c_fixed_div(a: i32, b: i32) -> i32 {
        if (a.wrapping_abs() >> 14) >= b.wrapping_abs() {
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
    fn fixed_mul_matches_c_for_fixed_inputs() {
        let cases = [
            (0, 0),
            (FRACUNIT, FRACUNIT),
            (2 * FRACUNIT, -3 * FRACUNIT),
            (12345, 67890),
            (-12345, 67890),
            (i32::MAX, 2),
            (i32::MIN, -1),
        ];

        for (a, b) in cases {
            assert_eq!(cdoom_core::cdoom_rust_fixed_mul(a, b), c_fixed_mul(a, b));
        }
    }

    #[test]
    fn fixed_div_matches_c_for_fixed_inputs() {
        let cases = [
            (FRACUNIT, FRACUNIT),
            (3 * FRACUNIT, 2 * FRACUNIT),
            (-3 * FRACUNIT, 2 * FRACUNIT),
            (12345, 67890),
            (-12345, -67890),
            (0, FRACUNIT),
        ];

        for (a, b) in cases {
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), c_fixed_div(a, b));
        }
    }

    #[test]
    fn fixed_div_saturates_like_c_for_overflow_inputs() {
        let cases = [
            (i32::MAX, 1, i32::MAX),
            (i32::MAX, -1, i32::MIN),
            (-i32::MAX, 1, i32::MIN),
            (-i32::MAX, -1, i32::MAX),
            (FRACUNIT, 0, i32::MAX),
            (-FRACUNIT, 0, i32::MIN),
        ];

        for (a, b, expected) in cases {
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), expected);
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), c_fixed_div(a, b));
        }
    }

    #[test]
    fn angle_to_fine_index_matches_c_shift_and_mask() {
        let cases = [
            0,
            1,
            0x0007_ffff,
            0x0008_0000,
            0x4000_0000,
            0x8000_0000,
            0xc000_0000,
            0xffff_ffff,
        ];

        for angle in cases {
            assert_eq!(
                cdoom_core::cdoom_rust_angle_to_fine_index(angle),
                (angle >> ANGLETOFINESHIFT) & FINEMASK
            );
        }
    }

    #[test]
    fn bbox_helpers_match_c_sentinel_and_update_order() {
        let mut rust_box = [0; 4];
        let mut c_box = [0; 4];

        unsafe {
            cdoom_core::cdoom_rust_m_clear_box(rust_box.as_mut_ptr());
        }
        c_clear_box(&mut c_box);
        assert_eq!(rust_box, c_box);
        assert_eq!(rust_box, [i32::MIN, i32::MAX, i32::MAX, i32::MIN]);

        unsafe {
            cdoom_core::cdoom_rust_m_add_to_box(rust_box.as_mut_ptr(), 10, 20);
        }
        c_add_to_box(&mut c_box, 10, 20);
        assert_eq!(rust_box, c_box);
        assert_eq!(rust_box[BOXRIGHT], i32::MIN);
        assert_eq!(rust_box[BOXTOP], i32::MIN);

        for (x, y) in [(-5, -7), (30, 40), (3, 50), (100, 12)] {
            unsafe {
                cdoom_core::cdoom_rust_m_add_to_box(rust_box.as_mut_ptr(), x, y);
            }
            c_add_to_box(&mut c_box, x, y);
            assert_eq!(rust_box, c_box);
        }

        assert_eq!(rust_box, [50, -7, -5, 100]);
    }
}
