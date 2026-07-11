//! Fixed-point math and bounding-box helpers migrated from Chocolate Doom.
//!
//! These functions intentionally mirror the C implementation in `m_fixed.c`
//! and `m_bbox.c`, including the update order in `M_AddToBox`.

pub type Fixed = i32;
pub type Angle = u32;

const FRACBITS: i32 = 16;
const FINEANGLES: u32 = 8192;
const FINEMASK: u32 = FINEANGLES - 1;
const ANGLETOFINESHIFT: u32 = 19;

const BOXTOP: usize = 0;
const BOXBOTTOM: usize = 1;
const BOXLEFT: usize = 2;
const BOXRIGHT: usize = 3;

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
    (angle >> ANGLETOFINESHIFT) & FINEMASK
}

pub fn clear_box(box_: &mut [Fixed; 4]) {
    box_[BOXTOP] = i32::MIN;
    box_[BOXRIGHT] = i32::MIN;
    box_[BOXBOTTOM] = i32::MAX;
    box_[BOXLEFT] = i32::MAX;
}

pub fn add_to_box(box_: &mut [Fixed; 4], x: Fixed, y: Fixed) {
    if x < box_[BOXLEFT] {
        box_[BOXLEFT] = x;
    } else if x > box_[BOXRIGHT] {
        box_[BOXRIGHT] = x;
    }

    if y < box_[BOXBOTTOM] {
        box_[BOXBOTTOM] = y;
    } else if y > box_[BOXTOP] {
        box_[BOXTOP] = y;
    }
}
