//! Core Rust crate for the cdoom strangler-fig migration.
//!
//! New subsystems land here as modules with a thin C ABI in [`ffi`].

mod ffi;
mod z_zone;

/// Human-readable library version (crate version plus component name).
pub fn version_string() -> &'static str {
    concat!("cdoom-core ", env!("CARGO_PKG_VERSION"))
}

pub use ffi::{
    cdoom_rust_init, cdoom_rust_version, cdoom_rust_z_alloc_count, cdoom_rust_z_change_tag,
    cdoom_rust_z_change_user, cdoom_rust_z_check_heap, cdoom_rust_z_free, cdoom_rust_z_free_count,
    cdoom_rust_z_free_memory, cdoom_rust_z_free_tags, cdoom_rust_z_free_tags_count,
    cdoom_rust_z_init, cdoom_rust_z_malloc, cdoom_rust_z_purge_count, cdoom_rust_z_reset_stats,
    cdoom_rust_z_zone_size,
};
