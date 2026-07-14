//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::{CStr, CString};
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
    fn net_expand_tic_num_matches_c_wraparound_rules() {
        let cases = [
            (0x1234, 0x56, 0x1256),
            (0x1205, 0xf0, 0x11f0),
            (0x12f0, 0x05, 0x1305),
            (0x0000, 0xf0, 0xfffffff0),
            (0x12b1, 0x3f, 0x133f),
            (0x123f, 0xb1, 0x11b1),
        ];

        for (relative, low_byte, expected) in cases {
            assert_eq!(
                cdoom_core::cdoom_rust_net_expand_tic_num(relative, low_byte),
                expected
            );
        }
    }

    #[test]
    fn net_packet_ffi_writes_chocolate_doom_big_endian_bytes() {
        let mut packet = [0u8; 8];
        let mut len = 0usize;

        assert_eq!(
            cdoom_core::cdoom_rust_net_write_int8(
                packet[len..].as_mut_ptr(),
                packet.len() - len,
                0xab
            ),
            1
        );
        len += 1;
        assert_eq!(
            cdoom_core::cdoom_rust_net_write_int16(
                packet[len..].as_mut_ptr(),
                packet.len() - len,
                0xcdef
            ),
            1
        );
        len += 2;
        assert_eq!(
            cdoom_core::cdoom_rust_net_write_int32(
                packet[len..].as_mut_ptr(),
                packet.len() - len,
                0x12345678
            ),
            1
        );
        len += 4;

        assert_eq!(&packet[..len], &[0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn net_packet_ffi_reads_unsigned_signed_and_preserves_pos_on_failure() {
        let packet = [0xab, 0xcd, 0xef, 0x80, 0x00, 0x80, 0x00, 0x00, 0x00];
        let mut pos = 0u32;
        let mut unsigned = 0u32;
        let mut signed = 0i32;

        assert_eq!(
            cdoom_core::cdoom_rust_net_read_int8(
                packet.as_ptr(),
                packet.len(),
                &mut pos,
                &mut unsigned,
            ),
            1
        );
        assert_eq!((pos, unsigned), (1, 0xab));

        assert_eq!(
            cdoom_core::cdoom_rust_net_read_int16(
                packet.as_ptr(),
                packet.len(),
                &mut pos,
                &mut unsigned,
            ),
            1
        );
        assert_eq!((pos, unsigned), (3, 0xcdef));

        assert_eq!(
            cdoom_core::cdoom_rust_net_read_sint16(
                packet.as_ptr(),
                packet.len(),
                &mut pos,
                &mut signed,
            ),
            1
        );
        assert_eq!((pos, signed), (5, -32768));

        assert_eq!(
            cdoom_core::cdoom_rust_net_read_sint32(
                packet.as_ptr(),
                packet.len(),
                &mut pos,
                &mut signed,
            ),
            1
        );
        assert_eq!((pos, signed), (9, -2147483648));

        pos = 8;
        assert_eq!(
            cdoom_core::cdoom_rust_net_read_int16(
                packet.as_ptr(),
                packet.len(),
                &mut pos,
                &mut unsigned,
            ),
            0
        );
        assert_eq!(pos, 8);
    }

    #[test]
    fn net_protocol_names_match_chocolate_doom_wire_strings() {
        let name = cdoom_core::cdoom_rust_net_protocol_name(
            cdoom_core::net::NET_PROTOCOL_CHOCOLATE_DOOM_0,
        );
        assert!(!name.is_null());
        let name = unsafe { CStr::from_ptr(name) };
        assert_eq!(name.to_bytes(), b"CHOCOLATE_DOOM_0");

        assert_eq!(
            cdoom_core::cdoom_rust_net_parse_protocol_name(name.as_ptr()),
            cdoom_core::net::NET_PROTOCOL_CHOCOLATE_DOOM_0,
        );

        let unknown = CString::new("CHOCOLATE_DOOM_FUTURE").unwrap();
        assert_eq!(
            cdoom_core::cdoom_rust_net_parse_protocol_name(unknown.as_ptr()),
            cdoom_core::net::NET_PROTOCOL_UNKNOWN,
        );

        assert_eq!(
            cdoom_core::net::write_protocol_list(),
            b"\x01CHOCOLATE_DOOM_0\0"
        );
    }
}
