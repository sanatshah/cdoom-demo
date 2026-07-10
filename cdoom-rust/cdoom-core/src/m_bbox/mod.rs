//! Bounding-box helpers ported from `m_bbox.c`.

use crate::m_fixed::Fixed;

pub const BOXTOP: usize = 0;
pub const BOXBOTTOM: usize = 1;
pub const BOXLEFT: usize = 2;
pub const BOXRIGHT: usize = 3;
pub const BOX_COORDS: usize = 4;

pub fn clear_box(box_: &mut [Fixed; BOX_COORDS]) {
    box_[BOXTOP] = i32::MIN;
    box_[BOXRIGHT] = i32::MIN;
    box_[BOXBOTTOM] = i32::MAX;
    box_[BOXLEFT] = i32::MAX;
}

pub fn add_to_box(box_: &mut [Fixed; BOX_COORDS], x: Fixed, y: Fixed) {
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
