//! Bounding-box helpers ported from `m_bbox.c`.

use crate::m_fixed::Fixed;

const BOXTOP: usize = 0;
const BOXBOTTOM: usize = 1;
const BOXLEFT: usize = 2;
const BOXRIGHT: usize = 3;
const BOX_LEN: usize = 4;

pub fn clear_box(box_values: &mut [Fixed; BOX_LEN]) {
    box_values[BOXTOP] = i32::MIN;
    box_values[BOXRIGHT] = i32::MIN;
    box_values[BOXBOTTOM] = i32::MAX;
    box_values[BOXLEFT] = i32::MAX;
}

pub fn add_to_box(box_values: &mut [Fixed; BOX_LEN], x: Fixed, y: Fixed) {
    if x < box_values[BOXLEFT] {
        box_values[BOXLEFT] = x;
    } else if x > box_values[BOXRIGHT] {
        box_values[BOXRIGHT] = x;
    }

    if y < box_values[BOXBOTTOM] {
        box_values[BOXBOTTOM] = y;
    } else if y > box_values[BOXTOP] {
        box_values[BOXTOP] = y;
    }
}

pub unsafe fn clear_box_ptr(box_values: *mut Fixed) {
    let box_values = unsafe { &mut *(box_values as *mut [Fixed; BOX_LEN]) };
    clear_box(box_values);
}

pub unsafe fn add_to_box_ptr(box_values: *mut Fixed, x: Fixed, y: Fixed) {
    let box_values = unsafe { &mut *(box_values as *mut [Fixed; BOX_LEN]) };
    add_to_box(box_values, x, y);
}
