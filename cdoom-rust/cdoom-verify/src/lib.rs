//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
use std::path::Path;

#[cfg(test)]
const TABLES_C: &str = include_str!("../../../chocolate-doom/src/tables.c");

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
fn c_initializer_for(declaration: &str) -> &'static str {
    let declaration_start = TABLES_C.find(declaration).expect("C declaration present");
    let initializer_start = TABLES_C[declaration_start..]
        .find('{')
        .map(|offset| declaration_start + offset + 1)
        .expect("C initializer start present");

    let mut depth = 1;
    for (offset, byte) in TABLES_C[initializer_start..].bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &TABLES_C[initializer_start..initializer_start + offset];
                }
            }
            _ => {}
        }
    }

    panic!("C initializer is unterminated");
}

#[cfg(test)]
fn parse_c_values(declaration: &str) -> Vec<i64> {
    let initializer = c_initializer_for(declaration);
    let mut values = Vec::new();
    let mut token = String::new();

    for ch in initializer.chars() {
        if ch == '-' || ch.is_ascii_digit() {
            token.push(ch);
        } else if !token.is_empty() {
            values.push(token.parse().expect("C integer token parses"));
            token.clear();
        }
    }

    if !token.is_empty() {
        values.push(token.parse().expect("C integer token parses"));
    }

    values
}

#[cfg(test)]
fn i32_byte_dump(values: impl IntoIterator<Item = i32>) -> Vec<u8> {
    values
        .into_iter()
        .flat_map(i32::to_ne_bytes)
        .collect::<Vec<_>>()
}

#[cfg(test)]
fn u32_byte_dump(values: impl IntoIterator<Item = u32>) -> Vec<u8> {
    values
        .into_iter()
        .flat_map(u32::to_ne_bytes)
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cdoom_core::{tables, types};
    use std::mem::size_of;

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
    fn rust_tables_match_c_lut_byte_dumps() {
        let c_finetangent = parse_c_values("const fixed_t finetangent[4096]");
        let rust_finetangent = tables::finetangent().iter().copied();
        assert_eq!(c_finetangent.len(), tables::FINETANGENT_LEN);
        assert_eq!(
            i32_byte_dump(c_finetangent.into_iter().map(|value| value as i32)),
            i32_byte_dump(rust_finetangent)
        );

        let c_finesine = parse_c_values("const fixed_t finesine[10240]");
        let rust_finesine = tables::finesine().iter().copied();
        assert_eq!(c_finesine.len(), tables::FINESINE_LEN);
        assert_eq!(
            i32_byte_dump(c_finesine.into_iter().map(|value| value as i32)),
            i32_byte_dump(rust_finesine)
        );

        let c_tantoangle = parse_c_values("const angle_t tantoangle[2049]");
        let rust_tantoangle = tables::tantoangle().iter().copied();
        assert_eq!(c_tantoangle.len(), tables::TANTOANGLE_LEN);
        assert_eq!(
            u32_byte_dump(c_tantoangle.into_iter().map(|value| value as u32)),
            u32_byte_dump(rust_tantoangle)
        );

        let c_gammatable = parse_c_values("const byte gammatable[5][256]");
        assert_eq!(
            c_gammatable.len(),
            tables::GAMMA_LEVELS * tables::GAMMA_VALUES
        );
        assert_eq!(
            c_gammatable
                .into_iter()
                .map(|value| value as u8)
                .collect::<Vec<_>>(),
            tables::gammatable_flat().as_slice()
        );
    }

    #[test]
    fn rust_tables_match_c_pointer_relationships() {
        assert_eq!(
            tables::finecosine().as_ptr(),
            tables::finesine()[types::FINEANGLES / 4..].as_ptr()
        );
    }

    #[test]
    fn rust_slope_div_matches_c_formula() {
        for (num, den) in [
            (0, 0),
            (0, 511),
            (0, 512),
            (1, 512),
            (2048, 2048),
            (2049, 2048),
            (u32::MAX, 512),
            (u32::MAX, u32::MAX),
        ] {
            let expected = if den < 512 {
                types::SLOPERANGE
            } else {
                let ans = (num << 3) / (den >> 8);
                if ans <= types::SLOPERANGE as u32 {
                    ans as i32
                } else {
                    types::SLOPERANGE
                }
            };

            assert_eq!(tables::slope_div(num, den), expected);
        }
    }

    #[test]
    fn rust_type_foundations_match_c_values() {
        assert_eq!(size_of::<types::Boolean>(), size_of::<i32>());
        assert_eq!(size_of::<types::Byte>(), size_of::<u8>());
        assert_eq!(size_of::<types::Fixed>(), size_of::<i32>());
        assert_eq!(size_of::<types::Angle>(), size_of::<u32>());

        assert_eq!(types::FALSE, 0);
        assert_eq!(types::TRUE, 1);
        assert_eq!(types::FRACBITS, 16);
        assert_eq!(types::FRACUNIT, 1 << 16);
        assert_eq!(types::KEY_RIGHTARROW, 0xae);
        assert_eq!(types::KEY_LEFTARROW, 0xac);
        assert_eq!(types::KEY_UPARROW, 0xad);
        assert_eq!(types::KEY_DOWNARROW, 0xaf);
        assert_eq!(types::KEY_ESCAPE, 27);
        assert_eq!(types::KEY_ENTER, 13);
        assert_eq!(types::KEY_RALT, 0x80 + 0x38);
        assert_eq!(types::KEY_LALT, types::KEY_RALT);
    }
}
