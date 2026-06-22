//! WAD parity checks against Freedoom and the C hash implementation.

use std::path::PathBuf;

use cdoom_core::{WadArchive, WadKind};

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn freedoom1_path() -> PathBuf {
    repo_root().join("wads/freedoom1.wad")
}

/// Parse Freedoom Phase 1 and return basic metadata when the WAD is present.
pub fn freedoom1_metadata() -> Option<(WadKind, usize)> {
    let path = freedoom1_path();
    let wad = WadArchive::open(&path).ok()?;
    Some((wad.kind, wad.num_lumps()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cdoom_core::lump_name_hash;

    #[test]
    fn lump_name_hash_playpal_is_stable() {
        // Regression guard: hash must stay stable for save/demo compatibility.
        assert_eq!(lump_name_hash("PLAYPAL"), 2_373_570_428);
    }

    #[test]
    fn freedoom1_is_iwad_with_expected_lumps() {
        let path = freedoom1_path();
        if !path.is_file() {
            eprintln!("skip: {} not found", path.display());
            return;
        }
        let wad = WadArchive::open(&path).expect("open freedoom1");
        assert_eq!(wad.kind, WadKind::Iwad);
        assert!(
            wad.num_lumps() >= 3000,
            "unexpected lump count {}",
            wad.num_lumps()
        );
        assert!(wad.check_num_for_name("PLAYPAL").is_some());
        assert!(wad.check_num_for_name("E1M1").is_some());
    }

    #[test]
    fn playpal_lump_is_non_empty() {
        let path = freedoom1_path();
        if !path.is_file() {
            return;
        }
        let wad = WadArchive::open(&path).unwrap();
        let idx = wad.check_num_for_name("PLAYPAL").unwrap();
        let data = wad.read_lump(idx).unwrap();
        assert!(data.len() >= 768, "PLAYPAL should contain at least one 768-byte palette");
    }

    #[test]
    fn hash_is_case_insensitive_for_eight_chars() {
        assert_eq!(lump_name_hash("pLayPal"), lump_name_hash("PLAYPAL"));
    }
}
