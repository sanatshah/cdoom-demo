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
    use std::sync::Mutex;

    const SCREENWIDTH: usize = cdoom_core::v_video::SCREENWIDTH;
    const SCREENHEIGHT: usize = cdoom_core::v_video::SCREENHEIGHT;
    const SCREEN_PIXELS: usize = SCREENWIDTH * SCREENHEIGHT;
    const BOXTOP: usize = 0;
    const BOXBOTTOM: usize = 1;
    const BOXLEFT: usize = 2;
    const BOXRIGHT: usize = 3;

    static VIDEO_TEST_LOCK: Mutex<()> = Mutex::new(());

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
    fn v_mark_rect_tracks_dirtybox_only_for_video_buffer() {
        let _guard = VIDEO_TEST_LOCK.lock().unwrap();
        let mut video = vec![0; SCREEN_PIXELS];
        let mut dirty = clear_box();
        cdoom_core::cdoom_rust_v_register_buffers(video.as_mut_ptr(), dirty.as_mut_ptr());

        cdoom_core::cdoom_rust_v_mark_rect(10, 20, 4, 3);

        assert_eq!(dirty[BOXLEFT], 10);
        assert_eq!(dirty[BOXRIGHT], 13);
        assert_eq!(dirty[BOXBOTTOM], 20);
        assert_eq!(dirty[BOXTOP], 22);

        let mut alt = vec![0; SCREEN_PIXELS];
        cdoom_core::cdoom_rust_v_use_buffer(alt.as_mut_ptr());
        cdoom_core::cdoom_rust_v_mark_rect(1, 2, 5, 6);

        assert_eq!(dirty[BOXLEFT], 10);
        assert_eq!(dirty[BOXRIGHT], 13);
        assert_eq!(dirty[BOXBOTTOM], 20);
        assert_eq!(dirty[BOXTOP], 22);
    }

    #[test]
    fn v_copy_draw_block_raw_and_box_helpers_match_c_layout() {
        let _guard = VIDEO_TEST_LOCK.lock().unwrap();
        let mut video = vec![0; SCREEN_PIXELS];
        let mut dirty = clear_box();
        cdoom_core::cdoom_rust_v_register_buffers(video.as_mut_ptr(), dirty.as_mut_ptr());

        let source_screen = patterned_screen(3);
        cdoom_core::cdoom_rust_v_copy_rect(4, 5, source_screen.as_ptr(), 6, 3, 20, 30);
        for row in 0..3 {
            for col in 0..6 {
                assert_eq!(
                    video[index(20 + col, 30 + row)],
                    source_screen[index(4 + col, 5 + row)]
                );
            }
        }

        let block: Vec<u8> = (0..12).map(|n| 100 + n).collect();
        cdoom_core::cdoom_rust_v_draw_block(40, 50, 4, 3, block.as_ptr());
        for row in 0..3 {
            for col in 0..4 {
                assert_eq!(video[index(40 + col, 50 + row)], block[row * 4 + col]);
            }
        }

        let raw = patterned_screen(17);
        cdoom_core::cdoom_rust_v_draw_raw_screen(raw.as_ptr());
        assert_eq!(video, raw);

        cdoom_core::cdoom_rust_v_draw_filled_box(2, 3, 4, 2, 77);
        for row in 0..2 {
            for col in 0..4 {
                assert_eq!(video[index(2 + col, 3 + row)], 77);
            }
        }

        cdoom_core::cdoom_rust_v_draw_horiz_line(10, 11, 5, 88);
        assert_eq!(&video[index(10, 11)..index(15, 11)], &[88; 5]);

        cdoom_core::cdoom_rust_v_draw_vert_line(12, 13, 4, 99);
        for row in 0..4 {
            assert_eq!(video[index(12, 13 + row)], 99);
        }

        cdoom_core::cdoom_rust_v_draw_box(30, 31, 5, 4, 111);
        for col in 0..5 {
            assert_eq!(video[index(30 + col, 31)], 111);
            assert_eq!(video[index(30 + col, 34)], 111);
        }
        for row in 0..4 {
            assert_eq!(video[index(30, 31 + row)], 111);
            assert_eq!(video[index(34, 31 + row)], 111);
        }
    }

    #[test]
    fn v_draw_patch_variants_match_post_column_semantics() {
        let _guard = VIDEO_TEST_LOCK.lock().unwrap();
        let mut patch = two_column_patch();

        let mut video = vec![0; SCREEN_PIXELS];
        let mut dirty = clear_box();
        cdoom_core::cdoom_rust_v_register_buffers(video.as_mut_ptr(), dirty.as_mut_ptr());
        cdoom_core::cdoom_rust_v_draw_patch(6, 8, patch_ptr(&mut patch));
        assert_eq!(video[index(5, 7)], 11);
        assert_eq!(video[index(5, 8)], 12);
        assert_eq!(video[index(6, 6)], 21);
        assert_eq!(dirty, [9, 6, 5, 6]);

        let mut flipped = vec![0; SCREEN_PIXELS];
        cdoom_core::cdoom_rust_v_register_buffers(flipped.as_mut_ptr(), dirty.as_mut_ptr());
        cdoom_core::cdoom_rust_v_draw_patch_flipped(6, 8, patch_ptr(&mut patch));
        assert_eq!(flipped[index(5, 6)], 21);
        assert_eq!(flipped[index(6, 7)], 11);
        assert_eq!(flipped[index(6, 8)], 12);
    }

    #[test]
    fn v_translucent_xla_and_shadowed_patches_use_lookup_tables() {
        let _guard = VIDEO_TEST_LOCK.lock().unwrap();
        let mut patch = two_column_patch();
        let tint: Vec<u8> = (0..=u16::MAX)
            .map(|index| ((index & 0xff) as u8) ^ ((index >> 8) as u8))
            .collect();
        let xla: Vec<u8> = (0..=u16::MAX)
            .map(|index| ((index & 0xff) as u8).wrapping_add((index >> 8) as u8))
            .collect();
        let mut dirty = clear_box();

        let mut tl = vec![3; SCREEN_PIXELS];
        cdoom_core::cdoom_rust_v_register_buffers(tl.as_mut_ptr(), dirty.as_mut_ptr());
        cdoom_core::cdoom_rust_v_set_tint_table(tint.as_ptr());
        cdoom_core::cdoom_rust_v_draw_tl_patch(6, 8, patch_ptr(&mut patch));
        assert_eq!(tl[index(5, 7)], tint[3 + (11 << 8)]);
        assert_eq!(tl[index(6, 6)], tint[3 + (21 << 8)]);

        let mut alt = vec![4; SCREEN_PIXELS];
        cdoom_core::cdoom_rust_v_register_buffers(alt.as_mut_ptr(), dirty.as_mut_ptr());
        cdoom_core::cdoom_rust_v_set_tint_table(tint.as_ptr());
        cdoom_core::cdoom_rust_v_draw_alt_tl_patch(6, 8, patch_ptr(&mut patch));
        assert_eq!(alt[index(5, 7)], tint[(4 << 8) + 11]);
        assert_eq!(alt[index(6, 6)], tint[(4 << 8) + 21]);

        let mut xla_buf = vec![5; SCREEN_PIXELS];
        cdoom_core::cdoom_rust_v_register_buffers(xla_buf.as_mut_ptr(), dirty.as_mut_ptr());
        cdoom_core::cdoom_rust_v_set_xla_table(xla.as_ptr());
        cdoom_core::cdoom_rust_v_draw_xla_patch(6, 8, patch_ptr(&mut patch));
        assert_eq!(xla_buf[index(5, 7)], xla[5 + (11 << 8)]);
        assert_eq!(xla_buf[index(6, 6)], xla[5 + (21 << 8)]);

        let mut shadow = vec![6; SCREEN_PIXELS];
        cdoom_core::cdoom_rust_v_register_buffers(shadow.as_mut_ptr(), dirty.as_mut_ptr());
        cdoom_core::cdoom_rust_v_set_tint_table(tint.as_ptr());
        cdoom_core::cdoom_rust_v_draw_shadowed_patch(6, 8, patch_ptr(&mut patch));
        assert_eq!(shadow[index(5, 7)], 11);
        assert_eq!(shadow[index(5, 8)], 12);
        assert_eq!(shadow[index(7, 9)], tint[6 << 8]);
        assert_eq!(shadow[index(7, 10)], tint[6 << 8]);
        assert_eq!(shadow[index(8, 8)], tint[6 << 8]);
    }

    #[test]
    fn v_disk_copy_region_matches_pitch_based_c_copy() {
        let _guard = VIDEO_TEST_LOCK.lock().unwrap();
        let src: Vec<u8> = (0..30).map(|n| n as u8).collect();
        let mut dest = vec![200; 40];

        cdoom_core::cdoom_rust_v_copy_region(dest.as_mut_ptr(), 8, src.as_ptr(), 6, 4, 3);

        for row in 0..3 {
            for col in 0..4 {
                assert_eq!(dest[row * 8 + col], src[row * 6 + col]);
            }
            assert_eq!(dest[row * 8 + 4], 200);
        }
    }

    fn clear_box() -> [i32; 4] {
        [i32::MIN, i32::MAX, i32::MAX, i32::MIN]
    }

    fn index(x: usize, y: usize) -> usize {
        y * SCREENWIDTH + x
    }

    fn patterned_screen(seed: u8) -> Vec<u8> {
        (0..SCREEN_PIXELS)
            .map(|index| seed.wrapping_add((index % 251) as u8))
            .collect()
    }

    fn patch_ptr(patch: &mut [u8]) -> *mut c_void {
        patch.as_mut_ptr().cast()
    }

    fn two_column_patch() -> Vec<u8> {
        let col0 = [1, 2, 0, 11, 12, 0, 0xff];
        let col1 = [0, 1, 0, 21, 0, 0xff];
        let col0_offset = 8 + 2 * 4;
        let col1_offset = col0_offset + col0.len();
        let mut patch = Vec::new();
        push_i16(&mut patch, 2);
        push_i16(&mut patch, 4);
        push_i16(&mut patch, 1);
        push_i16(&mut patch, 2);
        push_i32(&mut patch, col0_offset as i32);
        push_i32(&mut patch, col1_offset as i32);
        patch.extend_from_slice(&col0);
        patch.extend_from_slice(&col1);
        patch
    }

    fn push_i16(bytes: &mut Vec<u8>, value: i16) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_i32(bytes: &mut Vec<u8>, value: i32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}
