//! Fixed-point helpers matching Chocolate Doom's 16.16 C semantics.

pub type Fixed = i32;

const FRACBITS: i32 = 16;

/// Multiplies two 16.16 fixed-point values.
pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    (((a as i64) * (b as i64)) >> FRACBITS) as Fixed
}

/// Divides two 16.16 fixed-point values with Chocolate Doom saturation.
pub fn fixed_div(a: Fixed, b: Fixed) -> Fixed {
    if (a.wrapping_abs() >> 14) >= b.wrapping_abs() {
        if (a ^ b) < 0 {
            i32::MIN
        } else {
            i32::MAX
        }
    } else {
        (((a as i64) << FRACBITS) / (b as i64)) as Fixed
    }
}
