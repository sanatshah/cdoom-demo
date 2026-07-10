//! Core Rust crate for the cdoom strangler-fig migration.
//!
//! New subsystems land here as modules with a thin C ABI in [`ffi`].

pub mod tables;
pub mod types;

mod ffi;

/// Human-readable library version (crate version plus component name).
pub fn version_string() -> &'static str {
    concat!("cdoom-core ", env!("CARGO_PKG_VERSION"))
}

pub use ffi::{
    cdoom_rust_init, cdoom_rust_tables_finecosine, cdoom_rust_tables_finecosine_len,
    cdoom_rust_tables_finesine, cdoom_rust_tables_finesine_len, cdoom_rust_tables_finetangent,
    cdoom_rust_tables_finetangent_len, cdoom_rust_tables_gammatable,
    cdoom_rust_tables_gammatable_len, cdoom_rust_tables_slope_div, cdoom_rust_tables_tantoangle,
    cdoom_rust_tables_tantoangle_len, cdoom_rust_version,
};
