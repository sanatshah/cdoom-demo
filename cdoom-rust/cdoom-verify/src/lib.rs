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
    use cdoom_core::m_cheat::{CheatSeq, MAX_CHEAT_LEN, MAX_CHEAT_PARAMS};
    use std::ffi::c_void;
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int};

    fn c_char(byte: u8) -> c_char {
        byte as c_char
    }

    fn cheat_sequence(sequence: &[u8], parameter_chars: c_int) -> CheatSeq {
        let mut cheat = CheatSeq {
            sequence: [0; MAX_CHEAT_LEN],
            sequence_len: sequence.len(),
            parameter_chars,
            chars_read: 0,
            param_chars_read: 0,
            parameter_buf: [0; MAX_CHEAT_PARAMS],
        };

        for (index, byte) in sequence.iter().enumerate() {
            cheat.sequence[index] = c_char(*byte);
        }

        cheat
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
    fn m_argv_check_parm_matches_c_bounds_and_case() {
        let argv = [
            Some("chocolate-doom".as_bytes()),
            Some("-IWAD".as_bytes()),
            Some("doom.wad".as_bytes()),
            Some("-skill".as_bytes()),
        ];

        assert_eq!(
            cdoom_core::m_argv::check_parm_with_args(b"-iwad", &argv, 1),
            1
        );
        assert_eq!(
            cdoom_core::m_argv::check_parm_with_args(b"-skill", &argv, 1),
            0
        );
        assert_eq!(
            cdoom_core::m_argv::check_parm_with_args(b"-skill", &argv, 0),
            3
        );
    }

    #[test]
    fn m_argv_ffi_stops_at_null_argv_entry() {
        let check = CString::new("-later").unwrap();
        let arg0 = CString::new("chocolate-doom").unwrap();
        let arg1 = CString::new("-early").unwrap();
        let arg3 = CString::new("-later").unwrap();
        let argv = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            std::ptr::null(),
            arg3.as_ptr(),
        ];

        let result = unsafe {
            cdoom_core::cdoom_rust_m_check_parm_with_args(
                check.as_ptr(),
                argv.len() as c_int,
                argv.as_ptr(),
                0,
            )
        };

        assert_eq!(result, 0);
    }

    #[test]
    fn m_cheat_completes_and_resets_exact_sequence() {
        let mut cheat = cheat_sequence(b"iddqd", 0);

        for key in b"iddq" {
            assert_eq!(
                cdoom_core::m_cheat::check_cheat(&mut cheat, c_char(*key)),
                0
            );
        }

        assert_eq!(
            cdoom_core::m_cheat::check_cheat(&mut cheat, c_char(b'd')),
            1
        );
        assert_eq!(cheat.chars_read, 0);
        assert_eq!(cheat.param_chars_read, 0);
    }

    #[test]
    fn m_cheat_mismatch_resets_to_start() {
        let mut cheat = cheat_sequence(b"idkfa", 0);

        assert_eq!(
            cdoom_core::m_cheat::check_cheat(&mut cheat, c_char(b'i')),
            0
        );
        assert_eq!(
            cdoom_core::m_cheat::check_cheat(&mut cheat, c_char(b'x')),
            0
        );
        assert_eq!(cheat.chars_read, 0);

        for key in b"idkf" {
            assert_eq!(
                cdoom_core::m_cheat::check_cheat(&mut cheat, c_char(*key)),
                0
            );
        }

        assert_eq!(
            cdoom_core::m_cheat::check_cheat(&mut cheat, c_char(b'a')),
            1
        );
    }

    #[test]
    fn m_cheat_ffi_captures_parameter_bytes() {
        let mut cheat = cheat_sequence(b"engage", 2);

        for key in b"engage1" {
            let result = unsafe {
                cdoom_core::cdoom_rust_cht_check_cheat(
                    (&mut cheat as *mut CheatSeq).cast::<c_void>(),
                    c_char(*key),
                )
            };
            assert_eq!(result, 0);
        }

        let result = unsafe {
            cdoom_core::cdoom_rust_cht_check_cheat(
                (&mut cheat as *mut CheatSeq).cast::<c_void>(),
                c_char(b'2'),
            )
        };
        assert_eq!(result, 1);

        let mut buffer = [0; MAX_CHEAT_PARAMS];
        unsafe {
            cdoom_core::cdoom_rust_cht_get_param(
                (&cheat as *const CheatSeq).cast::<c_void>(),
                buffer.as_mut_ptr(),
            );
        }

        assert_eq!(buffer[0], c_char(b'1'));
        assert_eq!(buffer[1], c_char(b'2'));
    }

    #[test]
    fn m_cheat_short_sequence_with_params_matches_vanilla_guard() {
        let mut cheat = cheat_sequence(b"xy", 1);
        cheat.sequence_len = 3;

        assert_eq!(
            cdoom_core::m_cheat::check_cheat(&mut cheat, c_char(b'x')),
            0
        );
        assert_eq!(cheat.chars_read, 0);
        assert_eq!(cheat.param_chars_read, 0);
    }
}
