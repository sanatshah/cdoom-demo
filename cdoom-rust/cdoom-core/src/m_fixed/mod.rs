//! Fixed-point helpers matching Chocolate Doom's `m_fixed.c` semantics.

pub type Fixed = i32;
pub type Angle = u32;

pub const FRACBITS: u32 = 16;
pub const FRACUNIT: Fixed = 1 << FRACBITS;
pub const ANGLETOFINESHIFT: u32 = 19;

fn c_abs_i32(value: Fixed) -> Fixed {
    if value == Fixed::MIN {
        Fixed::MIN
    } else {
        value.abs()
    }
}

pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    (((a as i64) * (b as i64)) >> FRACBITS) as Fixed
}

pub fn fixed_div(a: Fixed, b: Fixed) -> Fixed {
    if (c_abs_i32(a) >> 14) >= c_abs_i32(b) {
        if (a ^ b) < 0 {
            Fixed::MIN
        } else {
            Fixed::MAX
        }
    } else {
        (((a as i64) << FRACBITS) / (b as i64)) as Fixed
    }
}

pub fn angle_to_fine_index(angle: Angle) -> u32 {
    angle >> ANGLETOFINESHIFT
}
