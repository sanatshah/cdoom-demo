//! Core Rust crate for the cdoom strangler-fig migration.
//!
//! New subsystems land here as modules with a thin C ABI in [`ffi`].

mod ffi;
pub mod net;

/// Human-readable library version (crate version plus component name).
pub fn version_string() -> &'static str {
    concat!("cdoom-core ", env!("CARGO_PKG_VERSION"))
}

pub use ffi::{
    cdoom_rust_init, cdoom_rust_net_expand_tic_num, cdoom_rust_net_parse_protocol_name,
    cdoom_rust_net_protocol_name, cdoom_rust_net_read_int16, cdoom_rust_net_read_int32,
    cdoom_rust_net_read_int8, cdoom_rust_net_read_sint16, cdoom_rust_net_read_sint32,
    cdoom_rust_net_read_sint8, cdoom_rust_net_write_int16, cdoom_rust_net_write_int32,
    cdoom_rust_net_write_int8, cdoom_rust_version,
};
