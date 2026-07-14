//! SHA-1 implementation kept layout-compatible with `sha1_context_t`.

use std::ffi::{c_char, c_void};
use std::ptr;

pub const DIGEST_LEN: usize = 20;

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sha1Context {
    pub h0: u32,
    pub h1: u32,
    pub h2: u32,
    pub h3: u32,
    pub h4: u32,
    pub nblocks: u32,
    pub buf: [u8; 64],
    pub count: i32,
}

impl Default for Sha1Context {
    fn default() -> Self {
        let mut context = Self {
            h0: 0,
            h1: 0,
            h2: 0,
            h3: 0,
            h4: 0,
            nblocks: 0,
            buf: [0; 64],
            count: 0,
        };
        init_context(&mut context);
        context
    }
}

fn context_from_void<'a>(context: *mut c_void) -> Option<&'a mut Sha1Context> {
    // SAFETY: Callers pass a pointer to the C `sha1_context_t`, which has the
    // same field order and integer widths as `Sha1Context`.
    unsafe { context.cast::<Sha1Context>().as_mut() }
}

pub fn init_context(context: &mut Sha1Context) {
    context.h0 = 0x6745_2301;
    context.h1 = 0xefcd_ab89;
    context.h2 = 0x98ba_dcfe;
    context.h3 = 0x1032_5476;
    context.h4 = 0xc3d2_e1f0;
    context.nblocks = 0;
    context.count = 0;
}

fn transform(context: &mut Sha1Context, data: &[u8; 64]) {
    let mut a = context.h0;
    let mut b = context.h1;
    let mut c = context.h2;
    let mut d = context.h3;
    let mut e = context.h4;

    let mut words = [0_u32; 16];
    for (i, chunk) in data.chunks_exact(4).enumerate() {
        words[i] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }

    for i in 0..80 {
        let (f, k, m) = match i {
            0..=15 => ((b & c) | ((!b) & d), 0x5a82_7999, words[i]),
            16..=19 => {
                let tm = words[i & 0x0f]
                    ^ words[(i - 14) & 0x0f]
                    ^ words[(i - 8) & 0x0f]
                    ^ words[(i - 3) & 0x0f];
                words[i & 0x0f] = tm.rotate_left(1);
                ((b & c) | ((!b) & d), 0x5a82_7999, words[i & 0x0f])
            }
            20..=39 => {
                let tm = words[i & 0x0f]
                    ^ words[(i - 14) & 0x0f]
                    ^ words[(i - 8) & 0x0f]
                    ^ words[(i - 3) & 0x0f];
                words[i & 0x0f] = tm.rotate_left(1);
                (b ^ c ^ d, 0x6ed9_eba1, words[i & 0x0f])
            }
            40..=59 => {
                let tm = words[i & 0x0f]
                    ^ words[(i - 14) & 0x0f]
                    ^ words[(i - 8) & 0x0f]
                    ^ words[(i - 3) & 0x0f];
                words[i & 0x0f] = tm.rotate_left(1);
                ((b & c) | (d & (b | c)), 0x8f1b_bcdc, words[i & 0x0f])
            }
            _ => {
                let tm = words[i & 0x0f]
                    ^ words[(i - 14) & 0x0f]
                    ^ words[(i - 8) & 0x0f]
                    ^ words[(i - 3) & 0x0f];
                words[i & 0x0f] = tm.rotate_left(1);
                (b ^ c ^ d, 0xca62_c1d6, words[i & 0x0f])
            }
        };

        e = e
            .wrapping_add(a.rotate_left(5))
            .wrapping_add(f)
            .wrapping_add(k)
            .wrapping_add(m);
        b = b.rotate_left(30);

        let old_e = e;
        e = d;
        d = c;
        c = b;
        b = a;
        a = old_e;
    }

    context.h0 = context.h0.wrapping_add(a);
    context.h1 = context.h1.wrapping_add(b);
    context.h2 = context.h2.wrapping_add(c);
    context.h3 = context.h3.wrapping_add(d);
    context.h4 = context.h4.wrapping_add(e);
}

pub fn update_context(context: &mut Sha1Context, mut inbuf: Option<&[u8]>) {
    if context.count == 64 {
        let block = context.buf;
        transform(context, &block);
        context.count = 0;
        context.nblocks = context.nblocks.wrapping_add(1);
    }

    let Some(mut input) = inbuf.take() else {
        return;
    };

    if context.count != 0 {
        while !input.is_empty() && context.count < 64 {
            context.buf[context.count as usize] = input[0];
            context.count += 1;
            input = &input[1..];
        }
        update_context(context, None);
        if input.is_empty() {
            return;
        }
    }

    while input.len() >= 64 {
        let mut block = [0_u8; 64];
        block.copy_from_slice(&input[..64]);
        transform(context, &block);
        context.count = 0;
        context.nblocks = context.nblocks.wrapping_add(1);
        input = &input[64..];
    }

    for &byte in input {
        if context.count < 64 {
            context.buf[context.count as usize] = byte;
            context.count += 1;
        }
    }
}

