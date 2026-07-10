//! Fixed-point math migrated from `m_fixed.c`.
//!
//! Chocolate Doom uses signed 16.16 fixed-point integers. Keep these helpers
//! behaviorally aligned with the C implementation rather than making them
//! mathematically "nicer"; callers depend on the exact overflow and saturation
//! behavior.

pub type Fixed = i32;

const FRACBITS: u32 = 16;
const DIV_OVERFLOW_SHIFT: u32 = 14;

pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    (((a as i64) * (b as i64)) >> FRACBITS) as Fixed
}

pub fn fixed_div(a: Fixed, b: Fixed) -> Fixed {
    if (a.wrapping_abs() >> DIV_OVERFLOW_SHIFT) >= b.wrapping_abs() {
        if (a ^ b) < 0 {
            Fixed::MIN
        } else {
            Fixed::MAX
        }
    } else {
        (((a as i64) << FRACBITS) / (b as i64)) as Fixed
    }
}
