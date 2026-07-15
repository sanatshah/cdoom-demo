//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;
use std::sync::Mutex;

use cdoom_core::m_argv;
use cdoom_core::m_cheat;

unsafe extern "C" {
    fn free(ptr: *mut c_void);
    fn malloc(size: usize) -> *mut c_void;
}

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

static ARGV_LOCK: Mutex<()> = Mutex::new(());

struct CArgvFixture {
    old_argc: c_int,
    old_argv: *mut *mut c_char,
}

impl CArgvFixture {
    fn new(args: &[&str]) -> Self {
        unsafe {
            let old_argc = m_argv::myargc;
            let old_argv = m_argv::myargv;
            let argv = malloc(args.len() * mem::size_of::<*mut c_char>()).cast::<*mut c_char>();

            assert!(!argv.is_null());

            for (index, arg) in args.iter().enumerate() {
                *argv.add(index) = duplicate_arg(arg);
            }

            m_argv::myargc = args.len() as c_int;
            m_argv::myargv = argv;

            Self { old_argc, old_argv }
        }
    }
}

impl Drop for CArgvFixture {
    fn drop(&mut self) {
        unsafe {
            for index in 0..m_argv::myargc {
                let arg = *m_argv::myargv.add(index as usize);

                if !arg.is_null() {
                    free(arg.cast::<c_void>());
                }
            }

            free(m_argv::myargv.cast::<c_void>());
            m_argv::myargc = self.old_argc;
            m_argv::myargv = self.old_argv;
        }
    }
}

unsafe fn duplicate_arg(arg: &str) -> *mut c_char {
    let bytes = arg.as_bytes();
    let dest = malloc(bytes.len() + 1).cast::<u8>();

    assert!(!dest.is_null());

    ptr::copy_nonoverlapping(bytes.as_ptr(), dest, bytes.len());
    *dest.add(bytes.len()) = 0;

    dest.cast::<c_char>()
}

unsafe fn current_args() -> Vec<String> {
    (0..m_argv::myargc)
        .map(|index| {
            CStr::from_ptr(*m_argv::myargv.add(index as usize))
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn key(ch: u8) -> c_char {
    ch as c_char
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
    fn argv_check_parm_matches_case_and_argument_count_semantics() {
        let _lock = ARGV_LOCK.lock().unwrap();
        let _fixture = CArgvFixture::new(&[
            "chocolate-doom",
            "-IWAD",
            "freedoom.wad",
            "-file",
            "addon.wad",
            "-solo",
        ]);
        let iwad = CString::new("-iwad").unwrap();
        let solo = CString::new("-solo").unwrap();
        let absent = CString::new("-missing").unwrap();

        unsafe {
            assert_eq!(m_argv::cdoom_rust_m_check_parm(iwad.as_ptr()), 1);
            assert_eq!(
                m_argv::cdoom_rust_m_check_parm_with_args(iwad.as_ptr(), 1),
                1
            );
            assert_eq!(
                m_argv::cdoom_rust_m_check_parm_with_args(solo.as_ptr(), 1),
                0
            );
            assert_eq!(m_argv::cdoom_rust_m_parm_exists(absent.as_ptr()), 0);
        }
    }

    #[test]
    fn argv_response_file_expands_quoted_and_unquoted_arguments() {
        let _lock = ARGV_LOCK.lock().unwrap();
        let response_path =
            std::env::temp_dir().join(format!("cdoom_phase3_{}_quoted.rsp", std::process::id()));
        std::fs::write(&response_path, b"-iwad \"two words.wad\"\n-file addon.wad").unwrap();
        let response_arg = format!("@{}", response_path.to_string_lossy());
        let _fixture = CArgvFixture::new(&["chocolate-doom", &response_arg, "-nosound"]);

        unsafe {
            m_argv::cdoom_rust_m_find_response_file();
            assert_eq!(
                current_args(),
                vec![
                    "chocolate-doom",
                    "-iwad",
                    "two words.wad",
                    "-file",
                    "addon.wad",
                    "-nosound"
                ]
            );
        }

        std::fs::remove_file(response_path).unwrap();
    }

    #[test]
    fn argv_response_option_replaces_marker_before_expansion() {
        let _lock = ARGV_LOCK.lock().unwrap();
        let response_path =
            std::env::temp_dir().join(format!("cdoom_phase3_{}_response.rsp", std::process::id()));
        std::fs::write(&response_path, b"-timedemo demo1").unwrap();
        let response_path = response_path.to_string_lossy().into_owned();
        let _fixture =
            CArgvFixture::new(&["chocolate-doom", "-response", &response_path, "-nosound"]);

        unsafe {
            m_argv::cdoom_rust_m_find_response_file();
            assert_eq!(
                current_args(),
                vec!["chocolate-doom", "-_", "-timedemo", "demo1", "-nosound"]
            );
        }

        std::fs::remove_file(response_path).unwrap();
    }

    #[test]
    fn cheat_sequence_matches_and_resets_like_c() {
        let mut cheat = m_cheat::CheatSeq::new(b"iddqd", 0);

        for ch in b"iddq" {
            assert_eq!(
                unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(*ch)) },
                0
            );
        }

        assert_eq!(
            unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(b'd')) },
            1
        );
        assert_eq!(cheat.chars_read, 0);
        assert_eq!(cheat.param_chars_read, 0);
    }

    #[test]
    fn cheat_wrong_key_resets_partial_match() {
        let mut cheat = m_cheat::CheatSeq::new(b"idkfa", 0);

        assert_eq!(
            unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(b'i')) },
            0
        );
        assert_eq!(
            unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(b'x')) },
            0
        );
        assert_eq!(cheat.chars_read, 0);
    }

    #[test]
    fn cheat_parameter_bytes_are_copied_after_match() {
        let mut cheat = m_cheat::CheatSeq::new(b"idclev", 2);
        let mut parameter = [0 as c_char; 2];

        for ch in b"idclev4" {
            assert_eq!(
                unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(*ch)) },
                0
            );
        }

        assert_eq!(
            unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(b'2')) },
            1
        );

        unsafe {
            m_cheat::cdoom_rust_cht_get_param(&cheat, parameter.as_mut_ptr());
        }

        assert_eq!(parameter, [key(b'4'), key(b'2')]);
    }

    #[test]
    fn cheat_short_parameterized_sequence_does_not_advance() {
        let mut cheat = m_cheat::CheatSeq::new(b"id", 2);
        cheat.sequence_len = 5;

        for ch in b"id12" {
            assert_eq!(
                unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(*ch)) },
                0
            );
        }

        assert_eq!(cheat.chars_read, 0);
        assert_eq!(cheat.param_chars_read, 0);
    }
}
