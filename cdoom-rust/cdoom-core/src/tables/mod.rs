//! Lookup tables migrated from `tables.c`.

pub mod data;

use crate::types::{AngleT, Byte, FixedT};

pub use data::{FINESINE, FINETANGENT, GAMMATABLE_FLAT, TANTOANGLE};

pub const FINEANGLES: usize = 8192;
pub const FINEMASK: usize = FINEANGLES - 1;
pub const ANGLETOFINESHIFT: u32 = 19;

pub const ANG45: AngleT = 0x20000000;
pub const ANG90: AngleT = 0x40000000;
pub const ANG180: AngleT = 0x80000000;
pub const ANG270: AngleT = 0xc0000000;
pub const ANG_MAX: AngleT = 0xffffffff;
pub const ANG1: AngleT = ANG45 / 45;
pub const ANG60: AngleT = ANG180 / 3;
pub const ANG1_X: AngleT = 0x01000000;

pub const SLOPERANGE: u32 = 2048;
pub const SLOPEBITS: u32 = 11;
pub const DBITS: u32 = crate::types::FRACBITS as u32 - SLOPEBITS;

pub const FINECOSINE_OFFSET: usize = FINEANGLES / 4;
pub const GAMMATABLE_ROWS: usize = 5;
pub const GAMMATABLE_COLUMNS: usize = 256;

pub fn finecosine_slice() -> &'static [FixedT] {
    &FINESINE[FINECOSINE_OFFSET..]
}

pub fn gammatable_row(row: usize) -> Option<&'static [Byte]> {
    let start = row.checked_mul(GAMMATABLE_COLUMNS)?;
    let end = start.checked_add(GAMMATABLE_COLUMNS)?;

    GAMMATABLE_FLAT.get(start..end)
}

pub fn slope_div(num: u32, den: u32) -> i32 {
    if den < 512 {
        return SLOPERANGE as i32;
    }

    let ans = num.wrapping_shl(3) / (den >> 8);

    if ans <= SLOPERANGE {
        ans as i32
    } else {
        SLOPERANGE as i32
    }
}
