//! Core Rust crate for the cdoom strangler-fig migration.
//!
//! New subsystems land here as modules with a thin C ABI in [`ffi`].

pub mod deh;
mod ffi;

/// Human-readable library version (crate version plus component name).
pub fn version_string() -> &'static str {
    concat!("cdoom-core ", env!("CARGO_PKG_VERSION"))
}

pub use ffi::{
    cdoom_rust_deh_add_string_replacement, cdoom_rust_deh_get_char,
    cdoom_rust_deh_max_string_length, cdoom_rust_deh_parse_assignment,
    cdoom_rust_deh_parse_context, cdoom_rust_deh_read_line, cdoom_rust_deh_set_mapping,
    cdoom_rust_deh_set_string_mapping, cdoom_rust_deh_string, cdoom_rust_deh_struct_sha1_sum,
    cdoom_rust_deh_text_start, cdoom_rust_init, cdoom_rust_version,
};
