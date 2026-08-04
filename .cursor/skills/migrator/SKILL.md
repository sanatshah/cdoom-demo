---
name: migrator
description: End-to-end Chocolate Doom C→Rust module migration using the strangler-fig loop. Use when porting a C file, starting a migration phase, or asking how to migrate a subsystem in cdoom-rust.
---

# cdoom Module Migrator

Orchestrates one module port from `chocolate-doom/src/` into `cdoom-rust/`. Read this skill first; delegate details to sibling skills as needed.

## Prerequisites

- Repo builds: `./build.sh`
- Rust workspace at `cdoom-rust/` (see [cdoom-rust/README.md](../../cdoom-rust/README.md))

## Migration loop (do not skip steps)

```
Task Progress:
- [ ] 1. Select module and read C source
- [ ] 2. Implement Rust module in cdoom-core
- [ ] 3. Export FFI wrappers (see ffi-bridge skill)
- [ ] 4. Add C dual-run shim behind USE_RUST_<MODULE>
- [ ] 5. Write parity tests (see parity-test skill)
- [ ] 6. Verify baseline (see verify-baseline skill)
- [ ] 7. Flip CMake flag and delete C (only after step 6 passes)
```

## Step 1 — Select and analyze

Pick a **leaf module** first (no heavy engine dependencies): math (`m_fixed`, `m_bbox`, `tables`), misc (`m_argv`, `m_misc`), crypto (`sha1`), WAD layer (`w_*`).

For the chosen C file under `chocolate-doom/src/`:

1. List exported functions and globals.
2. Note C types used (`fixed_t`, `byte`, `wad_file_t`, etc.) — mirror names in Rust.
3. Identify side effects: `malloc`/`I_Realloc`, static mutable state, file I/O.
4. Check `#include` graph — if it pulls in rendering or game logic, defer.

## Step 2 — Implement in cdoom-core

Layout mirrors C:

| C location | Rust location |
|---|---|
| `chocolate-doom/src/foo.c` | `cdoom-rust/cdoom-core/src/foo.rs` |
| `chocolate-doom/src/w/foo.c` | `cdoom-rust/cdoom-core/src/w/foo.rs` |

Conventions:

- Register the module in `cdoom-core/src/lib.rs` (`mod foo; pub use foo::*;` as appropriate).
- Keep logic in Rust modules; keep `ffi.rs` as thin wrappers only.
- Preserve observable behavior, not line-by-line structure.
- Match integer widths and signedness at boundaries (`i32` for `fixed_t`, etc.).
- Reproduce static/global state faithfully — use `OnceLock`, `Mutex`, or module-level `static mut` only when C had mutable statics and parity requires it.

### C→Rust translation cheatsheet

| C pattern | Rust approach |
|---|---|
| `fixed_t` (16.16) | `i32` with `FRACBITS = 16` constant |
| `I_Error("msg")` | Return `Result` internally; map to C abort at FFI boundary if needed |
| `I_Realloc` growable array | `Vec<T>` |
| `M_StringCopy` / `M_StringDuplicate` | Owned `String` internally; C strings only at FFI |
| `SHA1_*` state machine | Struct with methods mirroring C call order |
| `#ifdef` game variants | Separate modules or `cfg` features only when C already branches |

## Step 3 — FFI export

Follow the **ffi-bridge** skill. Every exported function gets a `cdoom_rust_*` wrapper in `ffi.rs`.

## Step 4 — C dual-run shim

In the original C file, wrap the public API:

```c
#ifdef USE_RUST_FOO
#include "cdoom_rust.h"
#endif

void Foo_DoThing(int x)
{
#ifdef USE_RUST_FOO
    cdoom_rust_foo_do_thing(x);
#else
    /* existing C body */
#endif
}
```

Rules:

- Keep diffs in `chocolate-doom/` minimal and migration-focused.
- Do **not** delete C bodies until baseline verification passes with `USE_RUST_*` enabled.
- Add `USE_RUST_<MODULE>` to CMake only after parity tests land.

## Step 5 — Parity tests

Follow the **parity-test** skill. Tests must pass before flipping production routing.

## Step 6 — Verify

Follow the **verify-baseline** skill. Both must pass:

```bash
cd cdoom-rust && cargo test --workspace
./scripts/verify-baseline.sh
```

## Step 7 — Cut over

1. Enable `USE_RUST_<MODULE>` in CMake for production builds.
2. Re-run full verification.
3. Remove the C implementation and `#else` branch; keep the thin `#ifdef USE_RUST_*` include if other modules still dual-run.

## Module priority (Phase 1 candidates)

Already natural first targets based on dependency order:

1. Math/tables — `m_fixed.c`, `m_bbox.c`, `tables.c`
2. Misc/argv — `m_argv.c`, `m_misc.c`
3. SHA1 — `sha1.c`
4. WAD subsystem — `w_*.c` (depends on sha1 + file I/O)

## Anti-patterns

- Hand-editing `cdoom-rust/include/cdoom_rust.h` (cbindgen owns it).
- Putting game logic in `ffi.rs`.
- Flipping `USE_RUST_*` before parity tests exist.
- Large unrelated refactors in vendored `chocolate-doom/`.
- Deleting C before timedemo baseline still passes.

## Related skills

- **ffi-bridge** — C ABI, cbindgen, pointer safety
- **parity-test** — cdoom-verify test patterns
- **verify-baseline** — build oracle and timedemo gate
