//! Behavioral port of Chocolate Doom's `m_cheat` sequence checker.

use std::os::raw::{c_char, c_int};
use std::ptr;

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

impl CheatSeq {
    pub fn new(sequence: &[u8], parameter_chars: c_int) -> Self {
        let mut cheat = Self {
            sequence: [0; MAX_CHEAT_LEN],
            sequence_len: sequence.len(),
            parameter_chars,
            chars_read: 0,
            param_chars_read: 0,
            parameter_buf: [0; MAX_CHEAT_PARAMS],
        };

        for (dest, src) in cheat.sequence.iter_mut().zip(sequence.iter().copied()) {
            *dest = src as c_char;
        }

        cheat
    }
}

fn sequence_strlen(sequence: &[c_char; MAX_CHEAT_LEN]) -> usize {
    sequence
        .iter()
        .position(|&ch| ch == 0)
        .unwrap_or(MAX_CHEAT_LEN)
}

pub fn check_cheat(cheat: &mut CheatSeq, key: c_char) -> bool {
    let sequence_len = sequence_strlen(&cheat.sequence);

    // Vanilla behavior: parameterized cheats with a shorter live string than
    // their declared length never advance.
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
        let param_index = cheat.param_chars_read as usize;

        if param_index < MAX_CHEAT_PARAMS {
            cheat.parameter_buf[param_index] = key;
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

pub fn get_param(cheat: &CheatSeq, buffer: *mut c_char) {
    if buffer.is_null() || cheat.parameter_chars <= 0 {
        return;
    }

    let parameter_chars = (cheat.parameter_chars as usize).min(MAX_CHEAT_PARAMS);

    unsafe {
        ptr::copy_nonoverlapping(cheat.parameter_buf.as_ptr(), buffer, parameter_chars);
    }
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_cht_check_cheat(cheat: *mut CheatSeq, key: c_char) -> c_int {
    if cheat.is_null() {
        return 0;
    }

    check_cheat(&mut *cheat, key) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_cht_get_param(cheat: *const CheatSeq, buffer: *mut c_char) {
    if cheat.is_null() {
        return;
    }

    get_param(&*cheat, buffer);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(ch: u8) -> c_char {
        ch as c_char
    }

    #[test]
    fn exact_sequence_matches_and_resets() {
        let mut cheat = CheatSeq::new(b"idkfa", 0);

        for ch in b"idkf" {
            assert!(!check_cheat(&mut cheat, key(*ch)));
        }

        assert!(check_cheat(&mut cheat, key(b'a')));
        assert_eq!(cheat.chars_read, 0);
        assert_eq!(cheat.param_chars_read, 0);
    }

    #[test]
    fn wrong_key_resets_sequence_progress() {
        let mut cheat = CheatSeq::new(b"iddqd", 0);

        assert!(!check_cheat(&mut cheat, key(b'i')));
        assert!(!check_cheat(&mut cheat, key(b'x')));
        assert_eq!(cheat.chars_read, 0);

        for ch in b"iddq" {
            assert!(!check_cheat(&mut cheat, key(*ch)));
        }

        assert!(check_cheat(&mut cheat, key(b'd')));
    }

    #[test]
    fn parameterized_sequence_collects_parameter_bytes() {
        let mut cheat = CheatSeq::new(b"idclev", 2);

        for ch in b"idclev4" {
            assert!(!check_cheat(&mut cheat, key(*ch)));
        }

        assert!(check_cheat(&mut cheat, key(b'2')));
        assert_eq!(cheat.parameter_buf[0], key(b'4'));
        assert_eq!(cheat.parameter_buf[1], key(b'2'));
    }
}
