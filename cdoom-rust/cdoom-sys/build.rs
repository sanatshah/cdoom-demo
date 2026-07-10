use std::env;
use std::fs;
use std::path::PathBuf;

const FALLBACK_CONFIG_H: &str = r#"#ifndef CDOOM_SYS_BINDGEN_CONFIG_H
#define CDOOM_SYS_BINDGEN_CONFIG_H

#define HAVE_DECL_STRCASECMP 1
#define HAVE_DECL_STRNCASECMP 1
#define HAVE_DIRENT_H 1

#endif
"#;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=../../chocolate-doom/src/m_fixed.h");
    println!("cargo:rerun-if-changed=../../chocolate-doom/src/m_bbox.h");
    println!("cargo:rerun-if-changed=../../chocolate-doom/src/z_zone.h");
    println!("cargo:rerun-if-changed=../../chocolate-doom/src/w_wad.h");
    println!("cargo:rerun-if-changed=../../chocolate-doom/src/w_file.h");
    println!("cargo:rerun-if-changed=../../chocolate-doom/src/d_event.h");
    println!("cargo:rerun-if-env-changed=USE_RUST_BINDGEN");
    println!("cargo:rerun-if-env-changed=CDOOM_SOURCE_DIR");
    println!("cargo:rerun-if-env-changed=CDOOM_BINARY_DIR");
    println!("cargo:rerun-if-env-changed=CDOOM_BINDGEN_INCLUDE_DIRS");
    println!("cargo:rerun-if-env-changed=CDOOM_BINDGEN_EXTRA_CLANG_ARGS");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));
    let bindings_path = out_dir.join("bindings.rs");

    if !bindgen_enabled() {
        fs::write(
            bindings_path,
            "/* cdoom-sys bindgen disabled by USE_RUST_BINDGEN=OFF. */\n",
        )
        .expect("write disabled bindings");
        return;
    }

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));
    let include_dirs = include_dirs(&manifest_dir, &out_dir);
    let clang_args = include_dirs
        .iter()
        .map(|dir| format!("-I{}", dir.display()))
        .chain(extra_clang_args());

    bindgen::Builder::default()
        .header(manifest_dir.join("wrapper.h").display().to_string())
        .clang_arg("-std=c99")
        .clang_args(clang_args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("D_.*")
        .allowlist_function("Fixed.*")
        .allowlist_function("M_.*")
        .allowlist_function("W_.*")
        .allowlist_function("Z_.*")
        .allowlist_type("buttoncode2_t")
        .allowlist_type("buttoncode_t")
        .allowlist_type("event_t")
        .allowlist_type("evtype_t")
        .allowlist_type("fixed_t")
        .allowlist_type("lumpindex_t")
        .allowlist_type("lumpinfo_t")
        .allowlist_type("wad_file_t")
        .allowlist_var("BOX.*")
        .allowlist_var("BT.*")
        .allowlist_var("BTS.*")
        .allowlist_var("FRAC.*")
        .allowlist_var("PU_.*")
        .allowlist_var("ev_.*")
        .allowlist_var("lumpinfo")
        .allowlist_var("numlumps")
        .derive_default(true)
        .generate_comments(false)
        .layout_tests(false)
        .generate()
        .expect("bindgen failed for Chocolate Doom shared headers")
        .write_to_file(bindings_path)
        .expect("write bindings.rs");
}

fn bindgen_enabled() -> bool {
    match env::var("USE_RUST_BINDGEN") {
        Ok(value) => {
            let normalized = value.to_ascii_lowercase();
            !matches!(normalized.as_str(), "0" | "false" | "off" | "no")
        }
        Err(_) => true,
    }
}

fn include_dirs(manifest_dir: &PathBuf, out_dir: &PathBuf) -> Vec<PathBuf> {
    if let Ok(value) = env::var("CDOOM_BINDGEN_INCLUDE_DIRS") {
        let dirs: Vec<PathBuf> = value
            .split(';')
            .flat_map(env::split_paths)
            .filter(|path| !path.as_os_str().is_empty())
            .collect();
        if !dirs.is_empty() {
            return dirs;
        }
    }

    let repo_root = manifest_dir
        .join("../..")
        .canonicalize()
        .expect("canonicalize repo root");
    let source_dir = env::var_os("CDOOM_SOURCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join("chocolate-doom"));
    let binary_dir = env::var_os("CDOOM_BINARY_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join("chocolate-doom/build"));

    let mut dirs = vec![binary_dir, source_dir.join("src")];

    if !dirs.iter().any(|dir| dir.join("config.h").is_file()) {
        let fallback_dir = out_dir.join("cdoom-bindgen-config");
        fs::create_dir_all(&fallback_dir).expect("create fallback config include dir");
        fs::write(fallback_dir.join("config.h"), FALLBACK_CONFIG_H)
            .expect("write fallback config.h");
        dirs.insert(0, fallback_dir);
    }

    dirs
}

fn extra_clang_args() -> impl Iterator<Item = String> {
    env::var("CDOOM_BINDGEN_EXTRA_CLANG_ARGS")
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>()
        .into_iter()
}
