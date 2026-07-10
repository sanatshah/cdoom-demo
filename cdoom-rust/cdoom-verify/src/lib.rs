//! Shared verification helpers for migration phases.
//!
//! Each new Rust module should add parity checks here (or as integration tests)
//! before flipping the CMake feature flag that routes production code through Rust.

use std::ffi::CStr;
#[cfg(test)]
use std::fs;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

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
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cdoom-verify has workspace parent")
        .parent()
        .expect("workspace has repo parent")
        .to_path_buf()
}

#[cfg(test)]
fn c_tables_source() -> String {
    fs::read_to_string(repo_root().join("chocolate-doom/src/tables.c"))
        .expect("read chocolate-doom/src/tables.c")
}

#[cfg(test)]
fn c_array_body<'a>(source: &'a str, name: &str) -> &'a str {
    let declaration_pos = [
        format!("const fixed_t {name}"),
        format!("const angle_t {name}"),
        format!("const byte {name}"),
    ]
    .into_iter()
    .find_map(|needle| source.find(&needle))
    .unwrap_or_else(|| panic!("array {name} not found"));
    let open_pos = source[declaration_pos..]
        .find('{')
        .map(|offset| declaration_pos + offset)
        .unwrap_or_else(|| panic!("array {name} has no opening brace"));

    let mut depth = 0usize;
    for (offset, ch) in source[open_pos..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open_pos + 1..open_pos + offset];
                }
            }
            _ => {}
        }
    }

    panic!("array {name} has no closing brace");
}

#[cfg(test)]
fn parse_c_numbers(source: &str, name: &str) -> Vec<i64> {
    c_array_body(source, name)
        .lines()
        .flat_map(|line| line.split("//").next().unwrap_or("").split(','))
        .map(str::trim)
        .filter(|token| !token.is_empty() && *token != "{" && *token != "}")
        .map(|token| {
            let token = token.trim_matches('{').trim_matches('}').trim();
            token
                .parse::<i64>()
                .unwrap_or_else(|err| panic!("parse {name} value {token:?}: {err}"))
        })
        .collect()
}

#[cfg(test)]
fn i32_bytes(values: &[i32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect()
}

#[cfg(test)]
fn u32_bytes(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn tables_luts_match_c_bytes() {
        let source = c_tables_source();

        let c_finetangent: Vec<i32> = parse_c_numbers(&source, "finetangent")
            .into_iter()
            .map(|value| value as i32)
            .collect();
        let c_finesine: Vec<i32> = parse_c_numbers(&source, "finesine")
            .into_iter()
            .map(|value| value as i32)
            .collect();
        let c_tantoangle: Vec<u32> = parse_c_numbers(&source, "tantoangle")
            .into_iter()
            .map(|value| value as u32)
            .collect();
        let c_gammatable: Vec<u8> = parse_c_numbers(&source, "gammatable")
            .into_iter()
            .map(|value| value as u8)
            .collect();

        assert_eq!(
            i32_bytes(&c_finetangent),
            i32_bytes(&cdoom_core::tables::FINETANGENT)
        );
        assert_eq!(
            i32_bytes(&c_finesine),
            i32_bytes(&cdoom_core::tables::FINESINE)
        );
        assert_eq!(
            u32_bytes(&c_tantoangle),
            u32_bytes(&cdoom_core::tables::TANTOANGLE)
        );
        assert_eq!(c_gammatable, cdoom_core::tables::GAMMATABLE_FLAT);
    }

    #[test]
    fn finecosine_reuses_finesine_quarter_turn() {
        assert_eq!(
            cdoom_core::tables::finecosine_slice()[0],
            cdoom_core::tables::FINESINE[cdoom_core::tables::FINEANGLES / 4]
        );
    }

    #[test]
    fn slope_div_matches_c_edge_cases() {
        assert_eq!(cdoom_core::tables::slope_div(0, 0), 2048);
        assert_eq!(cdoom_core::tables::slope_div(12345, 511), 2048);
        assert_eq!(cdoom_core::tables::slope_div(0, 512), 0);
        assert_eq!(cdoom_core::tables::slope_div(1, 512), 4);
        assert_eq!(cdoom_core::tables::slope_div(512, 512), 2048);
        assert_eq!(cdoom_core::tables::slope_div(513, 512), 2048);
        assert_eq!(cdoom_core::tables::slope_div(u32::MAX, u32::MAX), 256);
    }

    #[test]
    fn type_foundations_match_c_header_values() {
        assert_eq!(cdoom_core::types::FRACBITS, 16);
        assert_eq!(cdoom_core::types::FRACUNIT, 65536);
        assert_eq!(cdoom_core::types::FALSE, 0);
        assert_eq!(cdoom_core::types::TRUE, 1);
        assert_eq!(cdoom_core::types::KEY_UPARROW, 0xad);
        assert_eq!(cdoom_core::types::KEY_RIGHTARROW, 0xae);
        assert_eq!(cdoom_core::types::KEY_RCTRL, 0x80 + 0x1d);
        assert_eq!(
            cdoom_core::types::SCANCODE_TO_KEYS[82],
            cdoom_core::types::KEY_UPARROW
        );
        assert_eq!(cdoom_core::types::doom_short(0xff80), -128);
        assert_eq!(cdoom_core::types::doom_long(0xffff_ff80), -128);
    }
}
