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
    use cdoom_core::m_argv::{parse_response_args, ResponseParseError};
    use cdoom_core::m_cheat::{CheatSeq, MAX_CHEAT_LEN, MAX_CHEAT_PARAMS};
    use std::ffi::CString;
    use std::os::raw::c_char;

    fn cstring_args(args: &[&str]) -> (Vec<CString>, Vec<*mut c_char>) {
        let strings = args
            .iter()
            .map(|arg| CString::new(*arg).unwrap())
            .collect::<Vec<_>>();
        let argv = strings
            .iter()
            .map(|arg| arg.as_ptr() as *mut c_char)
            .collect::<Vec<_>>();

        (strings, argv)
    }

    fn cheat(sequence: &str, sequence_len: usize, parameter_chars: i32) -> CheatSeq {
        let mut chars = [0 as c_char; MAX_CHEAT_LEN];

        for (index, byte) in sequence.bytes().enumerate() {
            chars[index] = byte as c_char;
        }

        CheatSeq {
            sequence: chars,
            sequence_len,
            parameter_chars,
            chars_read: 0,
            param_chars_read: 0,
            parameter_buf: [0 as c_char; MAX_CHEAT_PARAMS],
        }
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
    fn m_argv_check_parm_matches_c_bounds_and_case_behavior() {
        let (_strings, mut argv) = cstring_args(&["chocolate-doom", "-IWAD", "doom.wad", "-file"]);
        let check = CString::new("-iwad").unwrap();
        let missing_arg_check = CString::new("-file").unwrap();

        assert_eq!(
            unsafe {
                cdoom_core::cdoom_rust_m_check_parm_with_args(
                    argv.len() as i32,
                    argv.as_mut_ptr(),
                    check.as_ptr(),
                    1,
                )
            },
            1
        );
        assert_eq!(
            unsafe {
                cdoom_core::cdoom_rust_m_check_parm_with_args(
                    argv.len() as i32,
                    argv.as_mut_ptr(),
                    missing_arg_check.as_ptr(),
                    1,
                )
            },
            0
        );
        assert_eq!(
            unsafe {
                cdoom_core::cdoom_rust_m_parm_exists(
                    argv.len() as i32,
                    argv.as_mut_ptr(),
                    check.as_ptr(),
                )
            },
            1
        );
    }

    #[test]
    fn m_argv_check_parm_stops_at_null_response_file_gap() {
        let (_strings, mut argv) = cstring_args(&["chocolate-doom", "-width", "800", "-iwad"]);
        argv[2] = std::ptr::null_mut();
        let width = CString::new("-width").unwrap();
        let iwad = CString::new("-iwad").unwrap();

        assert_eq!(
            unsafe {
                cdoom_core::cdoom_rust_m_check_parm_with_args(
                    argv.len() as i32,
                    argv.as_mut_ptr(),
                    width.as_ptr(),
                    1,
                )
            },
            1
        );
        assert_eq!(
            unsafe {
                cdoom_core::cdoom_rust_m_check_parm(
                    argv.len() as i32,
                    argv.as_mut_ptr(),
                    iwad.as_ptr(),
                )
            },
            0
        );
    }

    #[test]
    fn m_argv_response_parser_matches_c_quote_and_space_rules() {
        let parsed = parse_response_args(b" -iwad \"doom two.wad\"\n-file\tfoo.wad  ").unwrap();
        let strings = parsed
            .iter()
            .map(|arg| String::from_utf8(arg.clone()).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(strings, ["-iwad", "doom two.wad", "-file", "foo.wad"]);
        assert_eq!(
            parse_response_args(b"\"unterminated\n").unwrap_err(),
            ResponseParseError::UnclosedQuotes
        );
    }

    #[test]
    fn m_cheat_matches_sequence_and_resets_after_success() {
        let mut cheat = cheat("iddqd", "iddqd".len(), 0);

        for key in "iddq".bytes() {
            assert_eq!(
                unsafe { cdoom_core::cdoom_rust_cht_check_cheat(&mut cheat, key as c_char) },
                0
            );
        }

        assert_eq!(
            unsafe { cdoom_core::cdoom_rust_cht_check_cheat(&mut cheat, b'd' as c_char) },
            1
        );
        assert_eq!(cheat.chars_read, 0);
        assert_eq!(cheat.param_chars_read, 0);
    }

    #[test]
    fn m_cheat_mismatch_resets_to_sequence_start() {
        let mut cheat = cheat("idkfa", "idkfa".len(), 0);

        assert_eq!(
            unsafe { cdoom_core::cdoom_rust_cht_check_cheat(&mut cheat, b'i' as c_char) },
            0
        );
        assert_eq!(cheat.chars_read, 1);
        assert_eq!(
            unsafe { cdoom_core::cdoom_rust_cht_check_cheat(&mut cheat, b'x' as c_char) },
            0
        );
        assert_eq!(cheat.chars_read, 0);
    }

    #[test]
    fn m_cheat_reads_parameters_before_success() {
        let mut cheat = cheat("idclev", "idclev".len(), 2);

        for key in "idclev1".bytes() {
            assert_eq!(
                unsafe { cdoom_core::cdoom_rust_cht_check_cheat(&mut cheat, key as c_char) },
                0
            );
        }

        assert_eq!(
            unsafe { cdoom_core::cdoom_rust_cht_check_cheat(&mut cheat, b'9' as c_char) },
            1
        );

        let mut params = [0 as c_char; 2];
        unsafe { cdoom_core::cdoom_rust_cht_get_param(&cheat, params.as_mut_ptr()) };
        assert_eq!(params, [b'1' as c_char, b'9' as c_char]);
    }

    #[test]
    fn m_cheat_rejects_short_dehacked_parameter_sequence() {
        let mut cheat = cheat("a", 2, 1);

        assert_eq!(
            unsafe { cdoom_core::cdoom_rust_cht_check_cheat(&mut cheat, b'a' as c_char) },
            0
        );
        assert_eq!(cheat.chars_read, 0);
        assert_eq!(cheat.param_chars_read, 0);
    }
}
