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
    use cdoom_core::m_argv;
    use cdoom_core::m_cheat::{CheatSeq, MAX_CHEAT_LEN, MAX_CHEAT_PARAMS};
    use std::ffi::CString;
    use std::os::raw::c_char;

    fn cheat(value: &str, parameters: i32) -> CheatSeq {
        let mut sequence = [0 as c_char; MAX_CHEAT_LEN];

        for (index, byte) in value.bytes().enumerate() {
            sequence[index] = byte as c_char;
        }

        CheatSeq {
            sequence,
            sequence_len: value.len(),
            parameter_chars: parameters,
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
    fn m_argv_check_parm_matches_c_bounds_and_case() {
        let args = [
            CString::new("doom").unwrap(),
            CString::new("-FILE").unwrap(),
            CString::new("demo.lmp").unwrap(),
            CString::new("-skill").unwrap(),
        ];
        let argv: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
        let file = CString::new("-file").unwrap();
        let skill = CString::new("-skill").unwrap();

        unsafe {
            assert_eq!(
                cdoom_core::cdoom_rust_m_check_parm_with_args(
                    argv.len() as i32,
                    argv.as_ptr(),
                    file.as_ptr(),
                    1,
                ),
                1,
            );
            assert_eq!(
                cdoom_core::cdoom_rust_m_check_parm_with_args(
                    argv.len() as i32,
                    argv.as_ptr(),
                    skill.as_ptr(),
                    1,
                ),
                0,
            );
            assert_eq!(
                cdoom_core::cdoom_rust_m_check_parm(
                    argv.len() as i32,
                    argv.as_ptr(),
                    skill.as_ptr()
                ),
                3,
            );
        }
    }

    #[test]
    fn m_argv_check_parm_stops_at_null_like_load_response_file_window() {
        let args = [
            CString::new("doom").unwrap(),
            CString::new("-first").unwrap(),
            CString::new("-after-null").unwrap(),
        ];
        let mut argv: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
        argv[1] = std::ptr::null();
        let after_null = CString::new("-after-null").unwrap();

        unsafe {
            assert_eq!(
                cdoom_core::cdoom_rust_m_check_parm(
                    argv.len() as i32,
                    argv.as_ptr(),
                    after_null.as_ptr(),
                ),
                0,
            );
        }
    }

    #[test]
    fn m_argv_response_parser_matches_c_quotes_and_whitespace() {
        let args =
            m_argv::parse_response_bytes(b"  -iwad \"long path.wad\"\n-skill\t1\r\n").unwrap();

        assert_eq!(
            args,
            vec![
                b"-iwad".to_vec(),
                b"long path.wad".to_vec(),
                b"-skill".to_vec(),
                b"1".to_vec(),
            ]
        );
    }

    #[test]
    fn m_argv_response_parser_rejects_newline_before_closing_quote() {
        assert_eq!(
            m_argv::parse_response_bytes(b"\"unterminated\n").unwrap_err(),
            m_argv::ResponseError::QuotesUnclosed,
        );
    }

    #[test]
    fn m_cheat_matches_sequence_and_resets_on_wrong_key() {
        let mut seq = cheat("aba", 0);

        assert!(!cdoom_core::m_cheat::check_cheat(&mut seq, b'a' as c_char));
        assert!(!cdoom_core::m_cheat::check_cheat(&mut seq, b'b' as c_char));
        assert!(!cdoom_core::m_cheat::check_cheat(&mut seq, b'b' as c_char));
        assert_eq!(seq.chars_read, 0);
        assert!(!cdoom_core::m_cheat::check_cheat(&mut seq, b'a' as c_char));
        assert!(!cdoom_core::m_cheat::check_cheat(&mut seq, b'b' as c_char));
        assert!(cdoom_core::m_cheat::check_cheat(&mut seq, b'a' as c_char));
        assert_eq!(seq.chars_read, 0);
    }

    #[test]
    fn m_cheat_collects_parameter_bytes() {
        let mut seq = cheat("idmus", 2);

        for key in b"idmus1" {
            assert!(!cdoom_core::m_cheat::check_cheat(&mut seq, *key as c_char));
        }

        assert!(cdoom_core::m_cheat::check_cheat(&mut seq, b'2' as c_char));

        let mut params = [0 as c_char; MAX_CHEAT_PARAMS];
        cdoom_core::m_cheat::get_param(&seq, &mut params[..2]);
        assert_eq!(params[0], b'1' as c_char);
        assert_eq!(params[1], b'2' as c_char);
    }

    #[test]
    fn m_cheat_preserves_short_parameterized_sequence_quirk() {
        let mut seq = cheat("id", 1);
        seq.sequence[2] = 0;
        seq.sequence_len = 3;

        for key in b"id1id1" {
            assert!(!cdoom_core::m_cheat::check_cheat(&mut seq, *key as c_char));
        }
    }
}
