use std::os::raw::{c_char, c_int};

pub const MAX_CHEAT_LEN: usize = 25;
pub const MAX_CHEAT_PARAMS: usize = 5;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheatSeq {
    pub sequence: [c_char; MAX_CHEAT_LEN],
    pub sequence_len: usize,
    pub parameter_chars: c_int,
    pub chars_read: usize,
    pub param_chars_read: c_int,
    pub parameter_buf: [c_char; MAX_CHEAT_PARAMS],
}

fn sequence_strlen(cht: &CheatSeq) -> usize {
    cht.sequence
        .iter()
        .position(|&ch| ch == 0)
        .unwrap_or(MAX_CHEAT_LEN)
}

pub fn check_cheat(cht: &mut CheatSeq, key: c_char) -> c_int {
    let sequence_len = sequence_strlen(cht);

    if cht.parameter_chars > 0 && sequence_len < cht.sequence_len {
        return 0;
    }

    if cht.chars_read < sequence_len {
        if key == cht.sequence[cht.chars_read] {
            cht.chars_read += 1;
        } else {
            cht.chars_read = 0;
        }

        cht.param_chars_read = 0;
    } else if cht.param_chars_read < cht.parameter_chars {
        cht.parameter_buf[cht.param_chars_read as usize] = key;
        cht.param_chars_read += 1;
    }

    if cht.chars_read >= sequence_len && cht.param_chars_read >= cht.parameter_chars {
        cht.chars_read = 0;
        cht.param_chars_read = 0;

        return 1;
    }

    0
}

pub fn get_param(cht: &CheatSeq, buffer: &mut [c_char]) {
    let count = cht.parameter_chars as usize;
    buffer[..count].copy_from_slice(&cht.parameter_buf[..count]);
}
