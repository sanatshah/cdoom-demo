//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::path::Path;
use std::ptr;
use std::slice;

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

pub fn mus2mid_from_ffi(input: &[u8]) -> Option<Vec<u8>> {
    let mut output = ptr::null_mut();
    let mut output_len = 0usize;

    // SAFETY: input is a valid Rust slice and output pointers are valid locals.
    let status = unsafe {
        cdoom_core::cdoom_rust_mus2mid(input.as_ptr(), input.len(), &mut output, &mut output_len)
    };

    if status != 0 {
        assert!(output.is_null());
        assert_eq!(output_len, 0);
        return None;
    }

    assert!(!output.is_null());
    // SAFETY: cdoom_rust_mus2mid returned a valid buffer and length.
    let result = unsafe { slice::from_raw_parts(output, output_len).to_vec() };
    // SAFETY: frees the exact buffer returned by cdoom_rust_mus2mid.
    unsafe {
        cdoom_core::cdoom_rust_free_buffer(output, output_len);
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_MUS: &[u8] = &[
        b'M', b'U', b'S', 0x1A, 0x07, 0x00, 0x0E, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x90,
        0xBC, 0x64, 0x0A, 0x80, 0x3C, 0x05, 0x60,
    ];

    const SIMPLE_MIDI: &[u8] = &[
        b'M', b'T', b'h', b'd', 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x00, 0x46, b'M',
        b'T', b'r', b'k', 0x00, 0x00, 0x00, 0x10, 0x00, 0xB0, 0x7B, 0x00, 0x00, 0x90, 0x3C, 0x64,
        0x0A, 0x80, 0x3C, 0x00, 0x05, 0xFF, 0x2F, 0x00,
    ];

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
    fn ffi_mus2mid_matches_expected_bytes() {
        assert_eq!(mus2mid_from_ffi(SIMPLE_MUS).unwrap(), SIMPLE_MIDI);
    }

    #[test]
    fn ffi_mus2mid_rejects_truncated_input() {
        assert!(mus2mid_from_ffi(&SIMPLE_MUS[..6]).is_none());
    }
}
