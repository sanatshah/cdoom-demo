//! Fixed-point helpers matching Chocolate Doom's 16.16 arithmetic.

pub type Fixed = i32;
pub type Angle = u32;

pub const FRACBITS: i32 = 16;
pub const FRACUNIT: Fixed = 1 << FRACBITS;

pub const ANG45: Angle = 0x20000000;
pub const ANG90: Angle = 0x40000000;
pub const ANG180: Angle = 0x80000000;
pub const ANG270: Angle = 0xc0000000;
pub const ANG_MAX: Angle = 0xffffffff;
pub const ANG1: Angle = ANG45 / 45;
pub const ANG60: Angle = ANG180 / 3;
pub const ANGLETOFINESHIFT: u32 = 19;

pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    (((a as i64) * (b as i64)) >> FRACBITS) as Fixed
}

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

pub fn angle_to_fine_index(angle: Angle) -> u32 {
    angle >> ANGLETOFINESHIFT
}
