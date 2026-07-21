//! REJECT lump padding compatibility helpers.

const REJECT_PAD_WORDS: usize = 4;
const REJECT_PAD_LEN: usize = REJECT_PAD_WORDS * 4;

fn reject_pad_words(total_lines: i32) -> [u32; REJECT_PAD_WORDS] {
    let size = total_lines.wrapping_mul(4).wrapping_add(3) & !3;

    [size.wrapping_add(24) as u32, 0, 50, 0x1d4a11]
}

/// Pad an undersized REJECT lump exactly as Chocolate Doom's C helper does.
pub fn pad_reject_array(array: &mut [u8], total_lines: i32, pad_with_ff: bool) {
    let rejectpad = reject_pad_words(total_lines);
    let prefix_len = array.len().min(REJECT_PAD_LEN);

    for (i, dest) in array.iter_mut().take(prefix_len).enumerate() {
        let byte_num = i % 4;
        *dest = ((rejectpad[i / 4] >> (byte_num * 8)) & 0xff) as u8;
    }

    if array.len() > REJECT_PAD_LEN {
        let padvalue = if pad_with_ff { 0xff } else { 0x00 };
        array[REJECT_PAD_LEN..].fill(padvalue);
    }
}
