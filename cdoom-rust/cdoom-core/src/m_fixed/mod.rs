//! Fixed-point math helpers migrated from Chocolate Doom's `m_fixed.c`.
//!
//! These routines intentionally preserve the C implementation's 16.16
//! arithmetic, including signed narrowing behavior after 64-bit operations.

pub type Fixed = i32;
pub type Angle = u32;

pub const FRACBITS: i32 = 16;
pub const FINEANGLES: u32 = 8192;
pub const ANGLETOFINESHIFT: u32 = 19;

pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    ((i64::from(a) * i64::from(b)) >> FRACBITS) as Fixed
}

pub fn fixed_div(a: Fixed, b: Fixed) -> Fixed {
    if ((a.wrapping_abs() >> 14) >= b.wrapping_abs()) || b == 0 {
        if (a ^ b) < 0 {
            i32::MIN
        } else {
            i32::MAX
        }
    } else {
        ((i64::from(a) << FRACBITS) / i64::from(b)) as Fixed
    }
}

pub fn angle_to_fine_index(angle: Angle) -> u32 {
    angle >> ANGLETOFINESHIFT
}
