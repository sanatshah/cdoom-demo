//! REJECT lump padding used to emulate Vanilla Doom overflow behavior.

const REJECT_PAD_BYTES: usize = 16;

fn reject_pad_size(totallines: i32) -> u32 {
    let rounded_line_bytes = totallines.wrapping_mul(4).wrapping_add(3) & !3;

    rounded_line_bytes.wrapping_add(24) as u32
}

/// Pad a too-short REJECT lump with the same byte pattern as Chocolate Doom C.
pub fn pad_reject_array(array: &mut [u8], totallines: i32, pad_with_ff: bool) {
    let rejectpad = [reject_pad_size(totallines), 0, 50, 0x1d4a11];

    for (i, dest) in array.iter_mut().take(REJECT_PAD_BYTES).enumerate() {
        let byte_num = i % 4;
        *dest = (rejectpad[i / 4] >> (byte_num * 8)) as u8;
    }

    if array.len() > REJECT_PAD_BYTES {
        let padvalue = if pad_with_ff { 0xff } else { 0x00 };
        array[REJECT_PAD_BYTES..].fill(padvalue);
    }
}
