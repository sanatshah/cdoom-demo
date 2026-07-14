//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::path::Path;

/// Expected Chocolate Doom package version vendored in this repo.
pub const CHOCOLATE_DOOM_VERSION: &str = "3.1.1";

/// Returns `true` when a timedemo baseline can run (binary + IWAD present).
pub fn timedemo_baseline_available(root: &Path) -> bool {
    let binary = root.join("chocolate-doom/build/src/chocolate-doom");
    let wad = root.join("wads/freedoom1.wad");
    binary.is_file() && wad.is_file()
}

/// Reads the exported Rust version string from the C ABI.
pub fn rust_version_from_ffi() -> String {
    let ptr = cdoom_core::cdoom_rust_version();
    assert!(!ptr.is_null());
    // SAFETY: cdoom_rust_version returns a static NUL-terminated string.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::os::raw::c_char;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn ffi_version_matches_crate() {
        assert_eq!(rust_version_from_ffi(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn init_succeeds() {
        assert_eq!(cdoom_core::cdoom_rust_init(), 0);
    }

    #[test]
    fn version_string_is_non_empty() {
        assert!(!cdoom_core::version_string().is_empty());
    }

    #[test]
    fn freedoom_wad_summary_is_stable() {
        let wad = repo_root().join("wads/freedoom1.wad");
        let summary = cdoom_core::w_wad::summarize_file(&wad).expect("freedoom WAD should parse");

        assert!(summary.lump_count > 0);
        assert!(summary.total_size > 0);
        assert_ne!(summary.directory_checksum, 0);
        assert_eq!(&summary.first_name, b"E1M1\0\0\0\0");
    }

    #[test]
    fn wad_summary_ffi_matches_rust_module() {
        let wad = repo_root().join("wads/freedoom1.wad");
        let expected = cdoom_core::w_wad::summarize_file(&wad).expect("freedoom WAD should parse");
        let path =
            CString::new(wad.to_string_lossy().as_bytes()).expect("path must not contain NUL");
        let mut lump_count = 0u32;
        let mut total_size = 0u32;
        let mut directory_checksum = 0u64;
        let mut first_name = [0 as c_char; 9];
        let mut last_name = [0 as c_char; 9];

        let status = unsafe {
            cdoom_core::cdoom_rust_w_wad_file_summary(
                path.as_ptr(),
                &mut lump_count,
                &mut total_size,
                &mut directory_checksum,
                first_name.as_mut_ptr(),
                first_name.len(),
                last_name.as_mut_ptr(),
                last_name.len(),
            )
        };

        assert_eq!(status, 0);
        assert_eq!(lump_count, expected.lump_count);
        assert_eq!(total_size, expected.total_size);
        assert_eq!(directory_checksum, expected.directory_checksum);
        assert_eq!(
            unsafe { CStr::from_ptr(first_name.as_ptr()) }.to_bytes(),
            name_to_cstr_bytes(&expected.first_name)
        );
        assert_eq!(
            unsafe { CStr::from_ptr(last_name.as_ptr()) }.to_bytes(),
            name_to_cstr_bytes(&expected.last_name)
        );
    }

    fn name_to_cstr_bytes(name: &[u8; 8]) -> &[u8] {
        let len = name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name.len());
        &name[..len]
    }
}
