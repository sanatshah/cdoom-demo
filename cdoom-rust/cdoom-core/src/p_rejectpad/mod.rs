//! REJECT lump padding logic shared by all game variants.

const REJECT_PAD_WORDS: usize = 4;
const REJECT_PAD_BYTES: usize = REJECT_PAD_WORDS * 4;

fn reject_pad_size_word(totallines: i32) -> u32 {
    let padded_line_bytes = totallines.wrapping_mul(4).wrapping_add(3) & !3;
    padded_line_bytes.wrapping_add(24) as u32
}

fn reject_pad_words(totallines: i32) -> [u32; REJECT_PAD_WORDS] {
    [reject_pad_size_word(totallines), 0, 50, 0x1d4a11]
}

/// Pads a short REJECT lump using Chocolate Doom's vanilla overflow emulation.
pub fn pad_reject_array(array: &mut [u8], totallines: i32, pad_with_ff: bool) {
    let rejectpad = reject_pad_words(totallines);

    for (dest, source) in array.iter_mut().zip(
        rejectpad
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .take(REJECT_PAD_BYTES),
    ) {
        *dest = source;
    }

    if array.len() > REJECT_PAD_BYTES {
        let padvalue = if pad_with_ff { 0xff } else { 0x00 };
        array[REJECT_PAD_BYTES..].fill(padvalue);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_c_style_size_word_for_positive_lines() {
        assert_eq!(reject_pad_size_word(0), 24);
        assert_eq!(reject_pad_size_word(1), 28);
        assert_eq!(reject_pad_size_word(31), 148);
    }

    #[test]
    fn computes_c_style_size_word_for_negative_lines() {
        assert_eq!(reject_pad_size_word(-1), 20);
        assert_eq!(reject_pad_size_word(-2), 16);
    }

    #[test]
    fn truncates_header_when_destination_is_short() {
        let mut bytes = [0xaa; 5];

        pad_reject_array(&mut bytes, 3, false);

        assert_eq!(bytes, [36, 0, 0, 0, 0]);
    }

    #[test]
    fn writes_header_words_little_endian() {
        let mut bytes = [0xaa; REJECT_PAD_BYTES];

        pad_reject_array(&mut bytes, 3, false);

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
    fn fills_extra_bytes_with_zero_by_default() {
        let mut bytes = [0xaa; REJECT_PAD_BYTES + 4];

        pad_reject_array(&mut bytes, 3, false);

        assert_eq!(&bytes[REJECT_PAD_BYTES..], &[0, 0, 0, 0]);
    }

    #[test]
    fn fills_extra_bytes_with_ff_when_requested() {
        let mut bytes = [0xaa; REJECT_PAD_BYTES + 4];

        pad_reject_array(&mut bytes, 3, true);

        assert_eq!(&bytes[REJECT_PAD_BYTES..], &[0xff, 0xff, 0xff, 0xff]);
    }
}
