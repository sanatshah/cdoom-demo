//! Bounding-box helpers matching Chocolate Doom's `m_bbox.c`.

use crate::m_fixed::Fixed;

pub const BOX_TOP: usize = 0;
pub const BOX_BOTTOM: usize = 1;
pub const BOX_LEFT: usize = 2;
pub const BOX_RIGHT: usize = 3;
pub const BOX_COUNT: usize = 4;

/// Resets a bounding box to Chocolate Doom's empty sentinel values.
pub fn clear_box(box_: &mut [Fixed; BOX_COUNT]) {
    box_[BOX_TOP] = i32::MIN;
    box_[BOX_RIGHT] = i32::MIN;
    box_[BOX_BOTTOM] = i32::MAX;
    box_[BOX_LEFT] = i32::MAX;
}

/// Extends a bounding box to include the given point.
pub fn add_to_box(box_: &mut [Fixed; BOX_COUNT], x: Fixed, y: Fixed) {
    if x < box_[BOX_LEFT] {
        box_[BOX_LEFT] = x;
    } else if x > box_[BOX_RIGHT] {
        box_[BOX_RIGHT] = x;
    }

    if y < box_[BOX_BOTTOM] {
        box_[BOX_BOTTOM] = y;
    } else if y > box_[BOX_TOP] {
        box_[BOX_TOP] = y;
    }
}
