//! Core Rust crate for the cdoom strangler-fig migration.
//!
//! New subsystems land here as modules with a thin C ABI in [`ffi`].

pub mod d_event;
pub mod d_iwad;
pub mod d_mode;
pub mod d_ticcmd;

mod ffi;

/// Human-readable library version (crate version plus component name).
pub fn version_string() -> &'static str {
    concat!("cdoom-core ", env!("CARGO_PKG_VERSION"))
}

pub use ffi::{
    cdoom_rust_game_mission_string, cdoom_rust_game_mode_string, cdoom_rust_get_num_episodes,
    cdoom_rust_init, cdoom_rust_is_episode_map, cdoom_rust_is_iwad_name, cdoom_rust_pop_event,
    cdoom_rust_post_event, cdoom_rust_save_game_iwad_name, cdoom_rust_suggest_game_name,
    cdoom_rust_suggest_iwad_name, cdoom_rust_valid_episode_map, cdoom_rust_valid_game_mode,
    cdoom_rust_valid_game_version, cdoom_rust_version,
};
