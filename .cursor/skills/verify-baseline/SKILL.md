---
name: verify-baseline
description: Runs and interprets cdoom migration verification — cargo tests, FFI probe, timedemo oracle, and -cdoom-rust-info. Use after implementing or cutting over a module, or when debugging migration regressions.
---

# cdoom Baseline Verification

The migration oracle ensures Rust ports do not break vanilla compatibility. Run after every module change and before deleting C code.

## Quick run

From repo root:

```bash
./scripts/verify-baseline.sh
```

This executes, in order:

1. `cargo test --workspace` in `cdoom-rust/`
2. FFI probe build (if available)
3. `chocolate-doom -cdoom-rust-info`
4. Timedemo baseline (when binary + WAD exist)

## Prerequisites

```bash
./build.sh                          # CMake + Rust staticlib + Chocolate Doom
./run.sh                            # downloads Freedoom WADs on first run (optional for timedemo)
```

Disable Rust linking to compare C-only behavior:

```bash
cmake .. -DENABLE_CDOOM_RUST=OFF    # in chocolate-doom/build
```

## Individual checks

### Rust unit + parity tests

```bash
cd cdoom-rust && cargo test --workspace
```

| Failure type | Likely cause |
|---|---|
| Parity assertion mismatch | Rust logic diverges from C — fix implementation, not the test |
| Link error on cdoom-core | Missing `mod` registration or FFI export |
| cbindgen/build.rs error | Invalid type in exported FFI signature |

### FFI probe

```bash
cargo build --manifest-path cdoom-rust/Cargo.toml --release -p cdoom-core
./chocolate-doom/build/cdoom_rust_probe
```

Expect: `cdoom-rust probe OK: <version>`

| Failure | Action |
|---|---|
| Probe not built | Run `./build.sh` |
| `cdoom_rust_init failed` | Check init hook in `ffi.rs` |
| NULL version | Static string lifetime bug in FFI |

### In-game Rust info

```bash
./chocolate-doom/build/src/chocolate-doom -cdoom-rust-info
```

Prints the linked `cdoom-core` crate version. Confirms CMake linked the staticlib.

### Timedemo oracle

The compatibility gate for full engine behavior:

```bash
./chocolate-doom/build/src/chocolate-doom \
  -iwad wads/freedoom1.wad -timedemo demo1 -nosound -nomusic -nodraw
```

`verify-baseline.sh` runs this with a 120s timeout when `timeout` is available.

| Outcome | Meaning |
|---|---|
| Finishes successfully | No regression detected at demo playback level |
| Non-zero exit / timeout | Possible engine regression — investigate before cutover |
| Skip message | Binary or WAD missing — run `./build.sh && ./run.sh` |

Timedemo is the last gate before deleting C implementations. Parity tests catch module-level bugs; timedemo catches integration regressions.

## Verification checklist (module cutover)

```
Before enabling USE_RUST_<MODULE> in production:
- [ ] cargo test --workspace passes
- [ ] cdoom_rust_probe OK
- [ ] -cdoom-rust-info prints expected version

Before deleting C implementation:
- [ ] All above with USE_RUST_<MODULE> enabled
- [ ] Timedemo completes successfully
- [ ] No new warnings in chocolate-doom build
```

## Debugging regressions

1. **Isolate**: disable `USE_RUST_*` — if timedemo passes, the Rust port is the culprit.
2. **Narrow**: run `cargo test -p cdoom-verify <module>` for failing parity cases.
3. **Compare**: use dual-run to log C vs Rust outputs on the same inputs (temporary debug only).
4. **Rebuild clean**: `cargo clean` in `cdoom-rust/` then `./build.sh` if header staleness is suspected.

## CI / local parity

The workspace version is pinned to Chocolate Doom 3.1.1 (`CHOCOLATE_DOOM_VERSION` in `cdoom-verify/src/lib.rs`). Keep vendored upstream aligned.

## Do not

- Delete C source because parity tests alone pass — timedemo must also pass.
- Hand-edit `include/cdoom_rust.h` when probe fails — rebuild cdoom-core instead.
- Skip verification because the change "looks small" — fixed-point and WAD bugs are subtle.

## Related skills

- **migrator** — full migration loop
- **parity-test** — module-level correctness gate
