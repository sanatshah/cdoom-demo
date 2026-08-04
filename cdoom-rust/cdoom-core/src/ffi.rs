//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use std::ffi::c_void;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uint};
use std::sync::OnceLock;

use crate::z_zone;

static VERSION: OnceLock<CString> = OnceLock::new();

fn version_cstr() -> &'static CString {
    VERSION.get_or_init(|| {
        CString::new(env!("CARGO_PKG_VERSION")).expect("version must not contain NUL")
    })
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
pub extern "C" fn cdoom_rust_z_init(
    zone_memory: *mut c_void,
    size: c_int,
    zero_on_free: c_int,
    scan_on_free: c_int,
) -> c_int {
    // SAFETY: C passes the zone memory returned by I_ZoneBase, and Rust only
    // writes allocator metadata inside that region.
    unsafe { z_zone::init(zone_memory, size, zero_on_free != 0, scan_on_free != 0) }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_malloc(size: c_int, tag: c_int, user: *mut c_void) -> *mut c_void {
    // SAFETY: The C wrapper preserves z_zone.c's public contract.
    unsafe { z_zone::malloc(size, tag, user) }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_free(ptr: *mut c_void) -> c_int {
    // SAFETY: The C wrapper passes pointers previously returned by Z_Malloc.
    unsafe { z_zone::free(ptr) }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_free_tags(lowtag: c_int, hightag: c_int) {
    // SAFETY: Tags are plain integers and the global zone is initialized by Z_Init.
    unsafe { z_zone::free_tags(lowtag, hightag) }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_check_heap() -> c_int {
    // SAFETY: Heap metadata lives inside the initialized zone.
    unsafe { z_zone::check_heap() }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_change_tag(ptr: *mut c_void, tag: c_int) -> c_int {
    // SAFETY: The C wrapper passes a live zone pointer and maps failures to I_Error.
    unsafe { z_zone::change_tag(ptr, tag) }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_change_user(ptr: *mut c_void, user: *mut c_void) -> c_int {
    // SAFETY: The C wrapper passes a live zone pointer and a non-NULL owner slot.
    unsafe { z_zone::change_user(ptr, user) }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_free_memory() -> c_int {
    // SAFETY: Heap metadata lives inside the initialized zone.
    unsafe { z_zone::free_memory() }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_zone_size() -> c_uint {
    // SAFETY: Reading the initialized global zone size does not mutate memory.
    unsafe { z_zone::zone_size() }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_reset_stats() {
    // SAFETY: Stats are process-local allocator counters.
    unsafe { z_zone::reset_stats() }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_alloc_count() -> u64 {
    // SAFETY: Stats are process-local allocator counters.
    unsafe { z_zone::alloc_count() }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_free_count() -> u64 {
    // SAFETY: Stats are process-local allocator counters.
    unsafe { z_zone::free_count() }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_purge_count() -> u64 {
    // SAFETY: Stats are process-local allocator counters.
    unsafe { z_zone::purge_count() }
}

#[no_mangle]
pub extern "C" fn cdoom_rust_z_free_tags_count() -> u64 {
    // SAFETY: Stats are process-local allocator counters.
    unsafe { z_zone::free_tags_count() }
}
