//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::v_patch::PatchPtr;
use crate::v_video;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::OnceLock;

static VERSION: OnceLock<CString> = OnceLock::new();

fn version_cstr() -> &'static CString {
    VERSION.get_or_init(|| {
        CString::new(env!("CARGO_PKG_VERSION")).expect("version must not contain NUL")
    })
}

/// Returns a pointer to a static, NUL-terminated version string.
///
/// # Safety
///
/// The returned pointer is valid for the process lifetime and must not be freed.
#[no_mangle]
pub extern "C" fn cdoom_rust_version() -> *const c_char {
    version_cstr().as_ptr()
}

/// One-time initialization hook for future Rust subsystems.
///
/// Returns `0` on success. Reserved for later migration phases.
#[no_mangle]
pub extern "C" fn cdoom_rust_init() -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_register_buffers(video_buffer: *mut u8, dirtybox: *mut i32) {
    v_video::register_buffers(video_buffer, dirtybox);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_use_buffer(buffer: *mut u8) {
    v_video::use_buffer(buffer);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_restore_buffer() {
    v_video::restore_buffer();
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_set_tint_table(table: *const u8) {
    v_video::set_tint_table(table);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_set_xla_table(table: *const u8) {
    v_video::set_xla_table(table);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_mark_rect(x: i32, y: i32, width: i32, height: i32) {
    v_video::mark_rect(x, y, width, height);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_copy_rect(
    srcx: i32,
    srcy: i32,
    source: *const u8,
    width: i32,
    height: i32,
    destx: i32,
    desty: i32,
) {
    v_video::copy_rect(srcx, srcy, source, width, height, destx, desty);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_draw_block(x: i32, y: i32, width: i32, height: i32, src: *const u8) {
    v_video::draw_block(x, y, width, height, src);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_draw_raw_screen(raw: *const u8) {
    v_video::draw_raw_screen(raw);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_draw_filled_box(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: i32,
) {
    v_video::draw_filled_box(x, y, width, height, color);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_draw_horiz_line(x: i32, y: i32, width: i32, color: i32) {
    v_video::draw_horiz_line(x, y, width, color);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_draw_vert_line(x: i32, y: i32, height: i32, color: i32) {
    v_video::draw_vert_line(x, y, height, color);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_draw_box(x: i32, y: i32, width: i32, height: i32, color: i32) {
    v_video::draw_box(x, y, width, height, color);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_draw_patch(x: i32, y: i32, patch: PatchPtr) {
    v_video::draw_patch(x, y, patch);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_draw_patch_flipped(x: i32, y: i32, patch: PatchPtr) {
    v_video::draw_patch_flipped(x, y, patch);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_draw_tl_patch(x: i32, y: i32, patch: PatchPtr) {
    v_video::draw_tl_patch(x, y, patch);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_draw_alt_tl_patch(x: i32, y: i32, patch: PatchPtr) {
    v_video::draw_alt_tl_patch(x, y, patch);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_draw_xla_patch(x: i32, y: i32, patch: PatchPtr) {
    v_video::draw_xla_patch(x, y, patch);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_draw_shadowed_patch(x: i32, y: i32, patch: PatchPtr) {
    v_video::draw_shadowed_patch(x, y, patch);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_v_copy_region(
    dest: *mut u8,
    dest_pitch: i32,
    src: *const u8,
    src_pitch: i32,
    width: i32,
    height: i32,
) {
    v_video::copy_region(dest, dest_pitch, src, src_pitch, width, height);
}
