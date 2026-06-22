//! WAD I/O — Rust migration of Chocolate Doom's `w_wad.c`.

pub mod archive;
pub mod c_bindings;
pub mod format;
pub mod globals;
pub mod hash;
pub mod ops;

pub use archive::{LumpEntry, WadArchive};
pub use format::{WadKind, PWAD_LUMP_LIMIT};
pub use hash::{lump_name_hash, lump_name_hash_bytes};
pub use ops::check_num_for_name_in_archive;
