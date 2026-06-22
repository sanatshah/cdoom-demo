//! Placeholder for `bindgen`-generated Chocolate Doom bindings.
//!
//! Phase A+ will add `build.rs` + `wrapper.h` here. For now this crate re-exports
//! `cdoom-core` so downstream Rust tools can depend on a single sys layer.

pub use cdoom_core::*;
