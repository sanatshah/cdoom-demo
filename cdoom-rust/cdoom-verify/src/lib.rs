//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::path::Path;

use cdoom_core::m_argv;
use cdoom_core::m_cheat::{self, CheatSeq};

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
    fn m_argv_check_parm_matches_c_index_and_tail_room_rules() {
        let argv = [
            Some("doom".as_bytes()),
            Some("-IWAD".as_bytes()),
            Some("freedoom1.wad".as_bytes()),
            Some("-height".as_bytes()),
        ];

        assert_eq!(m_argv::check_parm(&argv, b"-iwad", 1), 1);
        assert_eq!(m_argv::check_parm(&argv, b"-height", 1), 0);
        assert_eq!(m_argv::check_parm(&argv, b"doom", 0), 0);
    }

    #[test]
    fn m_argv_check_parm_stops_at_null_slots_like_c_loop() {
        let argv = [
            Some("doom".as_bytes()),
            Some("-first".as_bytes()),
            None,
            Some("-later".as_bytes()),
        ];

        assert_eq!(m_argv::check_parm(&argv, b"-first", 0), 1);
        assert_eq!(m_argv::check_parm(&argv, b"-later", 0), 0);
    }

    #[test]
    fn m_argv_expands_at_files_and_response_placeholders() {
        let args = vec![
            b"doom".to_vec(),
            b"@outer.rsp".to_vec(),
            b"-file".to_vec(),
            b"tail.wad".to_vec(),
        ];

        let expanded = m_argv::expand_response_files(args, |filename| match filename {
            b"outer.rsp" => Ok(b"-skill 3 \"quoted arg\" -response inner.rsp".to_vec()),
            b"inner.rsp" => Ok(b"-warp 1 1".to_vec()),
            _ => Err(m_argv::ResponseError::MissingFile(filename.to_vec())),
        })
        .expect("response files should expand");

        assert_eq!(
            expanded,
            vec![
                b"doom".to_vec(),
                b"-skill".to_vec(),
                b"3".to_vec(),
                b"quoted arg".to_vec(),
                b"-_".to_vec(),
                b"-warp".to_vec(),
                b"1".to_vec(),
                b"1".to_vec(),
                b"-file".to_vec(),
                b"tail.wad".to_vec(),
            ]
        );
    }

    #[test]
    fn m_argv_rejects_unclosed_quoted_response_args_at_newline() {
        let err = m_argv::parse_response_bytes(b"\"unterminated\n", b"bad.rsp")
            .expect_err("newline before quote should fail");

        assert_eq!(
            err,
            m_argv::ResponseError::UnclosedQuote(b"bad.rsp".to_vec())
        );
    }

    #[test]
    fn m_cheat_matches_sequence_and_resets_on_success() {
        let mut cheat = CheatSeq::new(b"iddqd", 5, 0);

        assert!(!m_cheat::check_cheat(&mut cheat, b'i' as i8));
        assert!(!m_cheat::check_cheat(&mut cheat, b'x' as i8));
        assert_eq!(cheat.chars_read, 0);

        for key in b"iddq" {
            assert!(!m_cheat::check_cheat(&mut cheat, *key as i8));
        }

        assert!(m_cheat::check_cheat(&mut cheat, b'd' as i8));
        assert_eq!(cheat.chars_read, 0);
        assert_eq!(cheat.param_chars_read, 0);
    }

    #[test]
    fn m_cheat_captures_raw_parameter_bytes_without_terminator() {
        let mut cheat = CheatSeq::new(b"idclev", 6, 2);

        for key in b"idclev3" {
            assert!(!m_cheat::check_cheat(&mut cheat, *key as i8));
        }

        assert!(m_cheat::check_cheat(&mut cheat, b'1' as i8));

        let mut param = [0_i8; 3];
        unsafe {
            m_cheat::get_param(&cheat, param.as_mut_ptr());
        }

        assert_eq!(param, [b'3' as i8, b'1' as i8, 0]);
    }

    #[test]
    fn m_cheat_preserves_vanilla_shortened_parameterized_guard() {
        let mut cheat = CheatSeq::new(b"id", 6, 2);

        for key in b"id31" {
            assert!(!m_cheat::check_cheat(&mut cheat, *key as i8));
        }
    }
}
