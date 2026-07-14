//! AES-128 counter-mode PRNG used for secure demo determinism.

use std::ffi::c_void;
use std::sync::Mutex;

const AES_BLOCK_SIZE: usize = 16;
const AES_128_ROUNDS: usize = 10;
const AES_128_EXPANDED_KEY_SIZE: usize = 176;

const SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

const RCON: [u8; AES_128_ROUNDS] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];

#[derive(Clone, Copy)]
struct PrngState {
    enabled: bool,
    round_keys: [u8; AES_128_EXPANDED_KEY_SIZE],
    input_counter: u32,
    values: [u32; 4],
    value_index: usize,
}

impl PrngState {
    const fn new() -> Self {
        Self {
            enabled: false,
            round_keys: [0; AES_128_EXPANDED_KEY_SIZE],
            input_counter: 0,
            values: [0; 4],
            value_index: 0,
        }
    }

    fn start(&mut self, seed: &[u8; AES_BLOCK_SIZE]) {
        self.round_keys = expand_key(seed);
        self.value_index = 4;
        self.input_counter = 0;
        self.enabled = true;
    }

    fn stop(&mut self) {
        self.enabled = false;
    }

    fn random(&mut self) -> u32 {
        if !self.enabled {
            return 0;
        }

        if self.value_index >= 4 {
            self.generate();
        }

        let result = self.values[self.value_index];
        self.value_index += 1;
        result
    }

    fn generate(&mut self) {
        let mut input = [0_u8; AES_BLOCK_SIZE];
        for word in input.chunks_exact_mut(4) {
            word.copy_from_slice(&self.input_counter.to_le_bytes());
            self.input_counter = self.input_counter.wrapping_add(1);
        }

        let output = encrypt_block(&self.round_keys, &input);
        for (i, word) in output.chunks_exact(4).enumerate() {
            self.values[i] = u32::from_le_bytes([word[0], word[1], word[2], word[3]]);
        }
        self.value_index = 0;
    }
}

static PRNG_STATE: Mutex<PrngState> = Mutex::new(PrngState::new());

fn expand_key(key: &[u8; AES_BLOCK_SIZE]) -> [u8; AES_128_EXPANDED_KEY_SIZE] {
    let mut round_keys = [0_u8; AES_128_EXPANDED_KEY_SIZE];
    round_keys[..AES_BLOCK_SIZE].copy_from_slice(key);

    let mut bytes_generated = AES_BLOCK_SIZE;
    let mut rcon_index = 0;
    let mut temp = [0_u8; 4];

    while bytes_generated < AES_128_EXPANDED_KEY_SIZE {
        temp.copy_from_slice(&round_keys[bytes_generated - 4..bytes_generated]);

        if bytes_generated % AES_BLOCK_SIZE == 0 {
            temp.rotate_left(1);
            for byte in &mut temp {
                *byte = SBOX[*byte as usize];
            }
            temp[0] ^= RCON[rcon_index];
            rcon_index += 1;
        }

        for byte in temp {
            round_keys[bytes_generated] = round_keys[bytes_generated - AES_BLOCK_SIZE] ^ byte;
            bytes_generated += 1;
        }
    }

    round_keys
}

fn add_round_key(state: &mut [u8; AES_BLOCK_SIZE], round_key: &[u8]) {
    for (state_byte, key_byte) in state.iter_mut().zip(round_key) {
        *state_byte ^= *key_byte;
    }
}

fn sub_bytes(state: &mut [u8; AES_BLOCK_SIZE]) {
    for byte in state {
        *byte = SBOX[*byte as usize];
    }
}

fn shift_rows(state: &mut [u8; AES_BLOCK_SIZE]) {
    let original = *state;

    state[0] = original[0];
    state[4] = original[4];
    state[8] = original[8];
    state[12] = original[12];

    state[1] = original[5];
    state[5] = original[9];
    state[9] = original[13];
    state[13] = original[1];

    state[2] = original[10];
    state[6] = original[14];
    state[10] = original[2];
    state[14] = original[6];

    state[3] = original[15];
    state[7] = original[3];
    state[11] = original[7];
    state[15] = original[11];
}

fn xtime(value: u8) -> u8 {
    if value & 0x80 == 0 {
        value << 1
    } else {
        (value << 1) ^ 0x1b
    }
}

fn mix_columns(state: &mut [u8; AES_BLOCK_SIZE]) {
    for column in state.chunks_exact_mut(4) {
        let a0 = column[0];
        let a1 = column[1];
        let a2 = column[2];
        let a3 = column[3];
        let t = a0 ^ a1 ^ a2 ^ a3;

        column[0] ^= t ^ xtime(a0 ^ a1);
        column[1] ^= t ^ xtime(a1 ^ a2);
        column[2] ^= t ^ xtime(a2 ^ a3);
        column[3] ^= t ^ xtime(a3 ^ a0);
    }
}

pub fn encrypt_block(
    round_keys: &[u8; AES_128_EXPANDED_KEY_SIZE],
    input: &[u8; AES_BLOCK_SIZE],
) -> [u8; AES_BLOCK_SIZE] {
    let mut state = *input;
    add_round_key(&mut state, &round_keys[..AES_BLOCK_SIZE]);

    for round in 1..AES_128_ROUNDS {
        sub_bytes(&mut state);
        shift_rows(&mut state);
        mix_columns(&mut state);
        let start = round * AES_BLOCK_SIZE;
        add_round_key(&mut state, &round_keys[start..start + AES_BLOCK_SIZE]);
    }

    sub_bytes(&mut state);
    shift_rows(&mut state);
    let start = AES_128_ROUNDS * AES_BLOCK_SIZE;
    add_round_key(&mut state, &round_keys[start..start + AES_BLOCK_SIZE]);

    state
}

pub fn random_sequence(seed: [u8; AES_BLOCK_SIZE], count: usize) -> Vec<u32> {
    let mut state = PrngState::new();
    state.start(&seed);
    (0..count).map(|_| state.random()).collect()
}

/// # Safety
///
/// `seed` must point to a valid 16-byte PRNG seed.
pub unsafe fn start(seed: *const c_void) {
    if seed.is_null() {
        return;
    }

    let mut key = [0_u8; AES_BLOCK_SIZE];
    key.copy_from_slice(std::slice::from_raw_parts(seed.cast(), AES_BLOCK_SIZE));
    PRNG_STATE.lock().expect("PRNG mutex poisoned").start(&key);
}

pub fn stop() {
    PRNG_STATE.lock().expect("PRNG mutex poisoned").stop();
}

pub fn random() -> u32 {
    PRNG_STATE.lock().expect("PRNG mutex poisoned").random()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_128_fips_known_answer_vector_matches() {
        let key = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        let plaintext = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ];
        let expected = [
            0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4,
            0xc5, 0x5a,
        ];

        assert_eq!(encrypt_block(&expand_key(&key), &plaintext), expected);
    }

    #[test]
    fn prng_counter_mode_known_answer_vectors_match() {
        assert_eq!(
            random_sequence([0; AES_BLOCK_SIZE], 4),
            [0xe53e_64e8, 0x0cf4_962e, 0x3fe6_1c56, 0x0edc_8533]
        );

        let seed = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        assert_eq!(
            random_sequence(seed, 4),
            [0x8262_5d61, 0xbe3b_4d26, 0xe827_481e, 0x39b2_3067]
        );
    }

    #[test]
    fn stopped_prng_returns_zero() {
        stop();
        assert_eq!(random(), 0);
    }
}
