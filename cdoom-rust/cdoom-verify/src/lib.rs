//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::path::Path;

/// Expected Chocolate Doom package version vendored in this repo.
pub const CHOCOLATE_DOOM_VERSION: &str = "3.1.1";

/// Returns `true` when a timedemo baseline can run (binary + IWAD present).
pub fn timedemo_baseline_available(root: &Path) -> bool {
    let binary = root.join("chocolate-doom/build/src/chocolate-doom");
    let wad = root.join("wads/freedoom1.wad");
    binary.is_file() && wad.is_file()
}

/// Reads the exported Rust version string from the C ABI.
pub fn rust_version_from_ffi() -> String {
    let ptr = cdoom_core::cdoom_rust_version();
    assert!(!ptr.is_null());
    // SAFETY: cdoom_rust_version returns a static NUL-terminated string.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cdoom_core::tables;
    use cdoom_core::types;

    const TABLES_C: &str = include_str!("../../../chocolate-doom/src/tables.c");

    fn c_table_values(decl: &str) -> Vec<i64> {
        let start = TABLES_C.find(decl).expect("table declaration exists");
        let brace = TABLES_C[start..].find('{').expect("table body starts") + start;
        let mut depth = 0;
        let mut end = None;

        for (idx, ch) in TABLES_C[brace..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(brace + idx);
                        break;
                    }
                }
                _ => {}
            }
        }

        let end = end.expect("table body ends");
        parse_numbers(&TABLES_C[brace + 1..end])
    }

    fn parse_numbers(input: &str) -> Vec<i64> {
        let mut numbers = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            let negative = ch == '-';
            if !negative && !ch.is_ascii_digit() {
                continue;
            }

            if negative && !chars.peek().is_some_and(char::is_ascii_digit) {
                continue;
            }

            let mut value = 0i64;
            let mut saw_digit = false;

            if !negative {
                value = ch.to_digit(10).expect("digit") as i64;
                saw_digit = true;
            }

            while let Some(next) = chars.peek() {
                if !next.is_ascii_digit() {
                    break;
                }
                saw_digit = true;
                value = value * 10 + next.to_digit(10).expect("digit") as i64;
                chars.next();
            }

            if saw_digit {
                numbers.push(if negative { -value } else { value });
            }
        }

        numbers
    }

    fn i32_le_bytes(values: &[i32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn u32_le_bytes(values: &[u32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    #[test]
    fn ffi_version_matches_crate() {
        assert_eq!(rust_version_from_ffi(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn init_succeeds() {
        assert_eq!(cdoom_core::cdoom_rust_init(), 0);
    }

    #[test]
    fn version_string_is_non_empty() {
        assert!(!cdoom_core::version_string().is_empty());
    }

    #[test]
    fn rust_table_byte_dumps_match_c_tables() {
        let c_finetangent: Vec<i32> = c_table_values("const fixed_t finetangent[4096]")
            .into_iter()
            .map(|value| value.try_into().expect("fixed_t"))
            .collect();
        let c_finesine: Vec<i32> = c_table_values("const fixed_t finesine[10240]")
            .into_iter()
            .map(|value| value.try_into().expect("fixed_t"))
            .collect();
        let c_tantoangle: Vec<u32> = c_table_values("const angle_t tantoangle[2049]")
            .into_iter()
            .map(|value| value.try_into().expect("angle_t"))
            .collect();
        let c_gammatable: Vec<u8> = c_table_values("const byte gammatable[5][256]")
            .into_iter()
            .map(|value| value.try_into().expect("byte"))
            .collect();

        assert_eq!(c_finetangent.len(), tables::FINETANGENT_LEN);
        assert_eq!(c_finesine.len(), tables::FINESINE_LEN);
        assert_eq!(c_tantoangle.len(), tables::TANTOANGLE_LEN);
        assert_eq!(
            c_gammatable.len(),
            tables::GAMMA_LEVELS * tables::GAMMA_COLS
        );

        assert_eq!(
            i32_le_bytes(&tables::FINETANGENT),
            i32_le_bytes(&c_finetangent),
            "finetangent byte dump differs from C"
        );
        assert_eq!(
            i32_le_bytes(&tables::FINESINE),
            i32_le_bytes(&c_finesine),
            "finesine byte dump differs from C"
        );
        assert_eq!(
            u32_le_bytes(&tables::TANTOANGLE),
            u32_le_bytes(&c_tantoangle),
            "tantoangle byte dump differs from C"
        );

        let rust_gamma: Vec<u8> = tables::GAMMATABLE.iter().flatten().copied().collect();
        assert_eq!(
            rust_gamma, c_gammatable,
            "gammatable byte dump differs from C"
        );
    }

    #[test]
    fn rust_table_ffi_exports_have_expected_shapes() {
        assert!(!cdoom_core::cdoom_rust_tables_finetangent().is_null());
        assert!(!cdoom_core::cdoom_rust_tables_finesine().is_null());
        assert!(!cdoom_core::cdoom_rust_tables_finecosine().is_null());
        assert!(!cdoom_core::cdoom_rust_tables_tantoangle().is_null());
        assert!(!cdoom_core::cdoom_rust_tables_gammatable().is_null());

        assert_eq!(
            cdoom_core::cdoom_rust_tables_finetangent_len(),
            tables::FINETANGENT_LEN
        );
        assert_eq!(
            cdoom_core::cdoom_rust_tables_finesine_len(),
            tables::FINESINE_LEN
        );
        assert_eq!(
            cdoom_core::cdoom_rust_tables_finecosine_len(),
            tables::FINESINE_LEN - tables::FINECOSINE_OFFSET
        );
        assert_eq!(
            cdoom_core::cdoom_rust_tables_tantoangle_len(),
            tables::TANTOANGLE_LEN
        );
        assert_eq!(
            cdoom_core::cdoom_rust_tables_gammatable_len(),
            tables::GAMMA_LEVELS * tables::GAMMA_COLS
        );
    }

    #[test]
    fn slope_div_matches_c_edge_cases() {
        assert_eq!(tables::slope_div(0, 0), tables::SLOPERANGE as i32);
        assert_eq!(tables::slope_div(123, 511), tables::SLOPERANGE as i32);
        assert_eq!(tables::slope_div(0, 512), 0);
        assert_eq!(tables::slope_div(1024, 512), tables::SLOPERANGE as i32);
        assert_eq!(tables::slope_div(0x80000000, 512), 0);
        assert_eq!(cdoom_core::cdoom_rust_tables_slope_div(64, 1024), 128);
    }

    #[test]
    fn type_foundations_match_c_headers() {
        assert_eq!(types::FALSE, 0);
        assert_eq!(types::TRUE, 1);
        assert_eq!(types::FRACBITS, 16);
        assert_eq!(types::FRACUNIT, 65536);
        assert_eq!(types::KEY_RIGHTARROW, 0xae);
        assert_eq!(types::KEY_UPARROW, 0xad);
        assert_eq!(types::KEY_RCTRL, 0x80 + 0x1d);
        assert_eq!(types::SCANCODE_TO_KEYS.len(), 104);
        assert_eq!(types::SCANCODE_TO_KEYS[82], types::KEY_UPARROW);
        assert_eq!(types::short(0xff80), -128);
        assert_eq!(types::long(0xffff0000), -65536);
    }
}
