//! Bounding-box helpers migrated from `m_bbox.c`.

use crate::m_fixed::Fixed;

pub const BOXTOP: usize = 0;
pub const BOXBOTTOM: usize = 1;
pub const BOXLEFT: usize = 2;
pub const BOXRIGHT: usize = 3;
pub const BOX_COORDS: usize = 4;

pub fn clear_box(box_coords: &mut [Fixed; BOX_COORDS]) {
    box_coords[BOXTOP] = i32::MIN;
    box_coords[BOXRIGHT] = i32::MIN;
    box_coords[BOXBOTTOM] = i32::MAX;
    box_coords[BOXLEFT] = i32::MAX;
}

pub fn add_to_box(box_coords: &mut [Fixed; BOX_COORDS], x: Fixed, y: Fixed) {
    if x < box_coords[BOXLEFT] {
        box_coords[BOXLEFT] = x;
    } else if x > box_coords[BOXRIGHT] {
        box_coords[BOXRIGHT] = x;
    }

    if y < box_coords[BOXBOTTOM] {
        box_coords[BOXBOTTOM] = y;
    } else if y > box_coords[BOXTOP] {
        box_coords[BOXTOP] = y;
    }
}
