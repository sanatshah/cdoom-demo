//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::path::Path;

/// Expected Chocolate Doom package version vendored in this repo.
pub const CHOCOLATE_DOOM_VERSION: &str = "3.1.1";

/// Returns `true` when a timedemo baseline can run (binary + IWAD present).
pub fn timedemo_baseline_available(root: &Path) -> bool {
    let binary = root.join("chocolate-doom/build/src/chocolate-doom");
    let wad = root.join("wads/freedoom1.wad");
    binary.is_file() && wad.is_file()
}

/// Reads the exported Rust version string from the C ABI.
pub fn rust_version_from_ffi() -> String {
    let ptr = cdoom_core::cdoom_rust_version();
    assert!(!ptr.is_null());
    // SAFETY: cdoom_rust_version returns a static NUL-terminated string.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::sync::{Mutex, MutexGuard};

    const PU_STATIC: i32 = 1;
    const PU_LEVEL: i32 = 5;
    const PU_LEVSPEC: i32 = 6;
    const PU_CACHE: i32 = 8;

    static ZONE_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct ZoneFixture {
        memory: Vec<usize>,
    }

    impl ZoneFixture {
        fn new(bytes: usize) -> Self {
            let words = bytes.div_ceil(std::mem::size_of::<usize>());
            let mut fixture = Self {
                memory: vec![0; words],
            };
            let byte_len = fixture.memory.len() * std::mem::size_of::<usize>();
            let ok = cdoom_core::cdoom_rust_z_init(
                fixture.memory.as_mut_ptr().cast(),
                byte_len as i32,
                0,
                0,
            );
            assert_eq!(ok, 1);
            fixture
        }
    }

    fn owner_slot() -> *mut c_void {
        null_mut()
    }

    fn zone_test_guard() -> MutexGuard<'static, ()> {
        ZONE_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn ffi_version_matches_crate() {
        assert_eq!(rust_version_from_ffi(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn init_succeeds() {
        assert_eq!(cdoom_core::cdoom_rust_init(), 0);
    }

    #[test]
    fn version_string_is_non_empty() {
        assert!(!cdoom_core::version_string().is_empty());
    }

    #[test]
    fn z_zone_reports_size_and_initial_free_memory() {
        let _guard = zone_test_guard();
        let _zone = ZoneFixture::new(8192);

        let zone_size = cdoom_core::cdoom_rust_z_zone_size();
        let free_memory = cdoom_core::cdoom_rust_z_free_memory();

        assert_eq!(zone_size, 8192);
        assert!(free_memory > 0);
        assert!(free_memory < zone_size as i32);
        assert_eq!(cdoom_core::cdoom_rust_z_check_heap(), 1);
    }

    #[test]
    fn z_zone_malloc_aligns_sets_owner_and_free_coalesces() {
        let _guard = zone_test_guard();
        let _zone = ZoneFixture::new(8192);
        let before = cdoom_core::cdoom_rust_z_free_memory();
        let mut owner = owner_slot();

        let ptr =
            cdoom_core::cdoom_rust_z_malloc(13, PU_STATIC, (&mut owner as *mut *mut c_void).cast());

        assert!(!ptr.is_null());
        assert_eq!(owner, ptr);
        assert_eq!((ptr as usize) % std::mem::size_of::<usize>(), 0);
        assert!(cdoom_core::cdoom_rust_z_free_memory() < before);
        assert_eq!(cdoom_core::cdoom_rust_z_alloc_count(), 1);

        assert_eq!(cdoom_core::cdoom_rust_z_free(ptr), 1);
        assert!(owner.is_null());
        assert_eq!(cdoom_core::cdoom_rust_z_free_memory(), before);
        assert_eq!(cdoom_core::cdoom_rust_z_free_count(), 1);
        assert_eq!(cdoom_core::cdoom_rust_z_check_heap(), 1);
    }

    #[test]
    fn z_zone_free_tags_clears_inclusive_tag_range_only() {
        let _guard = zone_test_guard();
        let _zone = ZoneFixture::new(16384);
        let mut static_owner = owner_slot();
        let mut level_owner = owner_slot();
        let mut levspec_owner = owner_slot();
        let mut cache_owner = owner_slot();

        let static_ptr = cdoom_core::cdoom_rust_z_malloc(
            64,
            PU_STATIC,
            (&mut static_owner as *mut *mut c_void).cast(),
        );
        let _level_ptr = cdoom_core::cdoom_rust_z_malloc(
            64,
            PU_LEVEL,
            (&mut level_owner as *mut *mut c_void).cast(),
        );
        let _levspec_ptr = cdoom_core::cdoom_rust_z_malloc(
            64,
            PU_LEVSPEC,
            (&mut levspec_owner as *mut *mut c_void).cast(),
        );
        let cache_ptr = cdoom_core::cdoom_rust_z_malloc(
            64,
            PU_CACHE,
            (&mut cache_owner as *mut *mut c_void).cast(),
        );

        assert!(!static_ptr.is_null());
        assert!(!cache_ptr.is_null());

        cdoom_core::cdoom_rust_z_free_tags(PU_LEVEL, PU_LEVSPEC);

        assert_eq!(static_owner, static_ptr);
        assert!(level_owner.is_null());
        assert!(levspec_owner.is_null());
        assert_eq!(cache_owner, cache_ptr);
        assert_eq!(cdoom_core::cdoom_rust_z_free_count(), 2);
        assert_eq!(cdoom_core::cdoom_rust_z_free_tags_count(), 1);
        assert_eq!(cdoom_core::cdoom_rust_z_check_heap(), 1);
    }

    #[test]
    fn z_zone_purgable_blocks_are_reclaimed_for_large_allocations() {
        let _guard = zone_test_guard();
        let _zone = ZoneFixture::new(8192);
        let mut cache_a = owner_slot();

        let _a = cdoom_core::cdoom_rust_z_malloc(
            4096,
            PU_CACHE,
            (&mut cache_a as *mut *mut c_void).cast(),
        );
        let _static = cdoom_core::cdoom_rust_z_malloc(1024, PU_STATIC, null_mut());

        let large = cdoom_core::cdoom_rust_z_malloc(3500, PU_STATIC, null_mut());

        assert!(!large.is_null());
        assert!(cdoom_core::cdoom_rust_z_purge_count() > 0);
        assert!(cache_a.is_null());
        assert_eq!(cdoom_core::cdoom_rust_z_check_heap(), 1);
    }

    #[test]
    fn z_zone_change_tag_requires_owner_for_purgable_blocks() {
        let _guard = zone_test_guard();
        let _zone = ZoneFixture::new(8192);
        let ptr = cdoom_core::cdoom_rust_z_malloc(128, PU_STATIC, null_mut());
        let mut owner = owner_slot();

        assert!(!ptr.is_null());
        assert_eq!(cdoom_core::cdoom_rust_z_change_tag(ptr, PU_CACHE), -1);
        assert_eq!(
            cdoom_core::cdoom_rust_z_change_user(ptr, (&mut owner as *mut *mut c_void).cast(),),
            1
        );
        assert_eq!(owner, ptr);
        assert_eq!(cdoom_core::cdoom_rust_z_change_tag(ptr, PU_CACHE), 1);

        cdoom_core::cdoom_rust_z_free_tags(PU_CACHE, PU_CACHE);

        assert!(owner.is_null());
        assert_eq!(cdoom_core::cdoom_rust_z_check_heap(), 1);
    }

    #[test]
    fn z_zone_wad_loader_style_trace_matches_expected_counts() {
        let _guard = zone_test_guard();
        let _zone = ZoneFixture::new(6144);
        cdoom_core::cdoom_rust_z_reset_stats();
        let mut cache_owners = [owner_slot(), owner_slot(), owner_slot()];
        let mut level_owner = owner_slot();
        let mut levspec_owner = owner_slot();

        let directory = cdoom_core::cdoom_rust_z_malloc(512, PU_STATIC, null_mut());
        let lumpinfo = cdoom_core::cdoom_rust_z_malloc(768, PU_STATIC, null_mut());
        let cache_0 = cdoom_core::cdoom_rust_z_malloc(
            3000,
            PU_CACHE,
            (&mut cache_owners[0] as *mut *mut c_void).cast(),
        );
        let cache_1 = cdoom_core::cdoom_rust_z_malloc(
            128,
            PU_CACHE,
            (&mut cache_owners[1] as *mut *mut c_void).cast(),
        );
        let cache_2 = cdoom_core::cdoom_rust_z_malloc(
            128,
            PU_CACHE,
            (&mut cache_owners[2] as *mut *mut c_void).cast(),
        );
        let level = cdoom_core::cdoom_rust_z_malloc(
            128,
            PU_LEVEL,
            (&mut level_owner as *mut *mut c_void).cast(),
        );
        let levspec = cdoom_core::cdoom_rust_z_malloc(
            128,
            PU_LEVSPEC,
            (&mut levspec_owner as *mut *mut c_void).cast(),
        );

        assert!(!directory.is_null());
        assert!(!lumpinfo.is_null());
        assert!(!cache_0.is_null());
        assert!(!cache_1.is_null());
        assert!(!cache_2.is_null());
        assert!(!level.is_null());
        assert!(!levspec.is_null());

        cdoom_core::cdoom_rust_z_free_tags(PU_LEVEL, PU_LEVSPEC);
        let late_static = cdoom_core::cdoom_rust_z_malloc(2800, PU_STATIC, null_mut());

        assert!(!late_static.is_null());
        assert!(level_owner.is_null());
        assert!(levspec_owner.is_null());
        assert_eq!(cdoom_core::cdoom_rust_z_alloc_count(), 8);
        assert_eq!(cdoom_core::cdoom_rust_z_free_count(), 2);
        assert_eq!(cdoom_core::cdoom_rust_z_free_tags_count(), 1);
        assert_eq!(cdoom_core::cdoom_rust_z_purge_count(), 1);
        assert!(cache_owners[0].is_null());
        assert_eq!(cdoom_core::cdoom_rust_z_check_heap(), 1);
    }
}
