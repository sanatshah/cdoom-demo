//! Pixel-buffer routines for the `v_video` strangler migration.

pub mod patch;

use patch::PatchPtr;
use std::ptr;
use std::sync::Mutex;

pub const SCREENWIDTH: usize = 320;
pub const SCREENHEIGHT: usize = 200;
const SCREEN_PIXELS: usize = SCREENWIDTH * SCREENHEIGHT;

const BOXTOP: usize = 0;
const BOXBOTTOM: usize = 1;
const BOXLEFT: usize = 2;
const BOXRIGHT: usize = 3;

#[derive(Clone, Copy)]
struct VideoState {
    dest_screen: *mut u8,
    video_buffer: *mut u8,
    dirtybox: *mut i32,
    tinttable: *const u8,
    xlatab: *const u8,
}

// The original C module is global-state based and called from the main thread.
// The mutex gives tests deterministic access without changing those semantics.
unsafe impl Send for VideoState {}

impl VideoState {
    const fn new() -> Self {
        Self {
            dest_screen: ptr::null_mut(),
            video_buffer: ptr::null_mut(),
            dirtybox: ptr::null_mut(),
            tinttable: ptr::null(),
            xlatab: ptr::null(),
        }
    }
}

static STATE: Mutex<VideoState> = Mutex::new(VideoState::new());

enum PatchBlend {
    Opaque,
    Tint,
    AltTint,
    Xla,
    Shadow,
}

pub fn register_buffers(video_buffer: *mut u8, dirtybox: *mut i32) {
    let mut state = STATE.lock().expect("v_video state poisoned");
    state.video_buffer = video_buffer;
    state.dest_screen = video_buffer;
    state.dirtybox = dirtybox;
}

pub fn use_buffer(buffer: *mut u8) {
    let mut state = STATE.lock().expect("v_video state poisoned");
    state.dest_screen = buffer;
}

pub fn restore_buffer() {
    let mut state = STATE.lock().expect("v_video state poisoned");
    state.dest_screen = state.video_buffer;
}

pub fn set_tint_table(table: *const u8) {
    let mut state = STATE.lock().expect("v_video state poisoned");
    state.tinttable = table;
}

pub fn set_xla_table(table: *const u8) {
    let mut state = STATE.lock().expect("v_video state poisoned");
    state.xlatab = table;
}

pub fn mark_rect(x: i32, y: i32, width: i32, height: i32) {
    let state = STATE.lock().expect("v_video state poisoned");
    mark_rect_with_state(&state, x, y, width, height);
}

pub fn copy_rect(
    srcx: i32,
    srcy: i32,
    source: *const u8,
    width: i32,
    height: i32,
    destx: i32,
    desty: i32,
) {
    let state = STATE.lock().expect("v_video state poisoned");
    mark_rect_with_state(&state, destx, desty, width, height);

    let row_width = width as usize;
    let mut src_offset = srcy as usize * SCREENWIDTH + srcx as usize;
    let mut dest_offset = desty as usize * SCREENWIDTH + destx as usize;

    for _ in 0..height {
        // SAFETY: C preserves the original range checks before calling Rust.
        unsafe {
            ptr::copy_nonoverlapping(
                source.add(src_offset),
                state.dest_screen.add(dest_offset),
                row_width,
            );
        }
        src_offset += SCREENWIDTH;
        dest_offset += SCREENWIDTH;
    }
}

pub fn draw_block(x: i32, y: i32, width: i32, height: i32, src: *const u8) {
    let state = STATE.lock().expect("v_video state poisoned");
    mark_rect_with_state(&state, x, y, width, height);

    let row_width = width as usize;
    let mut src_offset = 0usize;
    let mut dest_offset = y as usize * SCREENWIDTH + x as usize;

    for _ in 0..height {
        // SAFETY: C preserves the original range checks before calling Rust.
        unsafe {
            ptr::copy_nonoverlapping(
                src.add(src_offset),
                state.dest_screen.add(dest_offset),
                row_width,
            );
        }
        src_offset += row_width;
        dest_offset += SCREENWIDTH;
    }
}

pub fn draw_raw_screen(raw: *const u8) {
    let state = STATE.lock().expect("v_video state poisoned");
    // SAFETY: The caller provides a full 320x200 source buffer, as in C.
    unsafe {
        ptr::copy_nonoverlapping(raw, state.dest_screen, SCREEN_PIXELS);
    }
}

