//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::os::raw::c_char;
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
            Some(b"chocolate-doom".as_slice()),
            Some(b"-IWAD".as_slice()),
            Some(b"freedoom1.wad".as_slice()),
            Some(b"-file".as_slice()),
            Some(b"extra.wad".as_slice()),
        ];

        assert_eq!(m_argv::check_parm_with_args(&args, b"-iwad", 1), 1);
        assert_eq!(m_argv::check_parm_with_args(&args, b"-file", 1), 3);
        assert_eq!(m_argv::check_parm_with_args(&args, b"extra.wad", 1), 0);
        assert_eq!(m_argv::check_parm_with_args(&args, b"-missing", 0), 0);
    }

    #[test]
    fn m_argv_check_parm_stops_at_null_during_response_load() {
        let args = [
            Some(b"chocolate-doom".as_slice()),
            Some(b"-iwad".as_slice()),
            None,
            Some(b"-file".as_slice()),
        ];

        assert_eq!(m_argv::check_parm_with_args(&args, b"-iwad", 0), 1);
        assert_eq!(m_argv::check_parm_with_args(&args, b"-file", 0), 0);
    }

    #[test]
    fn m_argv_response_parser_preserves_vanilla_quoting() {
        let parsed = m_argv::parse_response_file(
            b"  -iwad \"Freedoom One.wad\"\n-file\tmaps.wad \"quoted value\"",
        )
        .unwrap();

        assert_eq!(
            parsed,
            vec![
                b"-iwad".to_vec(),
                b"Freedoom One.wad".to_vec(),
                b"-file".to_vec(),
                b"maps.wad".to_vec(),
                b"quoted value".to_vec(),
            ]
        );
    }

    #[test]
    fn m_argv_response_parser_rejects_newline_before_closing_quote() {
        assert_eq!(
            m_argv::parse_response_file(b"\"unterminated\nvalue\""),
            Err(m_argv::ResponseParseError::UnclosedQuote)
        );
    }

    #[test]
    fn m_cheat_matches_sequence_and_resets_after_success() {
        let mut cheat = cheat("iddqd", 0);

        for key in b"iddq" {
            assert!(!cdoom_core::m_cheat::check_cheat(
                &mut cheat,
                *key as c_char
            ));
        }

        assert!(cdoom_core::m_cheat::check_cheat(&mut cheat, b'd' as c_char));
        assert_eq!(cheat.chars_read, 0);
        assert_eq!(cheat.param_chars_read, 0);
    }

    #[test]
    fn m_cheat_wrong_key_resets_to_start_without_overlap_matching() {
        let mut cheat = cheat("iddqd", 0);

        assert!(!cdoom_core::m_cheat::check_cheat(
            &mut cheat,
            b'i' as c_char
        ));
        assert_eq!(cheat.chars_read, 1);
        assert!(!cdoom_core::m_cheat::check_cheat(
            &mut cheat,
            b'x' as c_char
        ));
        assert_eq!(cheat.chars_read, 0);
        assert!(!cdoom_core::m_cheat::check_cheat(
            &mut cheat,
            b'd' as c_char
        ));
        assert_eq!(cheat.chars_read, 0);
    }

    #[test]
    fn m_cheat_collects_parameters_after_sequence() {
        let mut cheat = cheat("idmus", 2);

        for key in b"idmus1" {
            assert!(!cdoom_core::m_cheat::check_cheat(
                &mut cheat,
                *key as c_char
            ));
        }

        assert!(cdoom_core::m_cheat::check_cheat(&mut cheat, b'9' as c_char));

        let mut buffer = [0 as c_char; MAX_CHEAT_PARAMS];
        cdoom_core::m_cheat::get_param(&cheat, &mut buffer);
        assert_eq!(buffer[0], b'1' as c_char);
        assert_eq!(buffer[1], b'9' as c_char);
    }

    #[test]
    fn m_cheat_preserves_vanilla_short_parameter_sequence_failure() {
        let mut cheat = cheat("idmus", 2);
        cheat.sequence[3] = 0;

        for key in b"idm19" {
            assert!(!cdoom_core::m_cheat::check_cheat(
                &mut cheat,
                *key as c_char
            ));
        }
    }

    fn cheat(sequence: &str, parameter_chars: i32) -> CheatSeq {
        let mut bytes = [0 as c_char; MAX_CHEAT_LEN];

        for (index, byte) in sequence.bytes().enumerate() {
            bytes[index] = byte as c_char;
        }

        CheatSeq {
            sequence: bytes,
            sequence_len: sequence.len(),
            parameter_chars,
            chars_read: 0,
            param_chars_read: 0,
            parameter_buf: [0 as c_char; MAX_CHEAT_PARAMS],
        }
    }
}
