//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use crate::{d_dedicated, d_event, d_iwad, d_loop, d_mode, d_ticcmd};

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::OnceLock;

pub type DedicatedStep = Option<extern "C" fn()>;

static VERSION: OnceLock<CString> = OnceLock::new();

fn version_cstr() -> &'static CString {
    VERSION.get_or_init(|| {
        CString::new(env!("CARGO_PKG_VERSION")).expect("version must not contain NUL")
    })
}

fn c_boolean(value: bool) -> c_int {
    if value {
        1
    } else {
        0
    }
}

fn c_string(bytes: &'static [u8]) -> *const c_char {
    bytes.as_ptr().cast()
}

/// Returns a pointer to a static, NUL-terminated version string.
///
/// # Safety
///
/// The returned pointer is valid for the process lifetime and must not be freed.
#[no_mangle]
pub extern "C" fn cdoom_rust_version() -> *const c_char {
    version_cstr().as_ptr()
}

/// One-time initialization hook for future Rust subsystems.
///
/// Returns `0` on success. Reserved for later migration phases.
#[no_mangle]
pub extern "C" fn cdoom_rust_init() -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn cdoom_rust_valid_game_mode(mission: c_int, mode: c_int) -> c_int {
    c_boolean(d_mode::valid_game_mode(mission, mode))
}

#[no_mangle]
pub extern "C" fn cdoom_rust_valid_episode_map(
    mission: c_int,
    mode: c_int,
    episode: c_int,
    map: c_int,
) -> c_int {
    c_boolean(d_mode::valid_episode_map(mission, mode, episode, map))
}

#[no_mangle]
pub extern "C" fn cdoom_rust_get_num_episodes(mission: c_int, mode: c_int) -> c_int {
    d_mode::num_episodes(mission, mode)
}

#[no_mangle]
pub extern "C" fn cdoom_rust_valid_game_version(mission: c_int, version: c_int) -> c_int {
    c_boolean(d_mode::valid_game_version(mission, version))
}

#[no_mangle]
pub extern "C" fn cdoom_rust_is_episode_map(mission: c_int) -> c_int {
    c_boolean(d_mode::is_episode_map(mission))
}

#[no_mangle]
pub extern "C" fn cdoom_rust_game_mission_string(mission: c_int) -> *const c_char {
    c_string(d_mode::game_mission_string(mission))
}

#[no_mangle]
pub extern "C" fn cdoom_rust_game_mode_string(mode: c_int) -> *const c_char {
    c_string(d_mode::game_mode_string(mode))
}

#[no_mangle]
pub extern "C" fn cdoom_rust_post_event(event: *const c_void) {
    // SAFETY: The C wrapper passes a valid pointer to `event_t`, whose layout is
    // mirrored by `d_event::Event`.
    let event = unsafe { *event.cast::<d_event::Event>() };
    d_event::post_event(event);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_pop_event() -> *mut c_void {
    d_event::pop_event().cast()
}

#[no_mangle]
pub extern "C" fn cdoom_rust_is_iwad_name(name: *const c_char) -> c_int {
    // SAFETY: C callers provide a NUL-terminated string, matching strcasecmp's
    // contract in the original implementation.
    let name = unsafe { CStr::from_ptr(name) };
    c_boolean(d_iwad::is_iwad_name(name.to_bytes()))
}

#[no_mangle]
pub extern "C" fn cdoom_rust_save_game_iwad_name(
    gamemission: c_int,
    gamevariant: c_int,
) -> *const c_char {
    c_string(d_iwad::save_game_iwad_name(gamemission, gamevariant))
}

#[no_mangle]
pub extern "C" fn cdoom_rust_suggest_iwad_name(mission: c_int, mode: c_int) -> *const c_char {
    c_string(d_iwad::suggest_iwad_name(mission, mode))
}

#[no_mangle]
pub extern "C" fn cdoom_rust_suggest_game_name(mission: c_int, mode: c_int) -> *const c_char {
    c_string(d_iwad::suggest_game_name(mission, mode))
}

#[no_mangle]
pub extern "C" fn cdoom_rust_d_loop_nonvanilla_record_allowed(
    conditional: c_int,
    strict_demos: c_int,
) -> c_int {
    c_boolean(d_loop::nonvanilla_record_allowed(
        d_loop::c_bool(conditional),
        d_loop::c_bool(strict_demos),
    ))
}

#[no_mangle]
pub extern "C" fn cdoom_rust_d_loop_nonvanilla_playback_decision(
    conditional: c_int,
    strict_demos: c_int,
    is_demo_file: c_int,
) -> c_int {
    d_loop::nonvanilla_playback_decision(
        d_loop::c_bool(conditional),
        d_loop::c_bool(strict_demos),
        d_loop::c_bool(is_demo_file),
    ) as c_int
}

#[no_mangle]
pub extern "C" fn cdoom_rust_d_loop_get_low_tic(
    maketic: c_int,
    recvtic: c_int,
    net_client_connected: c_int,
    drone: c_int,
) -> c_int {
    d_loop::get_low_tic(
        maketic,
        recvtic,
        d_loop::c_bool(net_client_connected),
        d_loop::c_bool(drone),
    )
}

#[no_mangle]
pub extern "C" fn cdoom_rust_d_loop_ticdup_squash(cmds: *mut c_void, player_count: usize) {
    // SAFETY: C passes a writable `ticcmd_t` array with `player_count` entries.
    unsafe { d_loop::ticdup_squash(cmds.cast::<d_ticcmd::TicCmd>(), player_count) };
}

#[no_mangle]
pub extern "C" fn cdoom_rust_d_loop_single_player_clear(
    ingame: *mut c_int,
    player_count: usize,
    localplayer: c_int,
) {
    // SAFETY: C passes a writable `boolean` array with `player_count` entries.
    unsafe { d_loop::single_player_clear(ingame, player_count, localplayer) };
}

#[no_mangle]
pub extern "C" fn cdoom_rust_d_loop_players_in_game(
    net_client_connected: c_int,
    drone: c_int,
    local_playeringame: *const c_int,
    player_count: usize,
) -> c_int {
    // SAFETY: C passes a readable `boolean` array with `player_count` entries.
    c_boolean(unsafe {
        d_loop::players_in_game(
            d_loop::c_bool(net_client_connected),
            d_loop::c_bool(drone),
            local_playeringame,
            player_count,
        )
    })
}

#[no_mangle]
pub extern "C" fn cdoom_rust_dedicated_net_client_run() {
    d_dedicated::net_client_run();
}

#[no_mangle]
pub extern "C" fn cdoom_rust_dedicated_main(
    print_banner: DedicatedStep,
    init_zone: DedicatedStep,
    run_server: DedicatedStep,
) {
    d_dedicated::dedicated_main(print_banner, init_zone, run_server);
}

#[no_mangle]
pub extern "C" fn cdoom_rust_dedicated_rejected_option(
    argc: c_int,
    argv: *const *const c_char,
) -> *const c_char {
    // SAFETY: C passes `myargc` and `myargv`.
    unsafe { d_dedicated::rejected_option(argc, argv) }
}
