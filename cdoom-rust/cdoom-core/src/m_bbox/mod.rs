//! Bounding-box helpers matching Chocolate Doom's sentinel and update order.

use crate::m_fixed::Fixed;

pub const BOXTOP: usize = 0;
pub const BOXBOTTOM: usize = 1;
pub const BOXLEFT: usize = 2;
pub const BOXRIGHT: usize = 3;

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
