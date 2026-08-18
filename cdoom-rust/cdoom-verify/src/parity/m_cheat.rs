//! Cheat-sequence parity against chocolate-doom/src/m_cheat.c (Phase 3).

use std::os::raw::c_char;

use cdoom_core::m_cheat;

fn key(ch: u8) -> c_char {
    ch as c_char
}

#[test]
fn cheat_sequence_matches_and_resets_like_c() {
    // m_cheat.c:68-73 — full match returns true and clears counters.
    let mut cheat = m_cheat::CheatSeq::new(b"iddqd", 0);

    for ch in b"iddq" {
        assert_eq!(
            unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(*ch)) },
            0
        );
    }

    assert_eq!(
        unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(b'd')) },
        1
    );
    assert_eq!(cheat.chars_read, 0);
    assert_eq!(cheat.param_chars_read, 0);
}

#[test]
fn cheat_wrong_key_resets_without_rechecking_as_start() {
    // m_cheat.c:51-54 — mismatch sets chars_read = 0 and does not re-test key
    // against sequence[0]. Typing 'i' then another 'i' must leave chars_read at 0.
    let mut cheat = m_cheat::CheatSeq::new(b"idkfa", 0);

    assert_eq!(
        unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(b'i')) },
        0
    );
    assert_eq!(cheat.chars_read, 1);

    assert_eq!(
        unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(b'i')) },
        0
    );
    assert_eq!(cheat.chars_read, 0);

    assert_eq!(
        unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(b'x')) },
        0
    );
    assert_eq!(cheat.chars_read, 0);
}

#[test]
fn cheat_parameter_bytes_are_copied_after_match() {
    // m_cheat.c cht_GetParam — memcpy of parameter_buf after param collection.
    let mut cheat = m_cheat::CheatSeq::new(b"idclev", 2);
    let mut parameter = [0 as c_char; 2];

    for ch in b"idclev4" {
        assert_eq!(
            unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(*ch)) },
            0
        );
    }

    assert_eq!(
        unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(b'2')) },
        1
    );

    unsafe {
        m_cheat::cdoom_rust_cht_get_param(&cheat, parameter.as_mut_ptr());
    }

    assert_eq!(parameter, [key(b'4'), key(b'2')]);
}

#[test]
fn cheat_short_parameterized_sequence_does_not_advance() {
    // m_cheat.c:42-43 — parameter_chars > 0 && strlen(sequence) < sequence_len → false.
    let mut cheat = m_cheat::CheatSeq::new(b"id", 2);
    cheat.sequence_len = 5;

    for ch in b"id12" {
        assert_eq!(
            unsafe { m_cheat::cdoom_rust_cht_check_cheat(&mut cheat, key(*ch)) },
            0
        );
    }

    assert_eq!(cheat.chars_read, 0);
    assert_eq!(cheat.param_chars_read, 0);
}
