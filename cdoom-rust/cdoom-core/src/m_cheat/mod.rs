use std::os::raw::{c_char, c_int};

pub const MAX_CHEAT_LEN: usize = 25;
pub const MAX_CHEAT_PARAMS: usize = 5;

#[repr(C)]
pub struct CheatSeq {
    pub sequence: [c_char; MAX_CHEAT_LEN],
    pub sequence_len: usize,
    pub parameter_chars: c_int,
    pub chars_read: usize,
    pub param_chars_read: c_int,
    pub parameter_buf: [c_char; MAX_CHEAT_PARAMS],
}

fn sequence_strlen(sequence: &[c_char; MAX_CHEAT_LEN]) -> usize {
    sequence
        .iter()
        .position(|&ch| ch == 0)
        .unwrap_or(MAX_CHEAT_LEN)
}

pub fn check_cheat(cheat: &mut CheatSeq, key: c_char) -> bool {
    let sequence_len = sequence_strlen(&cheat.sequence);

    // Vanilla refuses parameterized cheats whose runtime string is shorter
    // than the declared sequence length.
    if cheat.parameter_chars > 0 && sequence_len < cheat.sequence_len {
        return false;
    }

    if cheat.chars_read < sequence_len {
        if key == cheat.sequence[cheat.chars_read] {
            cheat.chars_read += 1;
        } else {
            cheat.chars_read = 0;
        }

        cheat.param_chars_read = 0;
    } else if cheat.param_chars_read < cheat.parameter_chars {
        let index = cheat.param_chars_read as usize;
        if index < MAX_CHEAT_PARAMS {
            cheat.parameter_buf[index] = key;
        }

        cheat.param_chars_read += 1;
    }

    if cheat.chars_read >= sequence_len && cheat.param_chars_read >= cheat.parameter_chars {
        cheat.chars_read = 0;
        cheat.param_chars_read = 0;
        return true;
    }

    false
}

pub fn get_param(cheat: &CheatSeq, buffer: &mut [c_char]) {
    let count = (cheat.parameter_chars as usize).min(buffer.len());
    buffer[..count].copy_from_slice(&cheat.parameter_buf[..count]);
}
