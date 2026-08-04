//! Chocolate Doom zone allocator port.
//!
//! This intentionally mirrors `z_zone.c`: it owns one contiguous memory zone,
//! stores C-compatible block headers inside that zone, and preserves rover,
//! tag, purge, user-pointer, and coalescing behavior.

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::os::raw::{c_int, c_uint};
use std::ptr::{null_mut, write_bytes};

const MEM_ALIGN: usize = size_of::<*mut c_void>();
const ZONEID: c_int = 0x1d4a11;
const MINFRAGMENT: c_int = 64;

const PU_STATIC: c_int = 1;
const PU_FREE: c_int = 4;
const PU_LEVEL: c_int = 5;
const PU_LEVSPEC: c_int = 6;
const PU_PURGELEVEL: c_int = 7;

#[repr(C)]
struct MemBlock {
    size: c_int,
    user: *mut *mut c_void,
    tag: c_int,
    id: c_int,
    next: *mut MemBlock,
    prev: *mut MemBlock,
}

#[repr(C)]
struct MemZone {
    size: c_int,
    blocklist: MemBlock,
    rover: *mut MemBlock,
}

#[derive(Clone, Copy, Default)]
struct ZoneStats {
    allocs: u64,
    frees: u64,
    purges: u64,
    free_tags: u64,
}

static mut MAINZONE: *mut MemZone = null_mut();
static mut ZERO_ON_FREE: bool = false;
static mut SCAN_ON_FREE: bool = false;
static mut STATS: ZoneStats = ZoneStats {
    allocs: 0,
    frees: 0,
    purges: 0,
    free_tags: 0,
};

unsafe fn block_data(block: *mut MemBlock) -> *mut c_void {
    (block.cast::<u8>()).add(size_of::<MemBlock>()).cast()
}

unsafe fn block_from_data(ptr: *mut c_void) -> *mut MemBlock {
    (ptr.cast::<u8>()).sub(size_of::<MemBlock>()).cast()
}

unsafe fn blocklist(zone: *mut MemZone) -> *mut MemBlock {
    &mut (*zone).blocklist
}

unsafe fn clear_zone(zone: *mut MemZone) {
    let block = (zone.cast::<u8>())
        .add(size_of::<MemZone>())
        .cast::<MemBlock>();
    let sentinel = blocklist(zone);

    (*zone).blocklist.next = block;
    (*zone).blocklist.prev = block;
    (*zone).blocklist.user = zone.cast();
    (*zone).blocklist.tag = PU_STATIC;
    (*zone).blocklist.id = 0;
    (*zone).blocklist.size = 0;
    (*zone).rover = block;

    (*block).prev = sentinel;
    (*block).next = sentinel;
    (*block).tag = PU_FREE;
    (*block).id = 0;
    (*block).user = null_mut();
    (*block).size = (*zone).size - size_of::<MemZone>() as c_int;
}

pub unsafe fn init(
    zone_memory: *mut c_void,
    size: c_int,
    zero_on_free: bool,
    scan_on_free: bool,
) -> c_int {
    if zone_memory.is_null() || size <= (size_of::<MemZone>() + size_of::<MemBlock>()) as c_int {
        return 0;
    }

    let zone = zone_memory.cast::<MemZone>();
    (*zone).size = size;
    clear_zone(zone);

    MAINZONE = zone;
    ZERO_ON_FREE = zero_on_free;
    SCAN_ON_FREE = scan_on_free;
    STATS = zeroed();
    1
}

pub unsafe fn reset_stats() {
    STATS = zeroed();
}

pub unsafe fn alloc_count() -> u64 {
    STATS.allocs
}

pub unsafe fn free_count() -> u64 {
    STATS.frees
}

pub unsafe fn purge_count() -> u64 {
    STATS.purges
}

pub unsafe fn free_tags_count() -> u64 {
    STATS.free_tags
}

unsafe fn mainzone() -> Option<*mut MemZone> {
    if MAINZONE.is_null() {
        None
    } else {
        Some(MAINZONE)
    }
}

unsafe fn scan_for_block(start: *mut c_void, end: *mut c_void) {
    let Some(zone) = mainzone() else {
        return;
    };
    let sentinel = blocklist(zone);
    let mut block = (*sentinel).next;

    while (*block).next != sentinel {
        let tag = (*block).tag;

        if tag == PU_STATIC || tag == PU_LEVEL || tag == PU_LEVSPEC {
            let mem = block_data(block).cast::<*mut c_void>();
            let len = ((*block).size as usize - size_of::<MemBlock>()) / size_of::<*mut c_void>();

            for i in 0..len {
                let slot = mem.add(i);
                let value = *slot;
                if start <= value && value <= end {
                    eprintln!(
                        "{:p} has dangling pointer into freed block {:p} ({:p} -> {:p})",
                        mem, start, slot, value
                    );
                }
            }
        }

        block = (*block).next;
    }
}

