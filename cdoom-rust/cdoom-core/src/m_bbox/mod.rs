//! Bounding-box helpers migrated from `m_bbox.c`.

use crate::m_fixed::Fixed;

const BOXTOP: usize = 0;
const BOXBOTTOM: usize = 1;
const BOXLEFT: usize = 2;
const BOXRIGHT: usize = 3;

pub const BOX_LEN: usize = 4;

pub fn clear_box(box_: &mut [Fixed; BOX_LEN]) {
    box_[BOXTOP] = Fixed::MIN;
    box_[BOXRIGHT] = Fixed::MIN;
    box_[BOXBOTTOM] = Fixed::MAX;
    box_[BOXLEFT] = Fixed::MAX;
}

pub fn add_to_box(box_: &mut [Fixed; BOX_LEN], x: Fixed, y: Fixed) {
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
