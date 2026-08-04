---
name: parity-test
description: Authors cdoom-verify parity tests that prove Rust modules match C behavior before USE_RUST flags flip. Use when adding migration tests, comparing Rust vs C outputs, or registering parity modules.
---

# cdoom Parity Tests

Parity tests are the gate between dual-run shims and production cutover. **No `USE_RUST_*` flag flips without passing parity tests.**

## Layout

```
cdoom-rust/cdoom-verify/src/
  lib.rs              # shared helpers
  parity/
    mod.rs            # register modules here
    m_fixed.rs        # one file per migrated module
    sha1.rs
    w_checksum.rs
```

Run all tests:

```bash
cd cdoom-rust && cargo test --workspace
```

Run one module:

```bash
cargo test -p cdoom-verify fixed_mul
```

## Adding a new parity module

### 1. Create `parity/<module>.rs`

```rust
use cdoom_core::m_fixed;

#[test]
fn fixed_mul_known_vectors() {
    let cases: &[(i32, i32, i32)] = &[
        (0x00010000, 0x00020000, 0x00020000), // 1.0 * 2.0 = 2.0
        (0, 0x7FFFFFFF, 0),
        (-0x00010000, 0x00010000, -0x00010000),
    ];
    for &(a, b, expected) in cases {
        assert_eq!(m_fixed::fixed_mul(a, b), expected, "a={a:#x} b={b:#x}");
    }
}
```

### 2. Register in `parity/mod.rs`

```rust
mod m_fixed;
```

For integration-style tests that need the C ABI:

```rust
#[test]
fn ffi_fixed_mul_roundtrip() {
    let result = cdoom_core::cdoom_rust_fixed_mul(0x00010000, 0x00020000);
    assert_eq!(result, 0x00020000);
}
```

## What to assert

| Category | Assert on |
|---|---|
| Pure functions | Return values for representative + edge inputs |
| Stateful APIs | Final state after same call sequence as C |
| Hash/checksum | Exact byte output for fixed inputs |
| String/format | Exact bytes including NUL termination |
| Error paths | Same return codes or abort behavior as C |

## Deriving test vectors from C

1. Read the C implementation and identify edge cases (overflow, zero, INT_MIN, empty input).
2. For lookup tables (`tables.c`), compare entire static arrays — use a small C probe or transcribe from source.
3. For hashes, pick 2–3 small known inputs and compare digests byte-for-byte.
4. Document the C source line or comment that justifies each vector.

```rust
// Edge case from m_fixed.c FixedDiv: divisor overflow guard
#[test]
fn fixed_div_overflow_saturation() {
    assert_eq!(m_fixed::fixed_div(i32::MIN, 1), i32::MIN);
}
```

## Test categories checklist

```
Parity coverage:
- [ ] Happy-path inputs
- [ ] Zero / empty inputs
- [ ] Sign boundary (negative, INT_MIN, INT_MAX)
- [ ] Fixed-point fractional edge cases (FRACBITS = 16)
- [ ] Sequential stateful calls (init → update → final)
- [ ] FFI wrapper round-trip (if exported)
```

## Determinism requirements

- No `rand` without a fixed seed matching C's PRNG state.
- No platform-specific paths in assertions unless C is also platform-specific.
- Avoid asserting on debug formatting — compare raw bytes/ints.

## Property-style spot checks

For math functions, a few randomized spot checks against a C reference are optional **after** table-driven cases exist:

```rust
#[test]
fn fixed_mul_associativity_spot() {
    for (a, b) in [(0x00008000, 0x00008000), (0x00030000, 0x00005000)] {
        let ab = m_fixed::fixed_mul(a, b);
        // ab is the oracle — derived from known C output, not recomputed differently
        assert_eq!(ab, m_fixed::fixed_mul(a, b));
    }
}
```

Do not use a different algorithm as the oracle — the C output (or exact C port) is the source of truth.

## Dependencies

`cdoom-verify/Cargo.toml` depends on `cdoom-core`. Test internal Rust APIs directly; use FFI wrappers when validating the full bridge.

## Do not

- Flip `USE_RUST_*` in CMake before parity tests land and pass.
- Assert on internal data structures (private fields, Vec capacity).
- Skip edge cases that the C code explicitly handles.
- Delete C implementations to "force" parity — dual-run stays until baseline passes.

## After parity passes

Proceed to **verify-baseline** skill, then enable the CMake `USE_RUST_*` flag.