pub fn final_context(context: &mut Sha1Context) -> [u8; DIGEST_LEN] {
    update_context(context, None);

    let mut lsb = context.nblocks << 6;
    let mut msb = context.nblocks >> 26;
    let t = lsb;
    lsb = lsb.wrapping_add(context.count as u32);
    if lsb < t {
        msb = msb.wrapping_add(1);
    }

    let t = lsb;
    lsb <<= 3;
    msb <<= 3;
    msb |= t >> 29;

    if context.count < 56 {
        context.buf[context.count as usize] = 0x80;
        context.count += 1;
        while context.count < 56 {
            context.buf[context.count as usize] = 0;
            context.count += 1;
        }
    } else {
        context.buf[context.count as usize] = 0x80;
        context.count += 1;
        while context.count < 64 {
            context.buf[context.count as usize] = 0;
            context.count += 1;
        }
        update_context(context, None);
        context.buf[..56].fill(0);
    }

    context.buf[56..60].copy_from_slice(&msb.to_be_bytes());
    context.buf[60..64].copy_from_slice(&lsb.to_be_bytes());
    let block = context.buf;
    transform(context, &block);

    let mut digest = [0_u8; DIGEST_LEN];
    digest[0..4].copy_from_slice(&context.h0.to_be_bytes());
    digest[4..8].copy_from_slice(&context.h1.to_be_bytes());
    digest[8..12].copy_from_slice(&context.h2.to_be_bytes());
    digest[12..16].copy_from_slice(&context.h3.to_be_bytes());
    digest[16..20].copy_from_slice(&context.h4.to_be_bytes());
    context.buf[..DIGEST_LEN].copy_from_slice(&digest);

    digest
}

pub fn digest_bytes(input: &[u8]) -> [u8; DIGEST_LEN] {
    let mut context = Sha1Context::default();
    update_context(&mut context, Some(input));
    final_context(&mut context)
}

/// # Safety
///
/// `context` must point to a valid C `sha1_context_t`.
pub unsafe fn init(context: *mut c_void) {
    if let Some(context) = context_from_void(context) {
        init_context(context);
    }
}

/// # Safety
///
/// `context` must point to a valid C `sha1_context_t`. If `buf` is non-null it
/// must be valid for `len` bytes.
pub unsafe fn update(context: *mut c_void, buf: *const c_void, len: usize) {
    let Some(context) = context_from_void(context) else {
        return;
    };

    if buf.is_null() {
        update_context(context, None);
    } else {
        update_context(context, Some(std::slice::from_raw_parts(buf.cast(), len)));
    }
}

/// # Safety
///
/// `digest` must be valid for 20 bytes and `context` must point to a valid C
/// `sha1_context_t`.
pub unsafe fn final_digest(digest: *mut c_void, context: *mut c_void) {
    let Some(context) = context_from_void(context) else {
        return;
    };
    let result = final_context(context);
    if !digest.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr(), digest.cast(), result.len());
    }
}

/// # Safety
///
/// `context` must point to a valid C `sha1_context_t`.
pub unsafe fn update_int32(context: *mut c_void, value: u32) {
    update(context, value.to_be_bytes().as_ptr().cast(), 4);
}

/// # Safety
///
/// `context` must point to a valid C `sha1_context_t` and `str` must be a
/// NUL-terminated string.
pub unsafe fn update_string(context: *mut c_void, str: *const c_char) {
    if str.is_null() {
        return;
    }

    let mut len = 0;
    while *str.add(len) != 0 {
        len += 1;
    }
    update(context, str.cast(), len + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[test]
    fn known_sha1_vectors_match_gnupg_source_comments() {
        assert_eq!(
            hex(&digest_bytes(b"abc")),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(
            hex(&digest_bytes(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
        );
    }

    #[test]
    fn update_int32_and_string_include_c_wire_format() {
        let mut context = Sha1Context::default();
        update_context(&mut context, Some(&0x0102_0304_u32.to_be_bytes()));
        update_context(&mut context, Some(b"demo\0"));
        let expected = final_context(&mut context);

        let mut via_helpers = Sha1Context::default();
        unsafe {
            update_int32((&mut via_helpers as *mut Sha1Context).cast(), 0x0102_0304);
            update_string(
                (&mut via_helpers as *mut Sha1Context).cast(),
                c"demo".as_ptr(),
            );
        }
        assert_eq!(final_context(&mut via_helpers), expected);
    }
}
