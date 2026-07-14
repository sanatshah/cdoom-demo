//! Small, deterministic helpers from `d_loop.c`.
//!
//! The gameplay/network tick runner stays in C for now. These helpers cover
//! isolated policy decisions and per-tic command mutations that are easy to
//! verify without changing callback ownership or networking side effects.

use crate::d_ticcmd::TicCmd;

use std::os::raw::c_int;
use std::slice;

const BT_SPECIAL: u8 = 128;

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NonVanillaPlaybackDecision {
    DenySilently = 0,
    AllowDemoFile = 1,
    DenyWadDemo = 2,
}

pub fn nonvanilla_record_allowed(conditional: bool, strict_demos: bool) -> bool {
    conditional && !strict_demos
}

pub fn nonvanilla_playback_decision(
    conditional: bool,
    strict_demos: bool,
    is_demo_file: bool,
) -> NonVanillaPlaybackDecision {
    if !conditional || strict_demos {
        NonVanillaPlaybackDecision::DenySilently
    } else if is_demo_file {
        NonVanillaPlaybackDecision::AllowDemoFile
    } else {
        NonVanillaPlaybackDecision::DenyWadDemo
    }
}

pub fn get_low_tic(
    maketic: c_int,
    recvtic: c_int,
    net_client_connected: bool,
    drone: bool,
) -> c_int {
    let mut lowtic = maketic;

    if net_client_connected && (drone || recvtic < lowtic) {
        lowtic = recvtic;
    }

    lowtic
}

pub unsafe fn ticdup_squash(cmds: *mut TicCmd, player_count: usize) {
    if cmds.is_null() {
        return;
    }

    // SAFETY: The C caller passes a `ticcmd_t[NET_MAXPLAYERS]` buffer and the
    // matching player count. The Rust layout is verified against `d_ticcmd.h`.
    let cmds = unsafe { slice::from_raw_parts_mut(cmds, player_count) };

    for cmd in cmds {
        cmd.chatchar = 0;
        if cmd.buttons & BT_SPECIAL != 0 {
            cmd.buttons = 0;
        }
    }
}

pub unsafe fn single_player_clear(ingame: *mut c_int, player_count: usize, localplayer: c_int) {
    if ingame.is_null() {
        return;
    }

    // SAFETY: The C caller passes a `boolean[NET_MAXPLAYERS]` buffer and the
    // matching player count. Chocolate Doom's `boolean` is an `int`.
    let ingame = unsafe { slice::from_raw_parts_mut(ingame, player_count) };

    for (player, active) in ingame.iter_mut().enumerate() {
        if player as c_int != localplayer {
            *active = 0;
        }
    }
}

pub unsafe fn players_in_game(
    net_client_connected: bool,
    drone: bool,
    local_playeringame: *const c_int,
    player_count: usize,
) -> bool {
    let mut result = false;

    if net_client_connected && !local_playeringame.is_null() {
        // SAFETY: The C caller passes a `boolean[NET_MAXPLAYERS]` buffer and
        // the matching player count. Chocolate Doom's `boolean` is an `int`.
        let ingame = unsafe { slice::from_raw_parts(local_playeringame, player_count) };
        result = ingame.iter().any(|active| *active != 0);
    }

    if !drone {
        result = true;
    }

    result
}

pub fn c_bool(value: c_int) -> bool {
    value != 0
}
