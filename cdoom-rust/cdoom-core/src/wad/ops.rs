//! WAD operations exported to Chocolate Doom (replaces `w_wad.c` when `USE_RUST_WAD`).

use std::ffi::{c_char, c_int, c_ulong, CStr, CString};
use std::ptr;

use libc::{calloc, free, strncpy};

use crate::wad::archive::WadArchive;
use crate::wad::c_bindings::{
    self, LumpIndex, LumpInfo, WadFile, PU_CACHE, PU_STATIC,
};
use crate::wad::format::{
    FileLump, WadHeader, WadKind, FILE_LUMP_SIZE, PWAD_LUMP_LIMIT, WAD_HEADER_SIZE,
};
use crate::wad::globals::{self, WadGlobals, WAD_EXTRA};
use crate::wad::hash::{lump_name_hash, lump_name_hash_bytes};

fn fatal(msg: &str) -> ! {
    let cmsg = CString::new(msg).unwrap_or_else(|_| CString::new("WAD error").unwrap());
    unsafe { c_bindings::I_Error(cmsg.as_ptr()) }
}

fn c_str<'a>(ptr: *const c_char) -> &'a CStr {
    unsafe { CStr::from_ptr(ptr) }
}

fn swap_le_i32(v: i32) -> i32 {
    i32::from_le_bytes(v.to_le_bytes())
}

fn extract_file_base(path: &str) -> [u8; 8] {
    let mut dest = [0u8; 8];
    let cpath = CString::new(path).expect("path contains NUL");
    unsafe {
        c_bindings::M_ExtractFileBase(cpath.as_ptr(), dest.as_mut_ptr() as *mut c_char);
    }
    dest
}

fn is_wad_extension(path: &str) -> bool {
    if path.len() < 3 {
        return false;
    }
    let suffix = &path[path.len() - 3..];
    let cwad = CString::new("wad").unwrap();
    let csuffix = CString::new(suffix).unwrap();
    unsafe { c_bindings::strcasecmp(csuffix.as_ptr(), cwad.as_ptr()) == 0 }
}

fn wad_extra() -> &'static mut WadGlobals {
    unsafe { &mut *core::ptr::addr_of_mut!(WAD_EXTRA) }
}