pub fn draw_filled_box(x: i32, y: i32, width: i32, height: i32, color: i32) {
    let state = STATE.lock().expect("v_video state poisoned");
    let color = color as u8;

    for row in 0..height {
        let offset = (y + row) as usize * SCREENWIDTH + x as usize;
        for col in 0..width {
            // SAFETY: Matches unchecked C writes into I_VideoBuffer.
            unsafe {
                *state.video_buffer.add(offset + col as usize) = color;
            }
        }
    }
}

pub fn draw_horiz_line(x: i32, y: i32, width: i32, color: i32) {
    let state = STATE.lock().expect("v_video state poisoned");
    draw_horiz_line_with_state(&state, x, y, width, color as u8);
}

pub fn draw_vert_line(x: i32, y: i32, height: i32, color: i32) {
    let state = STATE.lock().expect("v_video state poisoned");
    draw_vert_line_with_state(&state, x, y, height, color as u8);
}

pub fn draw_box(x: i32, y: i32, width: i32, height: i32, color: i32) {
    let state = STATE.lock().expect("v_video state poisoned");
    let color = color as u8;
    draw_horiz_line_with_state(&state, x, y, width, color);
    draw_horiz_line_with_state(&state, x, y + height - 1, width, color);
    draw_vert_line_with_state(&state, x, y, height, color);
    draw_vert_line_with_state(&state, x + width - 1, y, height, color);
}

pub fn draw_patch(x: i32, y: i32, patch: PatchPtr) {
    draw_patch_inner(x, y, patch, false, PatchBlend::Opaque);
}

pub fn draw_patch_flipped(x: i32, y: i32, patch: PatchPtr) {
    draw_patch_inner(x, y, patch, true, PatchBlend::Opaque);
}

pub fn draw_tl_patch(x: i32, y: i32, patch: PatchPtr) {
    draw_patch_inner(x, y, patch, false, PatchBlend::Tint);
}

pub fn draw_alt_tl_patch(x: i32, y: i32, patch: PatchPtr) {
    draw_patch_inner(x, y, patch, false, PatchBlend::AltTint);
}

pub fn draw_xla_patch(x: i32, y: i32, patch: PatchPtr) {
    draw_patch_inner(x, y, patch, false, PatchBlend::Xla);
}

pub fn draw_shadowed_patch(x: i32, y: i32, patch: PatchPtr) {
    draw_patch_inner(x, y, patch, false, PatchBlend::Shadow);
}

pub fn copy_region(
    dest: *mut u8,
    dest_pitch: i32,
    src: *const u8,
    src_pitch: i32,
    width: i32,
    height: i32,
) {
    let row_width = width as usize;
    let mut src_offset = 0usize;
    let mut dest_offset = 0usize;

    for _ in 0..height {
        // SAFETY: The C disk icon code passes valid rectangular regions.
        unsafe {
            ptr::copy_nonoverlapping(src.add(src_offset), dest.add(dest_offset), row_width);
        }
        src_offset += src_pitch as usize;
        dest_offset += dest_pitch as usize;
    }
}

pub fn gamma_correct_palette(
    dest_rgb: *mut u8,
    source_rgb: *const u8,
    gamma_table: *const u8,
    gamma_level: i32,
    color_count: i32,
) {
    let gamma_offset = gamma_level as usize * 256;

    for component in 0..(color_count as usize * 3) {
        // SAFETY: C callers provide `color_count * 3` palette bytes and a
        // contiguous five-level, 256-entry gamma table, matching I_SetPalette.
        unsafe {
            let source = usize::from(*source_rgb.add(component));
            *dest_rgb.add(component) = *gamma_table.add(gamma_offset + source) & !3;
        }
    }
}

