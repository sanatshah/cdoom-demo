//! Cheat sequence checker ported from `m_cheat.c`.

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

unsafe extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

unsafe fn sequence_strlen(cht: *const CheatSeq) -> usize {
    // SAFETY: `cheatseq_t.sequence` is maintained as a NUL-terminated C string.
    unsafe { strlen((*cht).sequence.as_ptr()) }
}

/// Checks whether `key` advances or completes a cheat sequence.
///
/// # Safety
///
/// `cht` must point to a valid C `cheatseq_t` with Chocolate Doom layout.
pub unsafe fn check_cheat(cht: *mut CheatSeq, key: c_char) -> c_int {
    let sequence_len = unsafe { sequence_strlen(cht) };

    // If we make a short sequence on a cheat with parameters, this will not
    // work in vanilla Doom. Behave the same.
    if unsafe { (*cht).parameter_chars > 0 && sequence_len < (*cht).sequence_len } {
        return 0;
    }

    if unsafe { (*cht).chars_read < sequence_len } {
        let chars_read = unsafe { (*cht).chars_read };

        if key == unsafe { (*cht).sequence[chars_read] } {
            unsafe {
                (*cht).chars_read += 1;
            }
        } else {
            unsafe {
                (*cht).chars_read = 0;
            }
        }

        unsafe {
            (*cht).param_chars_read = 0;
        }
    } else if unsafe { (*cht).param_chars_read < (*cht).parameter_chars } {
        let param_chars_read = unsafe { (*cht).param_chars_read as usize };
        unsafe {
            (*cht).parameter_buf[param_chars_read] = key;
            (*cht).param_chars_read += 1;
        }
    }

    if unsafe {
        (*cht).chars_read >= sequence_len && (*cht).param_chars_read >= (*cht).parameter_chars
    } {
        unsafe {
            (*cht).chars_read = 0;
            (*cht).param_chars_read = 0;
        }

        1
    } else {
        0
    }
}

/// Copies the most recently read cheat parameter bytes into `buffer`.
///
/// # Safety
///
/// `cht` and `buffer` must be valid for `cht.parameter_chars` bytes.
pub unsafe fn get_param(cht: *const CheatSeq, buffer: *mut c_char) {
    unsafe {
        ptr::copy_nonoverlapping(
            (*cht).parameter_buf.as_ptr(),
            buffer,
            (*cht).parameter_chars as usize,
        );
    }
}
