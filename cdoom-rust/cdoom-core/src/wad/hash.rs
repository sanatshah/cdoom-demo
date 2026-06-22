//! Lump name hash (djb2, max 8 chars, uppercase) — matches `W_LumpNameHash` in C.

/// Hash a WAD lump name using Chocolate Doom's djb2 variant.
pub fn lump_name_hash(name: &str) -> u32 {
    let mut result: u32 = 5381;
    for (i, ch) in name.bytes().enumerate() {
        if i >= 8 || ch == 0 {
            break;
        }
        let upper = ch.to_ascii_uppercase();
        result = ((result << 5) ^ result) ^ u32::from(upper);
    }
    result
}

/// Hash a fixed 8-byte lump name field (may contain NUL padding).
pub fn lump_name_hash_bytes(name: &[u8; 8]) -> u32 {
    let mut result: u32 = 5381;
    for (i, &ch) in name.iter().enumerate() {
        if i >= 8 || ch == 0 {
            break;
        }
        let upper = ch.to_ascii_uppercase();
        result = ((result << 5) ^ result) ^ u32::from(upper);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_name_is_seed_only() {
        assert_eq!(lump_name_hash(""), 5381);
    }

    #[test]
    fn truncates_at_eight_chars() {
        assert_eq!(lump_name_hash("12345678"), lump_name_hash("123456789"));
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(lump_name_hash("playpal"), lump_name_hash("PLAYPAL"));
    }
}