unsafe fn free_impl(ptr: *mut c_void, purged: bool) -> c_int {
    let Some(zone) = mainzone() else {
        return 0;
    };
    if ptr.is_null() {
        return 0;
    }

    let mut block = block_from_data(ptr);

    if (*block).id != ZONEID {
        return 0;
    }

    if (*block).tag != PU_FREE && !(*block).user.is_null() {
        *(*block).user = null_mut();
    }

    (*block).tag = PU_FREE;
    (*block).user = null_mut();
    (*block).id = 0;

    if ZERO_ON_FREE {
        write_bytes(ptr, 0, (*block).size as usize - size_of::<MemBlock>());
    }

    if SCAN_ON_FREE {
        scan_for_block(
            ptr,
            (ptr.cast::<u8>())
                .add((*block).size as usize - size_of::<MemBlock>())
                .cast(),
        );
    }

    let mut other = (*block).prev;

    if (*other).tag == PU_FREE {
        (*other).size += (*block).size;
        (*other).next = (*block).next;
        (*(*other).next).prev = other;

        if block == (*zone).rover {
            (*zone).rover = other;
        }

        block = other;
    }

    other = (*block).next;
    if (*other).tag == PU_FREE {
        (*block).size += (*other).size;
        (*block).next = (*other).next;
        (*(*block).next).prev = block;

        if other == (*zone).rover {
            (*zone).rover = block;
        }
    }

    if purged {
        STATS.purges += 1;
    } else {
        STATS.frees += 1;
    }

    1
}

pub unsafe fn free(ptr: *mut c_void) -> c_int {
    free_impl(ptr, false)
}

pub unsafe fn malloc(size: c_int, tag: c_int, user: *mut c_void) -> *mut c_void {
    let Some(zone) = mainzone() else {
        return null_mut();
    };
    if size < 0 || (user.is_null() && tag >= PU_PURGELEVEL) {
        return null_mut();
    }

    let aligned = ((size as usize) + MEM_ALIGN - 1) & !(MEM_ALIGN - 1);
    let Ok(mut needed) = c_int::try_from(aligned + size_of::<MemBlock>()) else {
        return null_mut();
    };

    let mut base = (*zone).rover;
    if (*(*base).prev).tag == PU_FREE {
        base = (*base).prev;
    }

    let mut rover = base;
    let start = (*base).prev;

    loop {
        if rover == start {
            return null_mut();
        }

        if (*rover).tag != PU_FREE {
            if (*rover).tag < PU_PURGELEVEL {
                base = (*rover).next;
                rover = base;
            } else {
                base = (*base).prev;
                let purged_ptr = block_data(rover);
                if free_impl(purged_ptr, true) == 0 {
                    return null_mut();
                }
                base = (*base).next;
                rover = (*base).next;
            }
        } else {
            rover = (*rover).next;
        }

        if (*base).tag == PU_FREE && (*base).size >= needed {
            break;
        }
    }

    let extra = (*base).size - needed;

    if extra > MINFRAGMENT {
        let newblock = (base.cast::<u8>()).add(needed as usize).cast::<MemBlock>();
        (*newblock).size = extra;
        (*newblock).tag = PU_FREE;
        (*newblock).id = 0;
        (*newblock).user = null_mut();
        (*newblock).prev = base;
        (*newblock).next = (*base).next;
        (*(*newblock).next).prev = newblock;

        (*base).next = newblock;
        (*base).size = needed;
    } else {
        needed = (*base).size;
    }

    (*base).user = user.cast();
    (*base).tag = tag;

    let result = block_data(base);

    if !(*base).user.is_null() {
        *(*base).user = result;
    }

    (*zone).rover = (*base).next;
    (*base).id = ZONEID;
    (*base).size = needed;
    STATS.allocs += 1;

    result
}

pub unsafe fn free_tags(lowtag: c_int, hightag: c_int) {
    let Some(zone) = mainzone() else {
        return;
    };
    let sentinel = blocklist(zone);
    let mut block = (*sentinel).next;

    while block != sentinel {
        let next = (*block).next;

        if (*block).tag != PU_FREE && (*block).tag >= lowtag && (*block).tag <= hightag {
            let _ = free_impl(block_data(block), false);
        }

        block = next;
    }

    STATS.free_tags += 1;
}

pub unsafe fn check_heap() -> c_int {
    let Some(zone) = mainzone() else {
        return 0;
    };
    let sentinel = blocklist(zone);
    let mut block = (*sentinel).next;

    loop {
        if (*block).next == sentinel {
            break;
        }

        if (block.cast::<u8>()).add((*block).size as usize) != (*block).next.cast::<u8>() {
            return 0;
        }

        if (*(*block).next).prev != block {
            return 0;
        }

        if (*block).tag == PU_FREE && (*(*block).next).tag == PU_FREE {
            return 0;
        }

        block = (*block).next;
    }

    1
}

pub unsafe fn change_tag(ptr: *mut c_void, tag: c_int) -> c_int {
    if ptr.is_null() {
        return 0;
    }

    let block = block_from_data(ptr);

    if (*block).id != ZONEID {
        return 0;
    }

    if tag >= PU_PURGELEVEL && (*block).user.is_null() {
        return -1;
    }

    (*block).tag = tag;
    1
}

pub unsafe fn change_user(ptr: *mut c_void, user: *mut c_void) -> c_int {
    if ptr.is_null() || user.is_null() {
        return 0;
    }

    let block = block_from_data(ptr);

    if (*block).id != ZONEID {
        return 0;
    }

    (*block).user = user.cast();
    *(*block).user = ptr;
    1
}

pub unsafe fn free_memory() -> c_int {
    let Some(zone) = mainzone() else {
        return 0;
    };
    let sentinel = blocklist(zone);
    let mut free = 0;
    let mut block = (*sentinel).next;

    while block != sentinel {
        if (*block).tag == PU_FREE || (*block).tag >= PU_PURGELEVEL {
            free += (*block).size;
        }

        block = (*block).next;
    }

    free
}

pub unsafe fn zone_size() -> c_uint {
    match mainzone() {
        Some(zone) => (*zone).size as c_uint,
        None => 0,
    }
}
