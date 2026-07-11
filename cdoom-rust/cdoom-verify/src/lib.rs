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
    const ANGLETOFINESHIFT: u32 = 19;
    const BOXTOP: usize = 0;
    const BOXBOTTOM: usize = 1;
    const BOXLEFT: usize = 2;
    const BOXRIGHT: usize = 3;

    fn c_abs(value: i32) -> i32 {
        if value == i32::MIN {
            i32::MIN
        } else {
            value.abs()
        }
    }

    fn c_fixed_mul(a: i32, b: i32) -> i32 {
        (((a as i64) * (b as i64)) >> FRACBITS) as i32
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

    fn c_clear_box(box_coords: &mut [i32; 4]) {
        box_coords[BOXTOP] = i32::MIN;
        box_coords[BOXRIGHT] = i32::MIN;
        box_coords[BOXBOTTOM] = i32::MAX;
        box_coords[BOXLEFT] = i32::MAX;
    }

    fn c_add_to_box(box_coords: &mut [i32; 4], x: i32, y: i32) {
        if x < box_coords[BOXLEFT] {
            box_coords[BOXLEFT] = x;
        } else if x > box_coords[BOXRIGHT] {
            box_coords[BOXRIGHT] = x;
        }

        if y < box_coords[BOXBOTTOM] {
            box_coords[BOXBOTTOM] = y;
        } else if y > box_coords[BOXTOP] {
            box_coords[BOXTOP] = y;
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
    fn fixed_mul_matches_c_fixed_inputs() {
        let values = [
            0,
            1,
            -1,
            FRACUNIT,
            -FRACUNIT,
            FRACUNIT / 2,
            -FRACUNIT / 2,
            12345,
            -98765,
            i32::MAX,
            i32::MIN + 1,
        ];

        for a in values {
            for b in values {
                assert_eq!(cdoom_core::cdoom_rust_fixed_mul(a, b), c_fixed_mul(a, b));
            }
        }
    }

    #[test]
    fn fixed_div_matches_c_fixed_inputs() {
        let cases = [
            (0, 0),
            (FRACUNIT, 0),
            (-FRACUNIT, 0),
            (FRACUNIT, FRACUNIT),
            (-FRACUNIT, FRACUNIT),
            (FRACUNIT, -FRACUNIT),
            (3 * FRACUNIT, 2 * FRACUNIT),
            (-3 * FRACUNIT, 2 * FRACUNIT),
            (FRACUNIT / 2, FRACUNIT),
            (123456789, 23456),
            (-123456789, 23456),
            (i32::MAX, 1),
            (i32::MIN + 1, 1),
        ];

        for (a, b) in cases {
            assert_eq!(cdoom_core::cdoom_rust_fixed_div(a, b), c_fixed_div(a, b));
        }
    }

    #[test]
    fn angle_to_fine_index_matches_c_shift() {
        let angles = [
            0,
            0x2000_0000,
            0x4000_0000,
            0x8000_0000,
            0xc000_0000,
            u32::MAX,
        ];

        for angle in angles {
            assert_eq!(
                cdoom_core::cdoom_rust_angle_to_fine_index(angle),
                (angle >> ANGLETOFINESHIFT) as i32
            );
        }
    }

    #[test]
    fn bbox_clear_and_add_match_c_update_order() {
        let points = [(10, 20), (5, 30), (15, 10), (-100, 50), (200, -40)];
        let mut rust_box = [0; 4];
        let mut c_box = [0; 4];

        cdoom_core::cdoom_rust_m_clear_box(rust_box.as_mut_ptr());
        c_clear_box(&mut c_box);
        assert_eq!(rust_box, c_box);

        cdoom_core::cdoom_rust_m_add_to_box(rust_box.as_mut_ptr(), points[0].0, points[0].1);
        c_add_to_box(&mut c_box, points[0].0, points[0].1);
        assert_eq!(rust_box, [i32::MIN, 20, 10, i32::MIN]);
        assert_eq!(rust_box, c_box);

        for (x, y) in points.into_iter().skip(1) {
            cdoom_core::cdoom_rust_m_add_to_box(rust_box.as_mut_ptr(), x, y);
            c_add_to_box(&mut c_box, x, y);
            assert_eq!(rust_box, c_box);
        }
    }
}
