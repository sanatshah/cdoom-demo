//! Core Rust crate for the cdoom strangler-fig migration.
//!
//! New subsystems land here as modules with a thin C ABI in [`ffi`].

mod ffi;

/// Human-readable library version (crate version plus component name).
pub fn version_string() -> &'static str {
    concat!("cdoom-core ", env!("CARGO_PKG_VERSION"))
}

pub use ffi::{cdoom_rust_init, cdoom_rust_version};
