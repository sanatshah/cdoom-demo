//! Core Rust crate for the cdoom strangler-fig migration.
//!
//! New subsystems land here as modules with a thin C ABI in [`ffi`].

mod ffi;
pub mod m_bbox;
pub mod m_fixed;

/// Human-readable library version (crate version plus component name).
pub fn version_string() -> &'static str {
    concat!("cdoom-core ", env!("CARGO_PKG_VERSION"))
}

pub use ffi::{
    cdoom_rust_fixed_div, cdoom_rust_fixed_mul, cdoom_rust_init, cdoom_rust_m_add_to_box,
    cdoom_rust_m_clear_box, cdoom_rust_version,
};
