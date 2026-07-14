//! Shared game mode helpers ported from `d_mode.c`.

pub const DOOM: i32 = 0;
pub const DOOM2: i32 = 1;
pub const PACK_TNT: i32 = 2;
pub const PACK_PLUT: i32 = 3;
pub const PACK_CHEX: i32 = 4;
pub const PACK_HACX: i32 = 5;
pub const HERETIC: i32 = 6;
pub const HEXEN: i32 = 7;
pub const STRIFE: i32 = 8;

pub const SHAREWARE: i32 = 0;
pub const REGISTERED: i32 = 1;
pub const COMMERCIAL: i32 = 2;
pub const RETAIL: i32 = 3;

pub const EXE_DOOM_1_2: i32 = 0;
pub const EXE_DOOM_1_5: i32 = 1;
pub const EXE_DOOM_1_666: i32 = 2;
pub const EXE_DOOM_1_7: i32 = 3;
pub const EXE_DOOM_1_8: i32 = 4;
pub const EXE_DOOM_1_9: i32 = 5;
pub const EXE_HACX: i32 = 6;
pub const EXE_ULTIMATE: i32 = 7;
pub const EXE_FINAL: i32 = 8;
pub const EXE_FINAL2: i32 = 9;
pub const EXE_CHEX: i32 = 10;
pub const EXE_HERETIC_1_3: i32 = 11;
pub const EXE_HEXEN_1_1: i32 = 12;
pub const EXE_HEXEN_1_1R2: i32 = 13;
pub const EXE_STRIFE_1_2: i32 = 14;
pub const EXE_STRIFE_1_31: i32 = 15;

#[derive(Copy, Clone)]
struct ValidMode {
    mission: i32,
    mode: i32,
    episode: i32,
    map: i32,
}

const VALID_MODES: &[ValidMode] = &[
    ValidMode {
        mission: PACK_CHEX,
        mode: RETAIL,
        episode: 1,
        map: 5,
    },
    ValidMode {
        mission: DOOM,
        mode: SHAREWARE,
        episode: 1,
        map: 9,
    },
    ValidMode {
        mission: DOOM,
        mode: REGISTERED,
        episode: 3,
        map: 9,
    },
    ValidMode {
        mission: DOOM,
        mode: RETAIL,
        episode: 4,
        map: 9,
    },
    ValidMode {
        mission: DOOM2,
        mode: COMMERCIAL,
        episode: 1,
        map: 32,
    },
    ValidMode {
        mission: PACK_TNT,
        mode: COMMERCIAL,
        episode: 1,
        map: 32,
    },
    ValidMode {
        mission: PACK_PLUT,
        mode: COMMERCIAL,
        episode: 1,
        map: 32,
    },
    ValidMode {
        mission: PACK_HACX,
        mode: COMMERCIAL,
        episode: 1,
        map: 32,
    },
    ValidMode {
        mission: HERETIC,
        mode: SHAREWARE,
        episode: 1,
        map: 9,
    },
    ValidMode {
        mission: HERETIC,
        mode: REGISTERED,
        episode: 3,
        map: 9,
    },
    ValidMode {
        mission: HERETIC,
        mode: RETAIL,
        episode: 5,
        map: 9,
    },
    ValidMode {
        mission: HEXEN,
        mode: COMMERCIAL,
        episode: 1,
        map: 60,
    },
    ValidMode {
        mission: STRIFE,
        mode: COMMERCIAL,
        episode: 1,
        map: 34,
    },
];

const VALID_VERSIONS: &[(i32, i32)] = &[
    (DOOM, EXE_DOOM_1_2),
    (DOOM, EXE_DOOM_1_5),
    (DOOM, EXE_DOOM_1_666),
    (DOOM, EXE_DOOM_1_7),
    (DOOM, EXE_DOOM_1_8),
    (DOOM, EXE_DOOM_1_9),
    (DOOM, EXE_HACX),
    (DOOM, EXE_ULTIMATE),
    (DOOM, EXE_FINAL),
    (DOOM, EXE_FINAL2),
    (DOOM, EXE_CHEX),
    (HERETIC, EXE_HERETIC_1_3),
    (HEXEN, EXE_HEXEN_1_1),
    (HEXEN, EXE_HEXEN_1_1R2),
    (STRIFE, EXE_STRIFE_1_2),
    (STRIFE, EXE_STRIFE_1_31),
];

pub fn valid_game_mode(mission: i32, mode: i32) -> bool {
    VALID_MODES
        .iter()
        .any(|valid| valid.mode == mode && valid.mission == mission)
}

pub fn valid_episode_map(mission: i32, mode: i32, episode: i32, map: i32) -> bool {
    if mission == HERETIC {
        if mode == RETAIL && episode == 6 {
            return (1..=3).contains(&map);
        } else if mode == REGISTERED && episode == 4 {
            return map == 1;
        }
    }

    VALID_MODES
        .iter()
        .find(|valid| valid.mission == mission && valid.mode == mode)
        .is_some_and(|valid| {
            episode >= 1 && episode <= valid.episode && map >= 1 && map <= valid.map
        })
}

pub fn num_episodes(mission: i32, mode: i32) -> i32 {
    let mut episode = 1;

    while valid_episode_map(mission, mode, episode, 1) {
        episode += 1;
    }

    episode - 1
}

pub fn valid_game_version(mut mission: i32, version: i32) -> bool {
    if matches!(
        mission,
        DOOM2 | PACK_PLUT | PACK_TNT | PACK_HACX | PACK_CHEX
    ) {
        mission = DOOM;
    }

    VALID_VERSIONS.iter().any(|(valid_mission, valid_version)| {
        *valid_mission == mission && *valid_version == version
    })
}

pub fn is_episode_map(mission: i32) -> bool {
    matches!(mission, DOOM | HERETIC | PACK_CHEX)
}

pub fn game_mission_string(mission: i32) -> &'static [u8] {
    match mission {
        DOOM => b"doom\0",
        DOOM2 => b"doom2\0",
        PACK_TNT => b"tnt\0",
        PACK_PLUT => b"plutonia\0",
        PACK_HACX => b"hacx\0",
        PACK_CHEX => b"chex\0",
        HERETIC => b"heretic\0",
        HEXEN => b"hexen\0",
        STRIFE => b"strife\0",
        _ => b"none\0",
    }
}

pub fn game_mode_string(mode: i32) -> &'static [u8] {
    match mode {
        SHAREWARE => b"shareware\0",
        REGISTERED => b"registered\0",
        COMMERCIAL => b"commercial\0",
        RETAIL => b"retail\0",
        _ => b"unknown\0",
    }
}