/// Add a WAD or single-lump file to the lump directory.
#[no_mangle]
pub extern "C" fn W_AddFile(filename: *const c_char) -> *mut WadFile {
    if filename.is_null() {
        return ptr::null_mut();
    }

    let original = c_str(filename).to_string_lossy().into_owned();
    let mut path_str = original.clone();
    let extra = wad_extra();

    if original.starts_with('~') {
        if extra.reload_name.is_some() {
            fatal(
                "Prefixing a WAD filename with '~' indicates that the WAD should be reloaded\n\
                 on each level restart, for use by level authors for rapid development. You\n\
                 can only reload one WAD file, and it must be the last file in the -file list.",
            );
        }
        extra.reload_lump = unsafe { globals::numlumps as i32 };
        extra.reload_name = Some(original);
        path_str = path_str[1..].to_string();
    }

    let cpath = CString::new(path_str.as_str()).expect("path contains NUL");
    let wad_file = unsafe { c_bindings::W_OpenFile(cpath.as_ptr()) };
    if wad_file.is_null() {
        unsafe {
            let fmt = CString::new(" couldn't open %s\n").unwrap();
            c_bindings::printf(fmt.as_ptr(), cpath.as_ptr());
        }
        return ptr::null_mut();
    }

    let (numfilelumps, fileinfo_ptr) = if !is_wad_extension(&path_str) {
        let mut fileinfo: FileLump = unsafe { std::mem::zeroed() };
        fileinfo.filepos = swap_le_i32(0);
        fileinfo.size = swap_le_i32(unsafe { (*wad_file).length as i32 });
        fileinfo.name = extract_file_base(&path_str);

        let ptr = unsafe { c_bindings::Z_Malloc(FILE_LUMP_SIZE as i32, PU_STATIC, ptr::null_mut()) }
            as *mut FileLump;
        unsafe {
            ptr.copy_from_nonoverlapping(&fileinfo, 1);
        }
        (1, ptr)
    } else {
        let mut header = WadHeader {
            identification: [0; 4],
            numlumps: 0,
            infotableofs: 0,
        };
        let read = unsafe {
            c_bindings::W_Read(
                wad_file,
                0,
                &mut header as *mut WadHeader as *mut std::ffi::c_void,
                WAD_HEADER_SIZE,
            )
        };
        if read < WAD_HEADER_SIZE {
            unsafe { c_bindings::W_CloseFile(wad_file) };
            fatal(&format!("Wad file {path_str} read error"));
        }

        let kind = WadKind::from_ident(&header.identification);
        if kind.is_none() {
            unsafe { c_bindings::W_CloseFile(wad_file) };
            fatal(&format!("Wad file {path_str} doesn't have IWAD or PWAD id\n"));
        }

        let numlumps = swap_le_i32(header.numlumps);
        if kind == Some(WadKind::Pwad) && numlumps > PWAD_LUMP_LIMIT {
            unsafe { c_bindings::W_CloseFile(wad_file) };
            fatal(&format!(
                "Error: Vanilla limit for lumps in a WAD is 4046, PWAD {path_str} has {numlumps}"
            ));
        }

        let infotableofs = swap_le_i32(header.infotableofs) as u32;
        let length = numlumps as usize * FILE_LUMP_SIZE;
        let fileinfo = unsafe {
            c_bindings::Z_Malloc(length as i32, PU_STATIC, ptr::null_mut()) as *mut FileLump
        };
        let read = unsafe {
            c_bindings::W_Read(
                wad_file,
                infotableofs,
                fileinfo as *mut std::ffi::c_void,
                length,
            )
        };
        if read < length {
            unsafe {
                c_bindings::Z_Free(fileinfo as *mut std::ffi::c_void);
                c_bindings::W_CloseFile(wad_file);
            }
            fatal(&format!("Wad file {path_str} directory read error"));
        }
        (numlumps, fileinfo)
    };

    let filelumps = unsafe {
        calloc(numfilelumps as usize, std::mem::size_of::<LumpInfo>()) as *mut LumpInfo
    };
    if filelumps.is_null() {
        unsafe {
            c_bindings::Z_Free(fileinfo_ptr as *mut std::ffi::c_void);
            c_bindings::W_CloseFile(wad_file);
        }
        fatal("Failed to allocate array for lumps from new file.");
    }

    let startlump = unsafe { globals::numlumps };
    unsafe {
        globals::numlumps += numfilelumps as u32;
        globals::lumpinfo = c_bindings::I_Realloc(
            globals::lumpinfo as *mut std::ffi::c_void,
            globals::numlumps as usize * std::mem::size_of::<*mut LumpInfo>(),
        ) as *mut *mut LumpInfo;
    }

    let filerover = fileinfo_ptr;
    for i in startlump..unsafe { globals::numlumps } {
        let lump_p = unsafe { filelumps.add((i - startlump) as usize) };
        unsafe {
            (*lump_p).wad_file = wad_file;
            (*lump_p).position = swap_le_i32((*filerover.add((i - startlump) as usize)).filepos);
            (*lump_p).size = swap_le_i32((*filerover.add((i - startlump) as usize)).size);
            (*lump_p).cache = ptr::null_mut();
            strncpy(
                (*lump_p).name.as_mut_ptr() as *mut c_char,
                (*filerover.add((i - startlump) as usize)).name.as_ptr() as *const c_char,
                8,
            );
            *globals::lumpinfo.add(i as usize) = lump_p;
        }
    }

    unsafe {
        c_bindings::Z_Free(fileinfo_ptr as *mut std::ffi::c_void);
    }

    let extra = wad_extra();
    extra.lumphash = None;

    if extra.reload_name.is_some() {
        extra.reload_handle = wad_file;
        extra.reload_lumps = filelumps;
    }

    wad_file
}

#[no_mangle]
pub extern "C" fn W_NumLumps() -> c_int {
    unsafe { globals::numlumps as c_int }
}

#[no_mangle]
pub extern "C" fn W_LumpNameHash(name: *const c_char) -> u32 {
    if name.is_null() {
        return lump_name_hash("");
    }
    lump_name_hash(&c_str(name).to_string_lossy())
}

