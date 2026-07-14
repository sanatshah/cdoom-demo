//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::mem;
use std::path::Path;
use std::str;

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
    use cdoom_core::d_event::{self, Event};
    use cdoom_core::d_iwad;
    use cdoom_core::d_mode;
    use cdoom_core::d_ticcmd::TicCmd;

    fn nul_terminated_str(bytes: &'static [u8]) -> &'static str {
        let end = bytes
            .iter()
            .position(|byte| *byte == b'\0')
            .expect("string must be NUL-terminated");
        str::from_utf8(&bytes[..end]).expect("string must be UTF-8")
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
    fn d_mode_valid_modes_match_c_table() {
        let valid_modes = [
            (d_mode::PACK_CHEX, d_mode::RETAIL),
            (d_mode::DOOM, d_mode::SHAREWARE),
            (d_mode::DOOM, d_mode::REGISTERED),
            (d_mode::DOOM, d_mode::RETAIL),
            (d_mode::DOOM2, d_mode::COMMERCIAL),
            (d_mode::PACK_TNT, d_mode::COMMERCIAL),
            (d_mode::PACK_PLUT, d_mode::COMMERCIAL),
            (d_mode::PACK_HACX, d_mode::COMMERCIAL),
            (d_mode::HERETIC, d_mode::SHAREWARE),
            (d_mode::HERETIC, d_mode::REGISTERED),
            (d_mode::HERETIC, d_mode::RETAIL),
            (d_mode::HEXEN, d_mode::COMMERCIAL),
            (d_mode::STRIFE, d_mode::COMMERCIAL),
        ];

        for (mission, mode) in valid_modes {
            assert!(d_mode::valid_game_mode(mission, mode));
        }

        assert!(!d_mode::valid_game_mode(d_mode::DOOM, d_mode::COMMERCIAL));
        assert!(!d_mode::valid_game_mode(d_mode::HEXEN, d_mode::RETAIL));
        assert!(!d_mode::valid_game_mode(10, d_mode::COMMERCIAL));
    }

    #[test]
    fn d_mode_episode_and_version_edges_match_c() {
        assert!(d_mode::valid_episode_map(
            d_mode::HERETIC,
            d_mode::RETAIL,
            6,
            3
        ));
        assert!(!d_mode::valid_episode_map(
            d_mode::HERETIC,
            d_mode::RETAIL,
            6,
            4
        ));
        assert!(d_mode::valid_episode_map(
            d_mode::HERETIC,
            d_mode::REGISTERED,
            4,
            1
        ));
        assert!(!d_mode::valid_episode_map(
            d_mode::HERETIC,
            d_mode::REGISTERED,
            4,
            2
        ));
        assert!(!d_mode::valid_episode_map(
            d_mode::DOOM,
            d_mode::SHAREWARE,
            2,
            1
        ));
        assert!(!d_mode::valid_episode_map(
            d_mode::DOOM,
            d_mode::SHAREWARE,
            1,
            10
        ));

        assert_eq!(d_mode::num_episodes(d_mode::DOOM, d_mode::RETAIL), 4);
        assert_eq!(d_mode::num_episodes(d_mode::HERETIC, d_mode::RETAIL), 6);
        assert_eq!(d_mode::num_episodes(d_mode::HERETIC, d_mode::REGISTERED), 4);

        assert!(d_mode::valid_game_version(d_mode::DOOM2, d_mode::EXE_FINAL));
        assert!(d_mode::valid_game_version(
            d_mode::PACK_CHEX,
            d_mode::EXE_CHEX
        ));
        assert!(d_mode::valid_game_version(
            d_mode::STRIFE,
            d_mode::EXE_STRIFE_1_31
        ));
        assert!(!d_mode::valid_game_version(
            d_mode::HERETIC,
            d_mode::EXE_DOOM_1_9
        ));
    }

    #[test]
    fn d_mode_string_helpers_match_c_defaults() {
        assert!(d_mode::is_episode_map(d_mode::DOOM));
        assert!(d_mode::is_episode_map(d_mode::HERETIC));
        assert!(d_mode::is_episode_map(d_mode::PACK_CHEX));
        assert!(!d_mode::is_episode_map(d_mode::DOOM2));
        assert!(!d_mode::is_episode_map(10));

        assert_eq!(
            nul_terminated_str(d_mode::game_mission_string(d_mode::DOOM)),
            "doom"
        );
        assert_eq!(
            nul_terminated_str(d_mode::game_mission_string(d_mode::PACK_PLUT)),
            "plutonia"
        );
        assert_eq!(nul_terminated_str(d_mode::game_mission_string(10)), "none");
        assert_eq!(
            nul_terminated_str(d_mode::game_mode_string(d_mode::SHAREWARE)),
            "shareware"
        );
        assert_eq!(nul_terminated_str(d_mode::game_mode_string(4)), "unknown");
    }

    #[test]
    fn d_event_queue_matches_c_ring_behavior() {
        d_event::reset_for_tests();
        assert!(d_event::pop_event().is_null());

        d_event::post_event(Event {
            event_type: 0,
            data1: 11,
            data2: 12,
            data3: 13,
            data4: 14,
            data5: 15,
            data6: 16,
        });
        d_event::post_event(Event {
            event_type: 1,
            data1: 21,
            data2: 22,
            data3: 23,
            data4: 24,
            data5: 25,
            data6: 26,
        });

        // SAFETY: pop_event returns a pointer into the static queue when non-null.
        let first = unsafe { *d_event::pop_event() };
        // SAFETY: pop_event returns a pointer into the static queue when non-null.
        let second = unsafe { *d_event::pop_event() };
        assert_eq!(first.data1, 11);
        assert_eq!(second.data1, 21);
        assert!(d_event::pop_event().is_null());

        d_event::reset_for_tests();
        for value in 0..64 {
            d_event::post_event(Event {
                event_type: 0,
                data1: value,
                data2: 0,
                data3: 0,
                data4: 0,
                data5: 0,
                data6: 0,
            });
        }
        assert!(d_event::pop_event().is_null());

        d_event::reset_for_tests();
        for value in 0..65 {
            d_event::post_event(Event {
                event_type: 0,
                data1: value,
                data2: 0,
                data3: 0,
                data4: 0,
                data5: 0,
                data6: 0,
            });
        }
        // SAFETY: the sixty-fifth post leaves exactly one readable wrapped slot.
        let wrapped = unsafe { *d_event::pop_event() };
        assert_eq!(wrapped.data1, 64);
        assert!(d_event::pop_event().is_null());
    }

    #[test]
    fn d_iwad_lookup_helpers_match_c_table_order() {
        assert!(d_iwad::is_iwad_name(b"doom2.wad"));
        assert!(d_iwad::is_iwad_name(b"DOOM2.WAD"));
        assert!(!d_iwad::is_iwad_name(b"doom2.wadx"));

        assert_eq!(
            nul_terminated_str(d_iwad::save_game_iwad_name(d_mode::DOOM, d_iwad::FREEDOOM)),
            "freedoom1.wad"
        );
        assert_eq!(
            nul_terminated_str(d_iwad::save_game_iwad_name(d_mode::DOOM2, d_iwad::FREEDOOM)),
            "freedoom2.wad"
        );
        assert_eq!(
            nul_terminated_str(d_iwad::save_game_iwad_name(d_mode::DOOM2, d_iwad::FREEDM)),
            "freedm.wad"
        );
        assert_eq!(
            nul_terminated_str(d_iwad::save_game_iwad_name(d_mode::PACK_TNT, 0)),
            "tnt.wad"
        );
        assert_eq!(
            nul_terminated_str(d_iwad::save_game_iwad_name(10, 0)),
            "unknown.wad"
        );

        assert_eq!(
            nul_terminated_str(d_iwad::suggest_iwad_name(d_mode::DOOM2, d_mode::COMMERCIAL)),
            "doom2.wad"
        );
        assert_eq!(
            nul_terminated_str(d_iwad::suggest_iwad_name(
                d_mode::HERETIC,
                d_mode::SHAREWARE
            )),
            "heretic1.wad"
        );
        assert_eq!(
            nul_terminated_str(d_iwad::suggest_iwad_name(10, d_mode::COMMERCIAL)),
            "unknown.wad"
        );

        assert_eq!(
            nul_terminated_str(d_iwad::suggest_game_name(
                d_mode::DOOM2,
                d_iwad::INDETERMINED
            )),
            "Doom II"
        );
        assert_eq!(
            nul_terminated_str(d_iwad::suggest_game_name(d_mode::DOOM2, d_mode::COMMERCIAL)),
            "Doom II"
        );
        assert_eq!(
            nul_terminated_str(d_iwad::suggest_game_name(d_mode::DOOM2, d_mode::REGISTERED)),
            "Unknown game?"
        );
    }

    #[test]
    fn ticcmd_layout_matches_c_abi() {
        assert_eq!(mem::size_of::<TicCmd>(), 16);
        assert_eq!(mem::align_of::<TicCmd>(), 4);
        assert_eq!(mem::offset_of!(TicCmd, forwardmove), 0);
        assert_eq!(mem::offset_of!(TicCmd, sidemove), 1);
        assert_eq!(mem::offset_of!(TicCmd, angleturn), 2);
        assert_eq!(mem::offset_of!(TicCmd, chatchar), 4);
        assert_eq!(mem::offset_of!(TicCmd, buttons), 5);
        assert_eq!(mem::offset_of!(TicCmd, consistancy), 6);
        assert_eq!(mem::offset_of!(TicCmd, buttons2), 7);
        assert_eq!(mem::offset_of!(TicCmd, inventory), 8);
        assert_eq!(mem::offset_of!(TicCmd, lookfly), 12);
        assert_eq!(mem::offset_of!(TicCmd, arti), 13);
    }
}
