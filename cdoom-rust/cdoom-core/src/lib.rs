//! Core Rust crate for the cdoom strangler-fig migration.
//!
//! New subsystems land here as modules with a thin C ABI in [`ffi`].

mod ffi;
mod wad;

/// Human-readable library version (crate version plus component name).
pub fn version_string() -> &'static str {
    concat!("cdoom-core ", env!("CARGO_PKG_VERSION"))
}

pub use ffi::{cdoom_rust_init, cdoom_rust_version};
pub use wad::{
    cdoom_rust_wad_directory_size, cdoom_rust_wad_lump_name_hash,
    cdoom_rust_wad_parse_directory_entry, cdoom_rust_wad_parse_header,
};
