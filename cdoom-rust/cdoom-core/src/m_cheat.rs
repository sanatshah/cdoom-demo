//! Cheat sequence checker migrated from `m_cheat.c`.

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

fn strlen_fixed(buf: &[c_char]) -> usize {
    buf.iter().position(|&ch| ch == 0).unwrap_or(buf.len())
}

pub fn check_cheat(cheat: &mut CheatSeq, key: c_char) -> c_int {
    let sequence_len = strlen_fixed(&cheat.sequence);

    if cheat.parameter_chars > 0 && sequence_len < cheat.sequence_len {
        return 0;
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
        cheat.parameter_buf[index] = key;
        cheat.param_chars_read += 1;
    }

    if cheat.chars_read >= sequence_len && cheat.param_chars_read >= cheat.parameter_chars {
        cheat.chars_read = 0;
        cheat.param_chars_read = 0;
        return 1;
    }

    0
}

pub fn get_param(cheat: &CheatSeq, buffer: *mut c_char) {
    let count = cheat.parameter_chars as usize;

    // SAFETY: The C API requires the caller to pass a buffer large enough for
    // `parameter_chars`, matching `memcpy(buffer, parameter_buf, count)`.
    unsafe {
        std::ptr::copy_nonoverlapping(cheat.parameter_buf.as_ptr(), buffer, count);
    }
}
