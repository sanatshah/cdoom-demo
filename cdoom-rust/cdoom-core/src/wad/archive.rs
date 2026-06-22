//! Standalone WAD reader for verification and tooling (no C runtime required).

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::wad::format::{FileLump, WadHeader, WAD_HEADER_SIZE, FILE_LUMP_SIZE, WadKind, PWAD_LUMP_LIMIT};

/// Parsed lump metadata from a WAD directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LumpEntry {
    pub name: [u8; 8],
    pub offset: u32,
    pub size: u32,
}

/// A read-only view of a WAD file on disk.
#[derive(Debug, Clone)]
pub struct WadArchive {
    path: PathBuf,
    pub kind: WadKind,
    pub lumps: Vec<LumpEntry>,
}

impl WadArchive {
    /// Open and parse a `.wad` file.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path)?;

        let mut header_bytes = [0u8; WAD_HEADER_SIZE];
        file.read_exact(&mut header_bytes)?;
        let header: WadHeader = unsafe { std::ptr::read(header_bytes.as_ptr() as *const WadHeader) };

        let kind = WadKind::from_ident(&header.identification)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing IWAD/PWAD id"))?;

        let numlumps = u32::from_le_bytes(header.numlumps.to_le_bytes());
        let infotableofs = u32::from_le_bytes(header.infotableofs.to_le_bytes());

        if kind == WadKind::Pwad && numlumps > PWAD_LUMP_LIMIT as u32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("PWAD lump count {numlumps} exceeds vanilla limit {PWAD_LUMP_LIMIT}"),
            ));
        }

        file.seek(SeekFrom::Start(u64::from(infotableofs)))?;
        let mut lumps = Vec::with_capacity(numlumps as usize);
        for _ in 0..numlumps {
            let mut lump_bytes = [0u8; FILE_LUMP_SIZE];
            file.read_exact(&mut lump_bytes)?;
            let entry: FileLump = unsafe { std::ptr::read(lump_bytes.as_ptr() as *const FileLump) };
            lumps.push(LumpEntry {
                name: entry.name,
                offset: u32::from_le_bytes(entry.filepos.to_le_bytes()),
                size: u32::from_le_bytes(entry.size.to_le_bytes()),
            });
        }

        Ok(Self { path, kind, lumps })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn num_lumps(&self) -> usize {
        self.lumps.len()
    }

    /// Last matching lump wins (PWAD override semantics).
    pub fn check_num_for_name(&self, name: &str) -> Option<usize> {
        self.lumps
            .iter()
            .enumerate()
            .rev()
            .find(|(_, lump)| lump_name_matches(name, &lump.name))
            .map(|(i, _)| i)
    }

    pub fn lump_length(&self, index: usize) -> Option<u32> {
        self.lumps.get(index).map(|l| l.size)
    }

    pub fn read_lump(&self, index: usize) -> io::Result<Vec<u8>> {
        let lump = self
            .lumps
            .get(index)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "lump index out of range"))?;
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(u64::from(lump.offset)))?;
        let mut data = vec![0u8; lump.size as usize];
        file.read_exact(&mut data)?;
        Ok(data)
    }

}

fn lump_name_matches(name: &str, lump_name: &[u8; 8]) -> bool {
    let mut buf = [0u8; 8];
    let len = name.len().min(8);
    buf[..len].copy_from_slice(&name.as_bytes()[..len]);
    lump_name_field_eq(&buf, lump_name)
}

fn lump_name_field_eq(a: &[u8; 8], b: &[u8; 8]) -> bool {
    for i in 0..8 {
        let ca = if a[i] == 0 { 0 } else { a[i].to_ascii_uppercase() };
        let cb = if b[i] == 0 { 0 } else { b[i].to_ascii_uppercase() };
        if ca != cb {
            return false;
        }
        if ca == 0 {
            break;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn freedoom1_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../wads/freedoom1.wad")
    }

    #[test]
    fn opens_freedoom1_when_present() {
        let path = freedoom1_path();
        if !path.is_file() {
            return;
        }
        let wad = WadArchive::open(&path).expect("parse freedoom1");
        assert_eq!(wad.kind, WadKind::Iwad);
        assert!(wad.num_lumps() > 3000, "expected thousands of lumps, got {}", wad.num_lumps());
    }

    #[test]
    fn finds_playpal_in_freedoom1() {
        let path = freedoom1_path();
        if !path.is_file() {
            return;
        }
        let wad = WadArchive::open(&path).unwrap();
        let idx = wad.check_num_for_name("PLAYPAL").expect("PLAYPAL lump");
        assert!(wad.lump_length(idx).unwrap() > 0);
    }
}
