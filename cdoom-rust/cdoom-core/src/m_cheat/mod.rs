use std::os::raw::{c_char, c_int};
use std::ptr;

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

impl CheatSeq {
    pub fn new(sequence: &[u8], sequence_len: usize, parameter_chars: c_int) -> Self {
        let mut result = Self {
            sequence: [0; MAX_CHEAT_LEN],
            sequence_len,
            parameter_chars,
            chars_read: 0,
            param_chars_read: 0,
            parameter_buf: [0; MAX_CHEAT_PARAMS],
        };

        for (slot, value) in result.sequence.iter_mut().zip(sequence.iter().copied()) {
            *slot = value as c_char;
        }

        result
    }

    fn runtime_sequence_len(&self) -> usize {
        self.sequence
            .iter()
            .position(|&ch| ch == 0)
            .unwrap_or(self.sequence.len())
    }
}

pub fn check_cheat(cheat: &mut CheatSeq, key: c_char) -> bool {
    let runtime_len = cheat.runtime_sequence_len();

    // Vanilla keeps the original literal length separately; shortened
    // parameterized cheats must keep failing.
    if cheat.parameter_chars > 0 && runtime_len < cheat.sequence_len {
        return false;
    }

    if cheat.chars_read < runtime_len {
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

    if cheat.chars_read >= runtime_len && cheat.param_chars_read >= cheat.parameter_chars {
        cheat.chars_read = 0;
        cheat.param_chars_read = 0;
        return true;
    }

    false
}

pub unsafe fn get_param(cheat: &CheatSeq, buffer: *mut c_char) {
    if cheat.parameter_chars <= 0 {
        return;
    }

    ptr::copy_nonoverlapping(
        cheat.parameter_buf.as_ptr(),
        buffer,
        cheat.parameter_chars as usize,
    );
}
