//! Global WAD state shared with Chocolate Doom (`lumpinfo`, `numlumps`).

use std::ptr;

use crate::wad::c_bindings::{LumpInfo, WadFile};

/// Lump directory; each entry points at storage owned by a WAD file batch.
#[no_mangle]
pub static mut lumpinfo: *mut *mut LumpInfo = ptr::null_mut();

/// Total lump count across all loaded WADs.
#[no_mangle]
pub static mut numlumps: u32 = 0;

/// Reload-hack state and hash table (single-threaded, like the C original).
pub(crate) struct WadGlobals {
    pub lumphash: Option<Box<[i32]>>,
    pub reload_name: Option<String>,
    pub reload_lump: i32,
    pub reload_handle: *mut WadFile,
    pub reload_lumps: *mut LumpInfo,
}

impl WadGlobals {
    pub const fn new() -> Self {
        Self {
            lumphash: None,
            reload_name: None,
            reload_lump: -1,
            reload_handle: ptr::null_mut(),
            reload_lumps: ptr::null_mut(),
        }
    }
}

pub(crate) static mut WAD_EXTRA: WadGlobals = WadGlobals::new();
