//! Bounding-box helpers matching Chocolate Doom's `m_bbox.c`.

use crate::m_fixed::Fixed;

pub const BOXTOP: usize = 0;
pub const BOXBOTTOM: usize = 1;
pub const BOXLEFT: usize = 2;
pub const BOXRIGHT: usize = 3;

pub fn clear_box(bbox: &mut [Fixed; 4]) {
    bbox[BOXTOP] = i32::MIN;
    bbox[BOXRIGHT] = i32::MIN;
    bbox[BOXBOTTOM] = i32::MAX;
    bbox[BOXLEFT] = i32::MAX;
}

pub fn add_to_box(bbox: &mut [Fixed; 4], x: Fixed, y: Fixed) {
    if x < bbox[BOXLEFT] {
        bbox[BOXLEFT] = x;
    } else if x > bbox[BOXRIGHT] {
        bbox[BOXRIGHT] = x;
    }

    if y < bbox[BOXBOTTOM] {
        bbox[BOXBOTTOM] = y;
    } else if y > bbox[BOXTOP] {
        bbox[BOXTOP] = y;
    }
}
