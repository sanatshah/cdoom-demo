//! Lookup tables migrated from Chocolate Doom `tables.c`.
//!
//! These constants deliberately mirror the C data and naming so the C side can
//! switch between implementations without changing gameplay behavior.

use crate::types::FixedT;

pub const FINEANGLES: usize = 8192;
pub const FINEMASK: usize = FINEANGLES - 1;
pub const ANGLETOFINESHIFT: u32 = 19;
pub const FINETANGENT_LEN: usize = FINEANGLES / 2;
pub const FINESINE_LEN: usize = 5 * FINEANGLES / 4;
pub const FINECOSINE_OFFSET: usize = FINEANGLES / 4;
pub const SLOPERANGE: u32 = 2048;
pub const SLOPEBITS: u32 = 11;
pub const DBITS: u32 = crate::types::FRACBITS - SLOPEBITS;
pub const TANTOANGLE_LEN: usize = SLOPERANGE as usize + 1;
pub const GAMMA_LEVELS: usize = 5;
pub const GAMMA_COLS: usize = 256;

pub const ANG45: u32 = 0x20000000;
pub const ANG90: u32 = 0x40000000;
pub const ANG180: u32 = 0x80000000;
pub const ANG270: u32 = 0xc0000000;
pub const ANG_MAX: u32 = 0xffffffff;
pub const ANG1: u32 = ANG45 / 45;
pub const ANG60: u32 = ANG180 / 3;
pub const ANG1_X: u32 = 0x01000000;

mod generated;

pub use generated::{FINESINE, FINETANGENT, GAMMATABLE, TANTOANGLE};

pub fn finecosine() -> &'static [FixedT] {
    &FINESINE[FINECOSINE_OFFSET..]
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
