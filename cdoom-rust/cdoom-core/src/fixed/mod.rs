//! Chocolate Doom 16.16 fixed-point helpers.
//!
//! These functions intentionally mirror `m_fixed.c` rather than using a more
//! idiomatic numeric abstraction. Demo determinism depends on the exact shifts,
//! casts, and saturation guard.

pub type Fixed = i32;
pub type Angle = u32;

pub const FRACBITS: u32 = 16;
pub const FRACUNIT: Fixed = 1 << FRACBITS;
pub const FINEANGLES: u32 = 8192;
pub const FINEMASK: u32 = FINEANGLES - 1;
pub const ANGLETOFINESHIFT: u32 = 19;

#[inline]
fn c_abs(value: Fixed) -> Fixed {
    if value < 0 {
        value.wrapping_neg()
    } else {
        value
    }
}

pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    (((a as i64) * (b as i64)) >> FRACBITS) as Fixed
}

pub fn fixed_div(a: Fixed, b: Fixed) -> Fixed {
    if b == 0 || ((c_abs(a) >> 14) >= c_abs(b)) {
        if (a ^ b) < 0 {
            i32::MIN
        } else {
            i32::MAX
        }
    } else {
        (((a as i64) << FRACBITS) / (b as i64)) as Fixed
    }
}

pub fn angle_to_fine_index(angle: Angle) -> i32 {
    ((angle >> ANGLETOFINESHIFT) & FINEMASK) as i32
}