#[no_mangle]
pub extern "C" fn W_CheckNumForName(name: *const c_char) -> LumpIndex {
    if name.is_null() {
        return -1;
    }
    let name = c_str(name);
    let extra = wad_extra();
    let count = unsafe { globals::numlumps };

    if let Some(ref lumphash) = extra.lumphash {
        let hash = (W_LumpNameHash(name.as_ptr()) as usize) % count as usize;
        let mut i = lumphash[hash];
        while i != -1 {
            let lump = unsafe { *globals::lumpinfo.add(i as usize) };
            if lump_name_matches_c(name, unsafe { &(*lump).name }) {
                return i;
            }
            i = unsafe { (*lump).next };
        }
        -1
    } else {
        let mut i = count as i32 - 1;
        while i >= 0 {
            let lump = unsafe { *globals::lumpinfo.add(i as usize) };
            if lump_name_matches_c(name, unsafe { &(*lump).name }) {
                return i;
            }
            i -= 1;
        }
        -1
    }
}

#[no_mangle]
pub extern "C" fn W_GetNumForName(name: *const c_char) -> LumpIndex {
    let i = W_CheckNumForName(name);
    if i < 0 {
        let cname = if name.is_null() {
            "(null)".to_string()
        } else {
            c_str(name).to_string_lossy().into_owned()
        };
        fatal(&format!("W_GetNumForName: {cname} not found!"));
    }
    i
}

#[no_mangle]
pub extern "C" fn W_LumpLength(lump: LumpIndex) -> c_int {
    let count = unsafe { globals::numlumps };
    if lump < 0 || lump as u32 >= count {
        fatal(&format!("W_LumpLength: {lump} >= numlumps"));
    }
    unsafe { (*(*globals::lumpinfo.add(lump as usize))).size }
}

#[no_mangle]
pub extern "C" fn W_ReadLump(lump: LumpIndex, dest: *mut std::ffi::c_void) {
    let count = unsafe { globals::numlumps };
    if lump < 0 || lump as u32 >= count {
        fatal(&format!("W_ReadLump: {lump} >= numlumps"));
    }
    if dest.is_null() {
        fatal("W_ReadLump: null dest");
    }
    let l = unsafe { *globals::lumpinfo.add(lump as usize) };
    unsafe {
        c_bindings::V_BeginRead((*l).size as usize);
        let c = c_bindings::W_Read((*l).wad_file, (*l).position as u32, dest, (*l).size as usize);
        if c < (*l).size as usize {
            fatal(&format!(
                "W_ReadLump: only read {c} of {} on lump {lump}",
                (*l).size
            ));
        }
    }
}

#[no_mangle]
pub extern "C" fn W_CacheLumpNum(lumpnum: LumpIndex, tag: c_int) -> *mut u8 {
    let count = unsafe { globals::numlumps };
    if lumpnum < 0 || lumpnum as u32 >= count {
        fatal(&format!("W_CacheLumpNum: {lumpnum} >= numlumps"));
    }
    let lump = unsafe { *globals::lumpinfo.add(lumpnum as usize) };
    unsafe {
        if !(*(*lump).wad_file).mapped.is_null() {
            return (*(*lump).wad_file).mapped.add((*lump).position as usize);
        }
        if !(*lump).cache.is_null() {
            c_bindings::Z_ChangeTag2(
                (*lump).cache,
                tag,
                concat!(file!(), "\0").as_ptr() as *const c_char,
                line!() as c_int,
            );
            return (*lump).cache as *mut u8;
        }
        let user = &mut (*lump).cache as *mut *mut std::ffi::c_void;
        (*lump).cache =
            c_bindings::Z_Malloc((*lump).size, tag, user as *mut std::ffi::c_void);
        W_ReadLump(lumpnum, (*lump).cache);
        (*lump).cache as *mut u8
    }
}

#[no_mangle]
pub extern "C" fn W_CacheLumpName(name: *const c_char, tag: c_int) -> *mut u8 {
    W_CacheLumpNum(W_GetNumForName(name), tag)
}

