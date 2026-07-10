//! Fixed-point arithmetic matching Chocolate Doom's `m_fixed.c`.

/// Chocolate Doom's 16.16 fixed-point type.
pub type Fixed = i32;

/// Number of fractional bits in [`Fixed`].
pub const FRACBITS: i32 = 16;

/// One whole unit in 16.16 fixed-point representation.
pub const FRACUNIT: Fixed = 1 << FRACBITS;

/// Multiplies two 16.16 fixed-point values using Chocolate Doom semantics.
pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    (((a as i64) * (b as i64)) >> FRACBITS) as Fixed
}

/// Divides two 16.16 fixed-point values using Chocolate Doom semantics.
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
