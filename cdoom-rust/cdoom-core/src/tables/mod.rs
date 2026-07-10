//! Lookup tables and helpers from `tables.c`.

mod generated;

use crate::types::{Angle, Byte, Fixed, FINEANGLES, SLOPERANGE};

pub const FINETANGENT_LEN: usize = FINEANGLES / 2;
pub const FINESINE_LEN: usize = 5 * FINEANGLES / 4;
pub const FINECOSINE_OFFSET: usize = FINEANGLES / 4;
pub const TANTOANGLE_LEN: usize = SLOPERANGE as usize + 1;
pub const GAMMA_LEVELS: usize = 5;
pub const GAMMA_VALUES: usize = 256;

pub fn finetangent() -> &'static [Fixed; FINETANGENT_LEN] {
    &generated::FINETANGENT
}

pub fn finesine() -> &'static [Fixed; FINESINE_LEN] {
    &generated::FINESINE
}

pub fn finecosine() -> &'static [Fixed] {
    &generated::FINESINE[FINECOSINE_OFFSET..]
}

pub fn tantoangle() -> &'static [Angle; TANTOANGLE_LEN] {
    &generated::TANTOANGLE
}

pub fn gammatable_flat() -> &'static [Byte; GAMMA_LEVELS * GAMMA_VALUES] {
    &generated::GAMMATABLE_FLAT
}

pub fn slope_div(num: u32, den: u32) -> i32 {
    if den < 512 {
        return SLOPERANGE;
    }

    let ans = (num << 3) / (den >> 8);
    if ans <= SLOPERANGE as u32 {
        ans as i32
    } else {
        SLOPERANGE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_lengths_match_c_declarations() {
        assert_eq!(finetangent().len(), 4096);
        assert_eq!(finesine().len(), 10240);
        assert_eq!(
            finecosine().as_ptr(),
            finesine()[FINECOSINE_OFFSET..].as_ptr()
        );
        assert_eq!(finecosine().len(), 8192);
        assert_eq!(tantoangle().len(), 2049);
        assert_eq!(gammatable_flat().len(), 5 * 256);
    }

    #[test]
    fn slope_div_matches_c_edges() {
        assert_eq!(slope_div(0, 0), SLOPERANGE);
        assert_eq!(slope_div(0, 511), SLOPERANGE);
        assert_eq!(slope_div(0, 512), 0);
        assert_eq!(slope_div(2048, 2048), 2048);
        assert_eq!(slope_div(2049, 2048), SLOPERANGE);
        assert_eq!(slope_div(u32::MAX, 512), SLOPERANGE);
    }
}
