//! 16.16 fixed-point arithmetic ported from `m_fixed.c`.

pub const FRACBITS: i32 = 16;
pub const FRACUNIT: i32 = 1 << FRACBITS;

pub type Fixed = i32;

fn c_abs(value: Fixed) -> Fixed {
    value.wrapping_abs()
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
