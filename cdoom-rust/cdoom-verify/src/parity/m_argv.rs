//! Argv / response-file parity against chocolate-doom/src/m_argv.c (Phase 3).

use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::sync::Mutex;

use cdoom_core::m_argv;

unsafe extern "C" {
    fn free(ptr: *mut c_void);
    fn malloc(size: usize) -> *mut c_void;
}

/// Serialize mutation of the process-global `myargc` / `myargv` / `exedir`.
static ARGV_LOCK: Mutex<()> = Mutex::new(());

struct CArgvFixture {
    old_argc: c_int,
    old_argv: *mut *mut c_char,
}

impl CArgvFixture {
    fn new(args: &[&str]) -> Self {
        let slots: Vec<Option<&str>> = args.iter().copied().map(Some).collect();
        Self::from_slots(&slots)
    }

    /// Builds `myargv` from slots; `None` becomes a NULL hole (m_argv.c stops there).
    fn from_slots(slots: &[Option<&str>]) -> Self {
        unsafe {
            let old_argc = m_argv::myargc;
            let old_argv = m_argv::myargv;
            let argv =
                malloc(slots.len() * mem::size_of::<*mut c_char>()).cast::<*mut c_char>();

            assert!(!argv.is_null());

            for (index, slot) in slots.iter().enumerate() {
                *argv.add(index) = match slot {
                    Some(arg) => duplicate_arg(arg),
                    None => ptr::null_mut(),
                };
            }

            m_argv::myargc = slots.len() as c_int;
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
            let ptr = *m_argv::myargv.add(index as usize);
            if ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        })
        .collect()
}

#[test]
fn argv_check_parm_matches_case_and_argument_count_semantics() {
    // m_argv.c: M_CheckParmWithArgs — strcasecmp match; requires room for num_args.
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
        // -solo is last; WithArgs(1) needs one trailing arg — C returns 0.
        assert_eq!(
            m_argv::cdoom_rust_m_check_parm_with_args(solo.as_ptr(), 1),
            0
        );
        assert_eq!(m_argv::cdoom_rust_m_parm_exists(absent.as_ptr()), 0);
    }
}

#[test]
fn argv_check_parm_stops_at_null_hole_in_myargv() {
    // m_argv.c:50-53 — loop condition `myargv[i]` stops at a NULL slot.
    let _lock = ARGV_LOCK.lock().unwrap();
    let _fixture = CArgvFixture::from_slots(&[
        Some("chocolate-doom"),
        Some("-file"),
        None,
        Some("-iwad"),
        Some("freedoom.wad"),
    ]);
    let file = CString::new("-file").unwrap();
    let iwad = CString::new("-iwad").unwrap();

    unsafe {
        assert_eq!(m_argv::cdoom_rust_m_check_parm(file.as_ptr()), 1);
        assert_eq!(m_argv::cdoom_rust_m_check_parm(iwad.as_ptr()), 0);
        assert_eq!(m_argv::cdoom_rust_m_parm_exists(iwad.as_ptr()), 0);
    }
}

#[test]
fn argv_response_file_expands_quoted_and_unquoted_arguments() {
    // m_argv.c LoadResponseFile — quoted tokens keep interior spaces.
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

    let _ = std::fs::remove_file(response_path);
}

#[test]
fn argv_response_option_replaces_marker_before_expansion() {
    // m_argv.c M_FindResponseFile — `-response` becomes `-_` then expands the path.
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

    let _ = std::fs::remove_file(response_path);
}

#[test]
fn argv_executable_basename_and_exedir_trailing_separator() {
    // m_argv.c M_GetExecutableName / M_SetExeDir — basename + dirname + DIR_SEPARATOR.
    let _lock = ARGV_LOCK.lock().unwrap();
    let _fixture = CArgvFixture::new(&["/usr/local/bin/chocolate-doom"]);

    unsafe {
        let basename = CStr::from_ptr(m_argv::cdoom_rust_m_get_executable_name())
            .to_str()
            .unwrap();
        assert_eq!(basename, "chocolate-doom");

        let previous_exedir = m_argv::exedir;
        m_argv::cdoom_rust_m_set_exe_dir();
        let dir = CStr::from_ptr(m_argv::exedir).to_str().unwrap();
        assert_eq!(dir, "/usr/local/bin/");

        free(m_argv::exedir.cast::<c_void>());
        m_argv::exedir = previous_exedir;
    }
}

#[test]
fn argv_exedir_defaults_to_dot_slash_without_separator() {
    // m_argv.c M_SetExeDir — no DIR_SEPARATOR in argv[0] → "./".
    let _lock = ARGV_LOCK.lock().unwrap();
    let _fixture = CArgvFixture::new(&["chocolate-doom"]);

    unsafe {
        let previous_exedir = m_argv::exedir;
        m_argv::cdoom_rust_m_set_exe_dir();
        let dir = CStr::from_ptr(m_argv::exedir).to_str().unwrap();
        assert_eq!(dir, "./");

        free(m_argv::exedir.cast::<c_void>());
        m_argv::exedir = previous_exedir;
    }
}
