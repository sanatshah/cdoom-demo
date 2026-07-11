//! Fixed-point helpers migrated from `m_fixed.c`.
//!
//! Chocolate Doom uses signed 16.16 fixed-point arithmetic.  These helpers keep
//! the C overflow and saturation behavior that demos and collision code depend
//! on, rather than trying to make the operations more idiomatic.

pub type Fixed = i32;

pub const FRACBITS: i32 = 16;

const FIXED_DIV_OVERFLOW_SHIFT: i32 = 14;

fn c_abs(value: i32) -> i32 {
    if value == i32::MIN {
        i32::MIN
    } else {
        value.abs()
    }
}

pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    (((a as i64) * (b as i64)) >> FRACBITS) as Fixed
}

pub fn fixed_div(a: Fixed, b: Fixed) -> Fixed {
    if (c_abs(a) >> FIXED_DIV_OVERFLOW_SHIFT) >= c_abs(b) {
        if (a ^ b) < 0 {
            i32::MIN
        } else {
            i32::MAX
        }
    } else {
        (((a as i64) << FRACBITS) / (b as i64)) as Fixed
    }
}

pub fn angle_to_fine_index(angle: u32) -> i32 {
    (angle >> 19) as i32
}