fn draw_patch_inner(x: i32, y: i32, patch: PatchPtr, flipped: bool, blend: PatchBlend) {
    let state = STATE.lock().expect("v_video state poisoned");
    let draw_x = x - patch::left_offset(patch);
    let draw_y = y - patch::top_offset(patch);
    let width = patch::width(patch);
    let height = patch::height(patch);

    if matches!(blend, PatchBlend::Opaque) {
        mark_rect_with_state(&state, draw_x, draw_y, width, height);
    }

    for col in 0..width {
        let source_col = if flipped { width - 1 - col } else { col };
        let column_offset = patch::column_offset(patch, source_col);
        let mut column = (patch as *const u8).wrapping_add(column_offset);
        let dest_top_offset = draw_y as usize * SCREENWIDTH + (draw_x + col) as usize;
        let shadow_dest_top_offset =
            (draw_y + 2) as usize * SCREENWIDTH + (draw_x + col + 2) as usize;

        loop {
            // SAFETY: Patch columns are byte-coded and terminated by 0xff.
            let topdelta = unsafe { *column };
            if topdelta == 0xff {
                break;
            }

            // SAFETY: `length` follows `topdelta` in the post header.
            let length = unsafe { *column.add(1) };
            // C skips topdelta, length, and an unused padding byte.
            let mut source = column.wrapping_add(3);
            let mut dest_offset = dest_top_offset + usize::from(topdelta) * SCREENWIDTH;
            let mut shadow_dest_offset =
                shadow_dest_top_offset + usize::from(topdelta) * SCREENWIDTH;

            for _ in 0..length {
                // SAFETY: C-side range checks preserve valid destination spans.
                unsafe {
                    let src_pixel = *source;
                    match blend {
                        PatchBlend::Opaque => {
                            *state.dest_screen.add(dest_offset) = src_pixel;
                        }
                        PatchBlend::Tint => {
                            let dest_pixel = *state.dest_screen.add(dest_offset);
                            let index = usize::from(dest_pixel) + (usize::from(src_pixel) << 8);
                            *state.dest_screen.add(dest_offset) = *state.tinttable.add(index);
                        }
                        PatchBlend::AltTint => {
                            let dest_pixel = *state.dest_screen.add(dest_offset);
                            let index = (usize::from(dest_pixel) << 8) + usize::from(src_pixel);
                            *state.dest_screen.add(dest_offset) = *state.tinttable.add(index);
                        }
                        PatchBlend::Xla => {
                            let dest_pixel = *state.dest_screen.add(dest_offset);
                            let index = usize::from(dest_pixel) + (usize::from(src_pixel) << 8);
                            *state.dest_screen.add(dest_offset) = *state.xlatab.add(index);
                        }
                        PatchBlend::Shadow => {
                            let shadow_pixel = *state.dest_screen.add(shadow_dest_offset);
                            let shadow_index = usize::from(shadow_pixel) << 8;
                            *state.dest_screen.add(shadow_dest_offset) =
                                *state.tinttable.add(shadow_index);
                            *state.dest_screen.add(dest_offset) = src_pixel;
                        }
                    }
                }
                source = source.wrapping_add(1);
                dest_offset += SCREENWIDTH;
                shadow_dest_offset += SCREENWIDTH;
            }

            column = column.wrapping_add(usize::from(length) + 4);
        }
    }
}

fn mark_rect_with_state(state: &VideoState, x: i32, y: i32, width: i32, height: i32) {
    if state.dest_screen == state.video_buffer && !state.dirtybox.is_null() {
        add_to_box(state.dirtybox, x, y);
        add_to_box(state.dirtybox, x + width - 1, y + height - 1);
    }
}

fn add_to_box(box_ptr: *mut i32, x: i32, y: i32) {
    // SAFETY: `dirtybox` points at the C `int dirtybox[4]` global.
    unsafe {
        if x < *box_ptr.add(BOXLEFT) {
            *box_ptr.add(BOXLEFT) = x;
        } else if x > *box_ptr.add(BOXRIGHT) {
            *box_ptr.add(BOXRIGHT) = x;
        }

        if y < *box_ptr.add(BOXBOTTOM) {
            *box_ptr.add(BOXBOTTOM) = y;
        } else if y > *box_ptr.add(BOXTOP) {
            *box_ptr.add(BOXTOP) = y;
        }
    }
}

fn draw_horiz_line_with_state(state: &VideoState, x: i32, y: i32, width: i32, color: u8) {
    let offset = y as usize * SCREENWIDTH + x as usize;
    for col in 0..width {
        // SAFETY: Matches unchecked C writes into I_VideoBuffer.
        unsafe {
            *state.video_buffer.add(offset + col as usize) = color;
        }
    }
}

fn draw_vert_line_with_state(state: &VideoState, x: i32, y: i32, height: i32, color: u8) {
    let mut offset = y as usize * SCREENWIDTH + x as usize;
    for _ in 0..height {
        // SAFETY: Matches unchecked C writes into I_VideoBuffer.
        unsafe {
            *state.video_buffer.add(offset) = color;
        }
        offset += SCREENWIDTH;
    }
}
