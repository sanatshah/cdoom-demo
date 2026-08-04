---
name: migrator
description: End-to-end Chocolate Doom C→Rust module migration using the strangler-fig loop. Use when porting a C file, starting a migration phase, or asking how to migrate a subsystem in cdoom-rust. Lists unmigrated modules for selection or accepts a module name.
---

# cdoom Module Migrator

Orchestrates one module port from `chocolate-doom/src/` into `cdoom-rust/`. Read this skill first; delegate details to sibling skills as needed.

## Prerequisites

- Repo builds: `./build.sh`
- Rust workspace at `cdoom-rust/` (see [cdoom-rust/README.md](../../cdoom-rust/README.md))

## Step 0 — Select module

Every migration run targets **one module** from the registry. Two entry paths:

### A. User passes a module name

If the user names a module (e.g. "migrate m_fixed", "port sha1", "USE_RUST_M_BBOX"):

1. Resolve the name to a registry `id` using the alias table in [modules.md](modules.md).
2. Run status script to confirm it exists and show current status:
   ```bash
   .cursor/skills/migrator/scripts/module-status.sh
   ```
3. If the name doesn't match any registry entry, say so and offer the pending list (path B).
4. If status is `cutover`, tell the user it's already routed through Rust and ask whether to continue cleanup or pick another module.
5. Proceed to Step 1 with the resolved module.

### B. No module specified — show choices

When the user says "migrate a module", "what's next", "start migration", or similar without naming one:

1. Run:
   ```bash
   .cursor/skills/migrator/scripts/module-status.sh --pending-only
   ```
2. Present **pending** and **in_progress** modules grouped by phase, showing `id`, C file, and status.
3. Recommend the lowest-phase pending module whose dependencies are satisfied (see deps column in [modules.md](modules.md)).
4. Use **AskQuestion** so the user picks one module. Include an "Other (type module name)" option.
5. Proceed to Step 1 with the chosen module.

### Registry reference

Full module list, aliases, and deferred modules: [modules.md](modules.md)

| Status | Meaning |
|---|---|
| `pending` | No Rust, parity, or dual-run work detected |
| `in_progress` | Partial work (Rust and/or parity and/or shim) but not cut over |
| `cutover` | `USE_RUST_*` flag active in CMake |

## Migration loop (do not skip steps)

```
Task Progress:
- [ ] 0. Select module (above)
- [ ] 1. Read C source for chosen module
- [ ] 2. Implement Rust module in cdoom-core
- [ ] 3. Export FFI wrappers (see ffi-bridge skill)
- [ ] 4. Add C dual-run shim behind USE_RUST_<MODULE>
- [ ] 5. Write parity tests (see parity-test skill)
- [ ] 6. Verify baseline (see verify-baseline skill)
- [ ] 7. Flip CMake flag and delete C (only after step 6 passes)
```

## Step 1 — Analyze chosen module

Look up the module in [modules.md](modules.md) for C file path, Rust target, and CMake flag.

For the C file(s):

1. List exported functions and globals.
2. Note C types used (`fixed_t`, `byte`, `wad_file_t`, etc.) — mirror names in Rust.
3. Identify side effects: `malloc`/`I_Realloc`, static mutable state, file I/O.
4. Check `#include` graph — if it pulls in rendering or game logic, defer.
5. Note anything marked "stays in C" in [MIGRATION.md](../../../MIGRATION.md) for this module (e.g. trig LUTs in `tables.c`).

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

## Anti-patterns

- Hand-editing `cdoom-rust/include/cdoom_rust.h` (cbindgen owns it).
- Putting game logic in `ffi.rs`.
- Flipping `USE_RUST_*` before parity tests exist.
- Large unrelated refactors in vendored `chocolate-doom/`.
- Deleting C before timedemo baseline still passes.
- Migrating a module not in the registry without updating [modules.md](modules.md) first.

## Related skills

- **ffi-bridge** — C ABI, cbindgen, pointer safety
- **parity-test** — cdoom-verify test patterns
- **verify-baseline** — build oracle and timedemo gate

## Related subagents

Delegate isolated steps to project subagents in `.cursor/agents/`:

| Step | Subagent | When to delegate |
|---|---|---|
| 1–2 | **rust-port** | Analyze C source and implement in `cdoom-core` |
| 3–4 | **ffi-bridge** | Export `ffi.rs`, regenerate header, add C dual-run shims |
| 5 | **parity-test** | Author and register `cdoom-verify` parity tests |
| 6–7 | **verify-baseline** | Run `./scripts/verify-baseline.sh`, interpret failures, gate cutover |

Example: `Use the rust-port subagent to port m_fixed.c`
