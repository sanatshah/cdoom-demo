//! Core Rust crate for the cdoom strangler-fig migration.
//!
//! New subsystems land here as modules with a thin C ABI in [`ffi`].

mod ffi;
pub mod v_patch;
pub mod v_video;

/// Human-readable library version (crate version plus component name).
pub fn version_string() -> &'static str {
    concat!("cdoom-core ", env!("CARGO_PKG_VERSION"))
}

pub use ffi::{
    cdoom_rust_init, cdoom_rust_v_copy_rect, cdoom_rust_v_copy_region,
    cdoom_rust_v_draw_alt_tl_patch, cdoom_rust_v_draw_block, cdoom_rust_v_draw_box,
    cdoom_rust_v_draw_filled_box, cdoom_rust_v_draw_horiz_line, cdoom_rust_v_draw_patch,
    cdoom_rust_v_draw_patch_flipped, cdoom_rust_v_draw_raw_screen,
    cdoom_rust_v_draw_shadowed_patch, cdoom_rust_v_draw_tl_patch, cdoom_rust_v_draw_vert_line,
    cdoom_rust_v_draw_xla_patch, cdoom_rust_v_mark_rect, cdoom_rust_v_register_buffers,
    cdoom_rust_v_restore_buffer, cdoom_rust_v_set_tint_table, cdoom_rust_v_set_xla_table,
    cdoom_rust_v_use_buffer, cdoom_rust_version,
};
