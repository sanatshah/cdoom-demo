//! Core Rust crate for the cdoom strangler-fig migration.
//!
//! New subsystems land here as modules with a thin C ABI in [`ffi`].

pub mod aes_prng;
mod ffi;
pub mod memio;
pub mod sha1;

/// Human-readable library version (crate version plus component name).
pub fn version_string() -> &'static str {
    concat!("cdoom-core ", env!("CARGO_PKG_VERSION"))
}

pub use ffi::{
    cdoom_rust_init, cdoom_rust_mem_fclose, cdoom_rust_mem_fopen_read, cdoom_rust_mem_fopen_write,
    cdoom_rust_mem_fread, cdoom_rust_mem_fseek, cdoom_rust_mem_ftell, cdoom_rust_mem_fwrite,
    cdoom_rust_mem_get_buf, cdoom_rust_prng_random, cdoom_rust_prng_start, cdoom_rust_prng_stop,
    cdoom_rust_sha1_final, cdoom_rust_sha1_init, cdoom_rust_sha1_update,
    cdoom_rust_sha1_update_int32, cdoom_rust_sha1_update_string, cdoom_rust_version,
};
