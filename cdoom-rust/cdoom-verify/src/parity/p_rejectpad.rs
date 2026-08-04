//! REJECT lump padding parity against chocolate-doom/src/p_rejectpad.c.

#[test]
fn truncates_header_for_short_buffers() {
    let mut bytes = [0xaa; 5];

    cdoom_core::p_rejectpad::pad_reject_array(&mut bytes, 3, false);

    assert_eq!(bytes, [36, 0, 0, 0, 0]);
}

#[test]
fn matches_vanilla_header_words() {
    let mut bytes = [0xaa; 16];

    cdoom_core::p_rejectpad::pad_reject_array(&mut bytes, 3, false);

    assert_eq!(
        bytes,
        [
            36, 0, 0, 0, // Size
            0, 0, 0, 0, // z_zone header word
            50, 0, 0, 0, // PU_LEVEL
            0x11, 0x4a, 0x1d, 0x00, // DOOM_CONST_ZONEID
        ]
    );
}

#[test]
fn fills_long_buffers_with_zero_or_ff() {
    let mut zero_padded = [0xaa; 20];
    let mut ff_padded = [0xaa; 20];

    cdoom_core::p_rejectpad::pad_reject_array(&mut zero_padded, 3, false);
    cdoom_core::p_rejectpad::pad_reject_array(&mut ff_padded, 3, true);

    assert_eq!(&zero_padded[16..], &[0, 0, 0, 0]);
    assert_eq!(&ff_padded[16..], &[0xff, 0xff, 0xff, 0xff]);
}

#[test]
fn preserves_c_negative_totallines_behavior() {
    let mut bytes = [0xaa; 4];

    cdoom_core::p_rejectpad::pad_reject_array(&mut bytes, -1, false);

    assert_eq!(bytes, [20, 0, 0, 0]);
}

#[test]
fn ffi_matches_safe_api() {
    let mut safe_bytes = [0xaa; 20];
    let mut ffi_bytes = [0xaa; 20];

    cdoom_core::p_rejectpad::pad_reject_array(&mut safe_bytes, 31, true);
    unsafe {
        cdoom_core::cdoom_rust_pad_reject_array(
            ffi_bytes.as_mut_ptr(),
            ffi_bytes.len() as u32,
            31,
            1,
        );
    }

    assert_eq!(ffi_bytes, safe_bytes);
}
