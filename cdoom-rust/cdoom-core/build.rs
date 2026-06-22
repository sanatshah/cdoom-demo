use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let include_dir = manifest_dir.join("../include");
    std::fs::create_dir_all(&include_dir).expect("create include/");

    let header_path = include_dir.join("cdoom_rust.h");
    let config_path = manifest_dir.join("../cbindgen.toml");
    cbindgen::Builder::new()
        .with_crate(&manifest_dir)
        .with_config(cbindgen::Config::from_file(config_path).unwrap())
        .generate()
        .expect("cbindgen failed")
        .write_to_file(header_path);

    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=../cbindgen.toml");
}
