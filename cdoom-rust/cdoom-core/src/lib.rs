//! Core Rust crate for the cdoom strangler-fig migration.
//!
//! New subsystems land here as modules with a thin C ABI in [`ffi`].

mod ffi;
mod m_config;
mod m_controls;
mod m_misc;

/// Human-readable library version (crate version plus component name).
pub fn version_string() -> &'static str {
    concat!("cdoom-core ", env!("CARGO_PKG_VERSION"))
}

pub use ffi::{
    cdoom_rust_init, cdoom_rust_m_config_clean_config_value, cdoom_rust_m_config_key_from_scan,
    cdoom_rust_m_config_parse_float_parameter, cdoom_rust_m_config_parse_int_parameter,
    cdoom_rust_m_config_scan_from_key, cdoom_rust_m_controls_bind_ints,
    cdoom_rust_m_misc_base_name, cdoom_rust_m_misc_dir_name, cdoom_rust_m_misc_extract_file_base,
    cdoom_rust_m_misc_file_case_exists, cdoom_rust_m_misc_file_exists,
    cdoom_rust_m_misc_force_lowercase, cdoom_rust_m_misc_force_uppercase,
    cdoom_rust_m_misc_make_directory, cdoom_rust_m_misc_normalize_slashes,
    cdoom_rust_m_misc_str_case_str, cdoom_rust_m_misc_str_to_int, cdoom_rust_m_misc_string_concat,
    cdoom_rust_m_misc_string_copy, cdoom_rust_m_misc_string_duplicate,
    cdoom_rust_m_misc_string_ends_with, cdoom_rust_m_misc_string_replace,
    cdoom_rust_m_misc_string_starts_with, cdoom_rust_m_misc_temp_file,
    cdoom_rust_m_misc_write_file, cdoom_rust_version,
};
