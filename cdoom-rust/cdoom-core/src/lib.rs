//! Core Rust crate for the cdoom strangler-fig migration.
//!
//! New subsystems land here as modules with a thin C ABI in [`ffi`].

mod ffi;
pub mod m_argv;
pub mod m_cheat;

/// Human-readable library version (crate version plus component name).
pub fn version_string() -> &'static str {
    concat!("cdoom-core ", env!("CARGO_PKG_VERSION"))
}

pub use ffi::{cdoom_rust_init, cdoom_rust_version};

pub use m_argv::{
    cdoom_rust_m_check_parm, cdoom_rust_m_check_parm_with_args, cdoom_rust_m_find_response_file,
    cdoom_rust_m_get_executable_name, cdoom_rust_m_parm_exists, cdoom_rust_m_set_exe_dir, exedir,
    myargc, myargv,
};
pub use m_cheat::{
    cdoom_rust_cht_check_cheat, cdoom_rust_cht_get_param, check_cheat, get_param, CheatSeq,
    MAX_CHEAT_LEN, MAX_CHEAT_PARAMS,
};
