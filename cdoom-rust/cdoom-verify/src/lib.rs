//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::path::Path;
use std::ptr;

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
    fn memio_ffi_read_write_and_seek_edges_match_c_contract() {
        let mut source = *b"abcdef";
        let read_stream =
            unsafe { cdoom_core::cdoom_rust_mem_fopen_read(source.as_mut_ptr().cast(), 6) };
        let mut out = [0_u8; 8];

        assert_eq!(
            unsafe { cdoom_core::cdoom_rust_mem_fread(out.as_mut_ptr().cast(), 2, 4, read_stream) },
            3
        );
        assert_eq!(&out[..6], b"abcdef");
        assert_eq!(unsafe { cdoom_core::cdoom_rust_mem_ftell(read_stream) }, 6);
        assert_eq!(
            unsafe { cdoom_core::cdoom_rust_mem_fseek(read_stream, 6, 0) },
            -1
        );
        assert_eq!(
            unsafe { cdoom_core::cdoom_rust_mem_fseek(read_stream, -1, 2) },
            0
        );
        unsafe { cdoom_core::cdoom_rust_mem_fclose(read_stream) };

        let write_stream = cdoom_core::cdoom_rust_mem_fopen_write();
        let payload = vec![0x5a; 1500];
        assert_eq!(
            unsafe {
                cdoom_core::cdoom_rust_mem_fwrite(
                    payload.as_ptr().cast(),
                    1,
                    payload.len(),
                    write_stream,
                )
            },
            payload.len()
        );

        let mut buf = ptr::null_mut();
        let mut buflen = 0;
        unsafe { cdoom_core::cdoom_rust_mem_get_buf(write_stream, &mut buf, &mut buflen) };
        assert_eq!(buflen, payload.len());
        let written = unsafe { std::slice::from_raw_parts(buf.cast::<u8>(), buflen) };
        assert_eq!(written, payload.as_slice());
        unsafe { cdoom_core::cdoom_rust_mem_fclose(write_stream) };
    }

    #[test]
    fn sha1_ffi_known_answers_and_game_helpers_match() {
        let mut context = cdoom_core::sha1::Sha1Context::default();
        let mut digest = [0_u8; cdoom_core::sha1::DIGEST_LEN];

        unsafe {
            cdoom_core::cdoom_rust_sha1_init((&mut context as *mut _).cast());
            cdoom_core::cdoom_rust_sha1_update(
                (&mut context as *mut _).cast(),
                b"abc".as_ptr().cast(),
                3,
            );
            cdoom_core::cdoom_rust_sha1_final(
                digest.as_mut_ptr().cast(),
                (&mut context as *mut _).cast(),
            );
        }
        assert_eq!(
            digest,
            [
                0xa9, 0x99, 0x3e, 0x36, 0x47, 0x06, 0x81, 0x6a, 0xba, 0x3e, 0x25, 0x71, 0x78, 0x50,
                0xc2, 0x6c, 0x9c, 0xd0, 0xd8, 0x9d,
            ]
        );

        let mut via_helpers = cdoom_core::sha1::Sha1Context::default();
        unsafe {
            cdoom_core::cdoom_rust_sha1_update_int32((&mut via_helpers as *mut _).cast(), 17);
            cdoom_core::cdoom_rust_sha1_update_string(
                (&mut via_helpers as *mut _).cast(),
                c"PLAYPAL".as_ptr(),
            );
            cdoom_core::cdoom_rust_sha1_final(
                digest.as_mut_ptr().cast(),
                (&mut via_helpers as *mut _).cast(),
            );
        }

        assert_eq!(
            digest,
            cdoom_core::sha1::digest_bytes(&[
                0, 0, 0, 17, b'P', b'L', b'A', b'Y', b'P', b'A', b'L', 0
            ])
        );
    }

    #[test]
    fn aes_prng_ffi_known_answers_and_stop_behavior_match() {
        let seed = [0_u8; 16];
        unsafe { cdoom_core::cdoom_rust_prng_start(seed.as_ptr().cast()) };
        let values = [
            cdoom_core::cdoom_rust_prng_random(),
            cdoom_core::cdoom_rust_prng_random(),
            cdoom_core::cdoom_rust_prng_random(),
            cdoom_core::cdoom_rust_prng_random(),
        ];
        assert_eq!(values, [0xe53e_64e8, 0x0cf4_962e, 0x3fe6_1c56, 0x0edc_8533]);

        cdoom_core::cdoom_rust_prng_stop();
        assert_eq!(cdoom_core::cdoom_rust_prng_random(), 0);
    }
}