#[no_mangle]
pub extern "C" fn W_ReleaseLumpNum(lumpnum: LumpIndex) {
    let count = unsafe { globals::numlumps };
    if lumpnum < 0 || lumpnum as u32 >= count {
        fatal(&format!("W_ReleaseLumpNum: {lumpnum} >= numlumps"));
    }
    let lump = unsafe { *globals::lumpinfo.add(lumpnum as usize) };
    unsafe {
        if (*(*lump).wad_file).mapped.is_null() {
            c_bindings::Z_ChangeTag2(
                (*lump).cache,
                PU_CACHE,
                concat!(file!(), "\0").as_ptr() as *const c_char,
                line!() as c_int,
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn W_ReleaseLumpName(name: *const c_char) {
    W_ReleaseLumpNum(W_GetNumForName(name));
}

#[no_mangle]
pub extern "C" fn W_GenerateHashTable() {
    let extra = wad_extra();
    let count = unsafe { globals::numlumps };
    if count == 0 {
        extra.lumphash = None;
        return;
    }

    let mut table = vec![-1i32; count as usize];
    for i in 0..count {
        let lump = unsafe { *globals::lumpinfo.add(i as usize) };
        let hash = (lump_name_hash_bytes(unsafe { &(*lump).name }) as usize) % count as usize;
        unsafe {
            (*lump).next = table[hash];
        }
        table[hash] = i as i32;
    }
    extra.lumphash = Some(table.into_boxed_slice());
}

#[no_mangle]
pub extern "C" fn W_Reload() {
    let reload_name = {
        let extra = wad_extra();
        let Some(name) = extra.reload_name.take() else {
            return;
        };
        let reload_lump = extra.reload_lump;
        let reload_handle = extra.reload_handle;
        let reload_lumps = extra.reload_lumps;

        for i in reload_lump as u32..unsafe { globals::numlumps } {
            let lump = unsafe { *globals::lumpinfo.add(i as usize) };
            unsafe {
                if !(*lump).cache.is_null() {
                    c_bindings::Z_Free((*lump).cache);
                }
            }
        }

        unsafe {
            globals::numlumps = reload_lump as u32;
            c_bindings::W_CloseFile(reload_handle);
            free(reload_lumps as *mut std::ffi::c_void);
        }

        extra.reload_handle = ptr::null_mut();
        extra.reload_lumps = ptr::null_mut();
        extra.reload_lump = -1;
        name
    };

    let cname = CString::new(reload_name).expect("reload path");
    W_AddFile(cname.as_ptr());
    W_GenerateHashTable();
}

#[no_mangle]
pub extern "C" fn W_WadNameForLump(lump: *const LumpInfo) -> *const c_char {
    if lump.is_null() {
        return ptr::null();
    }
    unsafe { c_bindings::M_BaseName((*(*lump).wad_file).path) }
}

#[no_mangle]
pub extern "C" fn W_IsIWADLump(lump: *const LumpInfo) -> c_int {
    if lump.is_null() {
        return 0;
    }
    unsafe {
        let first = *globals::lumpinfo;
        if first.is_null() {
            return 0;
        }
        if (*lump).wad_file == (*first).wad_file {
            1
        } else {
            0
        }
    }
}

fn lump_name_matches_c(name: &CStr, lump_name: &[u8; 8]) -> bool {
    let bytes = name.to_bytes();
    let len = bytes.len().min(8);
    unsafe {
        c_bindings::strncasecmp(
            bytes.as_ptr() as *const c_char,
            lump_name.as_ptr() as *const c_char,
            len as c_ulong,
        ) == 0
    }
}

/// Pure-Rust helper used by verification tests.
pub fn check_num_for_name_in_archive(wad: &WadArchive, name: &str) -> Option<usize> {
    wad.check_num_for_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wad::archive::WadArchive;
    use std::path::PathBuf;

    #[test]
    fn hash_matches_c_style_playpal() {
        let h = lump_name_hash("PLAYPAL");
        assert_ne!(h, 5381);
        assert_eq!(h, lump_name_hash("playpal"));
    }

    #[test]
    fn archive_playpal_has_data() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../wads/freedoom1.wad");
        if !path.is_file() {
            return;
        }
        let wad = WadArchive::open(&path).unwrap();
        let idx = wad.check_num_for_name("PLAYPAL").unwrap();
        let data = wad.read_lump(idx).unwrap();
        assert_eq!(data.len(), wad.lump_length(idx).unwrap() as usize);
        assert!(data.len() >= 768);
    }
}
