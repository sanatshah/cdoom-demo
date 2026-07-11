//! Fixed-point helpers matching Chocolate Doom's 16.16 C arithmetic.

pub type Fixed = i32;
pub type Angle = u32;

const FRACBITS: i32 = 16;
const ANGLE_TO_FINE_SHIFT: u32 = 19;

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
    angle >> ANGLE_TO_FINE_SHIFT
}
