//! Fixed-point helpers ported from `m_fixed.c`.

pub const FRACBITS: i32 = 16;

pub type Fixed = i32;
pub type Angle = u32;

pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    (((a as i64) * (b as i64)) >> FRACBITS) as Fixed
}

pub fn fixed_div(a: Fixed, b: Fixed) -> Fixed {
    let abs_a = a.wrapping_abs();
    let abs_b = b.wrapping_abs();

    if (abs_a >> 14) >= abs_b {
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
    angle >> 19
}
