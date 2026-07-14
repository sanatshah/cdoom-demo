//! Lookup-only IWAD helpers ported from `d_iwad.c`.

use crate::d_mode::{
    COMMERCIAL, DOOM, DOOM2, HERETIC, HEXEN, PACK_CHEX, PACK_HACX, PACK_PLUT, PACK_TNT, REGISTERED,
    RETAIL, SHAREWARE, STRIFE,
};

pub const FREEDOOM: i32 = 1;
pub const FREEDM: i32 = 2;
pub const INDETERMINED: i32 = 4;

#[derive(Copy, Clone)]
struct Iwad {
    name: &'static [u8],
    mission: i32,
    mode: i32,
    description: &'static [u8],
}

const IWADS: &[Iwad] = &[
    Iwad {
        name: b"doom2.wad\0",
        mission: DOOM2,
        mode: COMMERCIAL,
        description: b"Doom II\0",
    },
    Iwad {
        name: b"plutonia.wad\0",
        mission: PACK_PLUT,
        mode: COMMERCIAL,
        description: b"Final Doom: Plutonia Experiment\0",
    },
    Iwad {
        name: b"tnt.wad\0",
        mission: PACK_TNT,
        mode: COMMERCIAL,
        description: b"Final Doom: TNT: Evilution\0",
    },
    Iwad {
        name: b"doom.wad\0",
        mission: DOOM,
        mode: RETAIL,
        description: b"Doom\0",
    },
    Iwad {
        name: b"doom1.wad\0",
        mission: DOOM,
        mode: SHAREWARE,
        description: b"Doom Shareware\0",
    },
    Iwad {
        name: b"doom2f.wad\0",
        mission: DOOM2,
        mode: COMMERCIAL,
        description: b"Doom II: L'Enfer sur Terre\0",
    },
    Iwad {
        name: b"chex.wad\0",
        mission: PACK_CHEX,
        mode: RETAIL,
        description: b"Chex Quest\0",
    },
    Iwad {
        name: b"hacx.wad\0",
        mission: PACK_HACX,
        mode: COMMERCIAL,
        description: b"Hacx\0",
    },
    Iwad {
        name: b"freedoom2.wad\0",
        mission: DOOM2,
        mode: COMMERCIAL,
        description: b"Freedoom: Phase 2\0",
    },
    Iwad {
        name: b"freedoom1.wad\0",
        mission: DOOM,
        mode: RETAIL,
        description: b"Freedoom: Phase 1\0",
    },
    Iwad {
        name: b"freedm.wad\0",
        mission: DOOM2,
        mode: COMMERCIAL,
        description: b"FreeDM\0",
    },
    Iwad {
        name: b"heretic.wad\0",
        mission: HERETIC,
        mode: RETAIL,
        description: b"Heretic\0",
    },
    Iwad {
        name: b"heretic1.wad\0",
        mission: HERETIC,
        mode: SHAREWARE,
        description: b"Heretic Shareware\0",
    },
    Iwad {
        name: b"hexen.wad\0",
        mission: HEXEN,
        mode: COMMERCIAL,
        description: b"Hexen\0",
    },
    Iwad {
        name: b"strife1.wad\0",
        mission: STRIFE,
        mode: COMMERCIAL,
        description: b"Strife\0",
    },
];

fn bytes_before_nul(bytes: &[u8]) -> &[u8] {
    let end = bytes
        .iter()
        .position(|byte| *byte == b'\0')
        .unwrap_or(bytes.len());
    &bytes[..end]
}

fn eq_ignore_ascii_case(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.eq_ignore_ascii_case(right))
}

pub fn is_iwad_name(name: &[u8]) -> bool {
    IWADS
        .iter()
        .any(|iwad| eq_ignore_ascii_case(name, bytes_before_nul(iwad.name)))
}

pub fn save_game_iwad_name(gamemission: i32, gamevariant: i32) -> &'static [u8] {
    if gamevariant == FREEDOOM {
        if gamemission == DOOM {
            return b"freedoom1.wad\0";
        } else if gamemission == DOOM2 {
            return b"freedoom2.wad\0";
        }
    } else if gamevariant == FREEDM && gamemission == DOOM2 {
        return b"freedm.wad\0";
    }

    IWADS
        .iter()
        .find(|iwad| gamemission == iwad.mission)
        .map_or(b"unknown.wad\0".as_slice(), |iwad| iwad.name)
}

pub fn suggest_iwad_name(mission: i32, mode: i32) -> &'static [u8] {
    IWADS
        .iter()
        .find(|iwad| iwad.mission == mission && iwad.mode == mode)
        .map_or(b"unknown.wad\0".as_slice(), |iwad| iwad.name)
}

pub fn suggest_game_name(mission: i32, mode: i32) -> &'static [u8] {
    IWADS
        .iter()
        .find(|iwad| iwad.mission == mission && (mode == INDETERMINED || iwad.mode == mode))
        .map_or(b"Unknown game?\0".as_slice(), |iwad| iwad.description)
}
