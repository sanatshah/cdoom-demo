//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::ptr;
use std::sync::Mutex;

use cdoom_core::m_cheat::{CheatSeq, MAX_CHEAT_LEN, MAX_CHEAT_PARAMS};

static ARGV_LOCK: Mutex<()> = Mutex::new(());

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

    fn cheat_sequence(sequence: &[u8], parameter_chars: i32) -> CheatSeq {
        let mut cheat = CheatSeq {
            sequence: [0; MAX_CHEAT_LEN],
            sequence_len: sequence.len(),
            parameter_chars,
            chars_read: 0,
            param_chars_read: 0,
            parameter_buf: [0; MAX_CHEAT_PARAMS],
        };

        for (index, &ch) in sequence.iter().enumerate() {
            cheat.sequence[index] = ch as c_char;
        }

        cheat
    }

    fn feed_cheat(cheat: &mut CheatSeq, input: &[u8]) -> Vec<i32> {
        input
            .iter()
            .map(|&key| cdoom_core::m_cheat::check_cheat(cheat, key as c_char))
            .collect()
    }

    #[test]
    fn m_cheat_matches_complete_sequence_and_resets() {
        let mut cheat = cheat_sequence(b"iddqd", 0);

        assert_eq!(feed_cheat(&mut cheat, b"iddq"), [0, 0, 0, 0]);
        assert_eq!(feed_cheat(&mut cheat, b"d"), [1]);
        assert_eq!(cheat.chars_read, 0);
        assert_eq!(feed_cheat(&mut cheat, b"iddqd"), [0, 0, 0, 0, 1]);
    }

    #[test]
    fn m_cheat_resets_to_start_without_rechecking_wrong_key() {
        let mut cheat = cheat_sequence(b"iddqd", 0);

        assert_eq!(feed_cheat(&mut cheat, b"iiddqd"), [0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn m_cheat_collects_parameter_bytes() {
        let mut cheat = cheat_sequence(b"idclev", 2);

        assert_eq!(
            feed_cheat(&mut cheat, b"idclev12"),
            [0, 0, 0, 0, 0, 0, 0, 1]
        );

        let mut params = [0 as c_char; 2];
        cdoom_core::m_cheat::get_param(&cheat, params.as_mut_ptr());
        assert_eq!(params, [b'1' as c_char, b'2' as c_char]);
    }

    #[test]
    fn m_cheat_preserves_short_parameter_sequence_edge_case() {
        let mut cheat = cheat_sequence(b"idclev", 1);
        cheat.sequence[3] = 0;

        assert_eq!(
            feed_cheat(&mut cheat, b"idc1idc1"),
            [0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    unsafe fn alloc_c_string(bytes: &[u8]) -> *mut c_char {
        let ptr = unsafe { malloc(bytes.len() + 1) as *mut c_char };
        assert!(!ptr.is_null());

        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, ptr, bytes.len());
            *ptr.add(bytes.len()) = 0;
        }

        ptr
    }

    unsafe fn install_argv(args: &[Option<&[u8]>]) {
        let argv =
            unsafe { malloc(std::mem::size_of::<*mut c_char>() * args.len()) as *mut *mut c_char };
        assert!(!argv.is_null());

        for (index, arg) in args.iter().enumerate() {
            let ptr = match arg {
                Some(bytes) => unsafe { alloc_c_string(bytes) },
                None => ptr::null_mut(),
            };

            unsafe {
                *argv.add(index) = ptr;
            }
        }

        unsafe {
            cdoom_core::myargc = args.len() as i32;
            cdoom_core::myargv = argv;
        }
    }

    unsafe fn collect_argv() -> Vec<Option<String>> {
        let mut result = Vec::new();

        unsafe {
            for index in 0..cdoom_core::myargc {
                let ptr = *cdoom_core::myargv.add(index as usize);
                if ptr.is_null() {
                    result.push(None);
                } else {
                    result.push(Some(CStr::from_ptr(ptr).to_string_lossy().into_owned()));
                }
            }
        }

        result
    }

    unsafe fn clear_argv() {
        unsafe {
            if !cdoom_core::myargv.is_null() {
                for index in 0..cdoom_core::myargc {
                    let ptr = *cdoom_core::myargv.add(index as usize);
                    if !ptr.is_null() {
                        free(ptr as *mut c_void);
                    }
                }

                free(cdoom_core::myargv as *mut c_void);
            }

            cdoom_core::myargc = 0;
            cdoom_core::myargv = ptr::null_mut();

            if !cdoom_core::exedir.is_null() {
                free(cdoom_core::exedir as *mut c_void);
                cdoom_core::exedir = ptr::null_mut();
            }
        }
    }

    #[test]
    fn m_argv_check_parm_matches_case_and_argument_count_semantics() {
        let _guard = ARGV_LOCK.lock().unwrap();

        unsafe {
            install_argv(&[
                Some(b"/tmp/chocolate-doom"),
                Some(b"-IWAD"),
                Some(b"freedoom1.wad"),
                Some(b"-skill"),
            ]);

            assert_eq!(
                cdoom_core::cdoom_rust_m_check_parm_with_args(c"-iwad".as_ptr(), 1,),
                1
            );
            assert_eq!(
                cdoom_core::cdoom_rust_m_check_parm_with_args(c"-skill".as_ptr(), 1,),
                0
            );
            assert_eq!(cdoom_core::cdoom_rust_m_parm_exists(c"-IWAD".as_ptr()), 1);
            assert_eq!(
                cdoom_core::cdoom_rust_m_parm_exists(c"-missing".as_ptr()),
                0
            );

            clear_argv();
        }
    }

    #[test]
    fn m_argv_check_parm_stops_at_null_hole() {
        let _guard = ARGV_LOCK.lock().unwrap();

        unsafe {
            install_argv(&[Some(b"doom"), Some(b"-first"), None, Some(b"-later")]);

            assert_eq!(cdoom_core::cdoom_rust_m_check_parm(c"-first".as_ptr()), 1);
            assert_eq!(cdoom_core::cdoom_rust_m_check_parm(c"-later".as_ptr()), 0);

            clear_argv();
        }
    }

    #[test]
    fn m_argv_find_response_file_expands_quoted_and_following_args() {
        let _guard = ARGV_LOCK.lock().unwrap();
        let response_path =
            std::env::temp_dir().join(format!("cdoom-rust-response-{}.rsp", std::process::id()));

        std::fs::write(&response_path, b"-iwad \"two words.wad\"\n-skill 1").unwrap();
        let response_arg = format!("@{}", response_path.display());

        unsafe {
            install_argv(&[
                Some(b"doom"),
                Some(response_arg.as_bytes()),
                Some(b"-after"),
            ]);

            cdoom_core::cdoom_rust_m_find_response_file();

            assert_eq!(
                collect_argv(),
                [
                    Some("doom".to_string()),
                    Some("-iwad".to_string()),
                    Some("two words.wad".to_string()),
                    Some("-skill".to_string()),
                    Some("1".to_string()),
                    Some("-after".to_string()),
                ]
            );

            clear_argv();
        }

        std::fs::remove_file(response_path).unwrap();
    }

    #[test]
    fn m_argv_executable_name_and_exedir_match_c_path_helpers() {
        let _guard = ARGV_LOCK.lock().unwrap();

        unsafe {
            install_argv(&[Some(b"/usr/local/bin/chocolate-doom")]);

            let basename = CStr::from_ptr(cdoom_core::cdoom_rust_m_get_executable_name())
                .to_string_lossy()
                .into_owned();
            assert_eq!(basename, "chocolate-doom");

            cdoom_core::cdoom_rust_m_set_exe_dir();
            let exedir = CStr::from_ptr(cdoom_core::exedir)
                .to_string_lossy()
                .into_owned();
            assert_eq!(exedir, "/usr/local/bin/");

            clear_argv();
        }
    }
}
