# AGENTS.md

## Cursor Cloud specific instructions

This repo (`cdoom`) is a from-source build of **Chocolate Doom** (C, via CMake/Ninja)
with a grafted **Rust** migration workspace (`cdoom-rust/`, Cargo). Game data is
Freedoom; the WADs are already committed in `wads/`. See `README.md` and
`cdoom-rust/README.md` for the high-level layout and standard commands.

### Building (Linux — the documented `build.sh` is macOS/Homebrew only)
On this VM, dependencies are installed by the startup update script, so skip
`build.sh` (it runs `brew install`). Build directly:

1. **Generate the Rust header first.** A clean tree has no `cdoom-rust/include/`
   (it is gitignored and produced by cbindgen during the Rust build). CMake
   configure will FAIL with `Imported target "cdoom_rust" includes non-existent
   path ".../cdoom-rust/include"` if you skip this step. Run:
   `cargo build --manifest-path cdoom-rust/Cargo.toml --release -p cdoom-core`
2. Then configure + build:
   `mkdir -p chocolate-doom/build && cd chocolate-doom/build`
   `cmake .. -G Ninja -DCMAKE_BUILD_TYPE=Release && ninja`
   Outputs: `chocolate-doom/build/src/chocolate-doom` and `.../cdoom_rust_probe`.
   (Pass `-DENABLE_CDOOM_RUST=OFF` to skip Rust entirely; it is ON by default and
   links `libcdoom_core.a` into all game binaries.)

### Rust toolchain
`cdoom-rust/rust-toolchain.toml` pins `channel = "stable"`. Build deps (e.g.
`cbindgen`'s `clap`) require **edition2024 → rustc ≥ 1.85**. The base image's
*default* rustup toolchain may be an **older** pinned rustc (e.g. `1.83.0`) with
no `stable` toolchain installed. That default is too old: any root-level
`cargo build --manifest-path cdoom-rust/Cargo.toml ...` (used in build step 1
above and in `scripts/verify-baseline.sh`) fails with
`feature 'edition2024' is required ... not stabilized in this version of Cargo`.
`rustup` only honors `cdoom-rust/rust-toolchain.toml` when the cwd is inside
`cdoom-rust/`, so the CMake cargo invocation (which cds into `cdoom-rust/`) is
fine but root-level invocations are not. The startup update script therefore
installs `stable` (≥ 1.85) if absent **and makes it the default** with
`rustup default stable`, so cargo picks a new-enough toolchain everywhere.

Do NOT run a bare `rustup toolchain install stable` unconditionally at startup.
If a `stable` toolchain already lived in the read-only overlay lower layer,
rustup would attempt an in-place *upgrade* whenever upstream publishes a newer
stable and die moving the old component out of the tree:
`error: could not rename ... Invalid cross-device link (os error 18)` (EXDEV),
making setup exit non-zero (`INSTALL_FAILED`). The update script guards the
install so it only runs when `stable` is missing:
`rustup toolchain list | grep -q '^stable-' || rustup toolchain install stable --profile minimal`.

### Tests / verification
- Rust unit tests: `cd cdoom-rust && cargo test --workspace` (3 tests in `cdoom-verify`).
- Migration oracle: `./scripts/verify-baseline.sh` (Rust tests + FFI probe +
  `-cdoom-rust-info` + headless timedemo).
- FFI smoke test: `./chocolate-doom/build/cdoom_rust_probe`.
- Rust version exposed to C: `./chocolate-doom/build/src/chocolate-doom -cdoom-rust-info`.

### Running the game
`./run.sh` launches the game with `wads/freedoom1.wad` (WADs already present, so
its download path is skipped). GUI requires a display — a VNC desktop is available
on `DISPLAY=:1`. Config/saves land in `~/.local/share/chocolate-doom/`.

### Known caveat — headless timedemo does not self-exit
`chocolate-doom ... -timedemo demo1 -nodraw -nosound -nomusic` runs the full
engine simulation and prints e.g. `timed 7117 gametics in 3 realtics (... fps)`,
but the headless process then hangs instead of exiting. `verify-baseline.sh`
wraps it in `timeout 120` and prints "timedemo did not finish within 120s"; this
is expected here and is NOT a build/regression failure — the gametics/fps line
proves the engine ran.
