# cdoom Rust migration

Chocolate Doom is being migrated to Rust incrementally using a [strangler-fig](https://martinfowler.com/bliki/StranglerFigApplication.html) pattern. The C binary stays the host; Rust grows as a linked static library with bit-exact parity gates at every step.

## Current status

**Step 0 (scaffolding) is complete on `main`.** The build graft, C ABI, and verification oracle are wired, but no engine modules route through Rust yet. All game logic still runs in the vendored C tree.

| Metric | Value |
|--------|-------|
| Migration phase | Step 0 |
| Rust modules on main | 0 (infrastructure only) |
| FFI exports | `cdoom_rust_version`, `cdoom_rust_init` |
| `USE_RUST_*` flags active | 0 |
| Rust tests passing | 3 (FFI smoke tests) |

Confirm the link:

```bash
./chocolate-doom/build/src/chocolate-doom -cdoom-rust-info
```

## Approach

### Strangler-fig graft

```
chocolate-doom/src/*.c  ──►  #ifdef USE_RUST_*  ──►  cdoom_rust_*()  ──►  cdoom-core
         ▲                                                          │
         └──────────── cdoom-verify parity tests ◄──────────────────┘
```

1. **C stays host.** Chocolate Doom executables (doom, heretic, hexen, strife) remain the entry point.
2. **Rust grows as a static library** (`libcdoom_core.a`) linked via CMake.
3. **C symbols are preserved.** Each migrated function keeps its original C name; the body delegates to Rust when a CMake flag is on.
4. **Parity before cutover.** Rust output must match C on fixed inputs before production code routes through Rust.
5. **Vanilla demo compatibility is the ultimate oracle.** `./scripts/verify-baseline.sh` runs timedemo against Freedoom as the end-to-end gate.

### Dual-run pattern

In the original C file, the public API stays unchanged:

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

Per-module `USE_RUST_*` flags default **OFF**. A flag flips to **ON** only after parity tests pass and the baseline oracle is green.

### Design constraints

- **Bit-exact parity** on fixed inputs — assert observable behavior, not implementation details.
- **Leaf-first ordering** — pure math and I/O before allocator and gameplay.
- **Minimal diffs in `chocolate-doom/`** — vendored upstream; only migration-focused changes.
- **All new Rust logic in `cdoom-rust/`** — never hand-edit `include/cdoom_rust.h`; cbindgen generates it on build.
- **Shared globals stay in C when awkward** — e.g. trig LUTs referenced by ~40 files, RNG index globals with external linkage.

## Architecture

| Layer | Path | Role |
|-------|------|------|
| Vendored C engine | `chocolate-doom/` | Upstream Chocolate Doom 3.1.1 (~296 `.c` files) |
| Rust workspace | `cdoom-rust/` | Migration modules and verification |
| FFI bridge | `cdoom-core/src/ffi.rs` → `cdoom-rust/include/cdoom_rust.h` | Thin C ABI with `cdoom_rust_*` prefix |
| CMake integration | `cdoom-rust/cmake/cdoom_rust.cmake` | Builds and links the static library |
| Verification | `cdoom-verify` + `scripts/verify-baseline.sh` | Parity tests + timedemo oracle |

### Crates

| Crate | Role |
|-------|------|
| `cdoom-core` | Migration modules + `staticlib` linked into Chocolate Doom |
| `cdoom-sys` | Future home for `bindgen` C bindings (placeholder) |
| `cdoom-verify` | Parity checks and baseline test helpers |

### Build pipeline

```
CMake (cdoom_rust.cmake)
  └── Cargo build -p cdoom-core
        ├── libcdoom_core.a
        └── cdoom_rust.h (cbindgen)
              └── linked into chocolate-doom / heretic / hexen / strife
```

## Migration loop

Every module port follows the same five steps — do not skip or reorder:

1. **Implement** in `cdoom-rust/cdoom-core/src/<module>/` (mirror the C file layout).
2. **Export** facade functions in `cdoom-core/src/ffi.rs` — cbindgen regenerates `include/cdoom_rust.h` on build.
3. **Dual-run** in the matching C file behind `#ifdef USE_RUST_<MODULE>` so C and Rust can be compared.
4. **Parity test** in `cdoom-rust/cdoom-verify/src/parity/<module>.rs` before routing production code through Rust.
5. **Flip** the CMake feature flag, then delete the C implementation only after `./scripts/verify-baseline.sh` passes.

### Verification gate

Before marking a module done:

```bash
cd cdoom-rust && cargo test --workspace
./scripts/verify-baseline.sh
```

## Phases

Phases are ordered by dependency and blast radius. Each phase may contain multiple modules, each with its own `USE_RUST_*` flag.

### Phase 0 — Scaffold (done)

Infrastructure: staticlib link, cbindgen C ABI, `-cdoom-rust-info` CLI probe, `verify-baseline.sh` oracle.

| Component | Location |
|-----------|----------|
| FFI exports | `cdoom-core/src/ffi.rs` |
| CMake graft | `cdoom-rust/cmake/cdoom_rust.cmake` |
| CLI hook | `chocolate-doom/src/i_main.c` |
| Baseline script | `scripts/verify-baseline.sh` |

CMake flag: `ENABLE_CDOOM_RUST` (links the static library; does not route any C code through Rust).

### Phase 1 — `m_` math + RNG

Fixed-point math (`FixedMul`/`FixedDiv`), bounding-box helpers, `SlopeDiv`, and the deterministic PRNG across all four games.

| C files | Rust module | CMake flags |
|---------|-------------|-------------|
| `m_fixed.c` | `math.rs` | `USE_RUST_M_FIXED` |
| `m_bbox.c` | `math.rs` | `USE_RUST_M_BBOX` |
| `tables.c` | `math.rs` | `USE_RUST_TABLES` |
| `*/m_random.c` (×4 games) | `random.rs` | `USE_RUST_M_RANDOM` |

**Stays in C:** trig LUTs (`finesine`, `finetangent`, `gammatable`); RNG index globals (`rndindex`, `prndindex`).

**Parity gate:** known-answer vectors for fixed-point ops; 512-step golden PRNG sequences (classic + Hexen tables).

**Risk:** low — pure, deterministic functions.

### Phase 2 — SHA-1 / memio

| C files | Rust module | CMake flags |
|---------|-------------|-------------|
| `sha1.c` | `sha1.rs` | `USE_RUST_SHA1` |
| `memio.c` | — | — |

**Parity gate:** known-answer SHA-1 digests (short and long inputs).

**Risk:** low.

### Phase 3 — `m_` utilities

Command-line parsing and file/string helpers.

| C files | Rust module | CMake flags |
|---------|-------------|-------------|
| `m_argv.c` | `m_argv.rs` | `USE_RUST_M_ARGV` |
| `m_misc.c` | `m_misc.rs` | `USE_RUST_M_MISC` |
| `m_cheat.c`, `m_controls.c`, `m_config.c` | — | — |

**Parity gate:** `M_CheckParm`, `M_fopen`, `M_StringCopy` parity tests.

**Risk:** medium — config and controls have wide call-site surface.

### Phase 4 — `w_` WAD stack

Full WAD subsystem: lump hash, stdio file I/O, WAD directory, lump cache, checksum, merge (NWT/flat/sprite), and main loader.

| C files | Rust module | CMake flags |
|---------|-------------|-------------|
| `w_wad.c` (hash) | `w/lump_hash.rs` | `USE_RUST_W_LUMP_NAME_HASH` |
| `w_checksum.c` | `w/checksum.rs` | `USE_RUST_W_CHECKSUM` |
| `w_file_stdc.c` | `w/file_stdc.rs` | `USE_RUST_W_FILE_STDC` |
| `w_file.c` | `w/file.rs` | `USE_RUST_W_FILE` |
| `w_wad.c` | `w/wad.rs`, `w/wad_cache.rs` | `USE_RUST_W_WAD`, `USE_RUST_W_WAD_CACHE` |
| `w_merge.c` | `w/merge.rs` | `USE_RUST_W_MERGE` |
| `w_main.c` | `w/main.rs` | `USE_RUST_W_MAIN` |

**Stays in C:** platform mmap backends (`w_file_posix.c`, `w_file_win32.c`).

**Parity gate:** lump hash, checksum, directory lookup, merge fixtures, argv hooks.

**Risk:** medium — depends on Phase 2 (SHA-1) and Phase 3 (argv/misc).

### Phase 5 — `z_` zone allocator

Global zone memory allocator with tag/purge semantics.

| C files | Notes |
|---------|-------|
| `z_zone.c`, `z_native.c` | Highest blast radius — allocations everywhere |

**Parity gate:** full timedemo must pass (every subsystem allocates through zone).

**Risk:** critical.

### Phase 6 — `deh_` Dehacked

DEH patch parsing and string/state table mutation.

| C files |
|---------|
| `deh_str.c`, `deh_io.c`, `deh_main.c`, `deh_mapping.c`, `deh_text.c` |

**Parity gate:** apply sample DEH patches; diff string and state tables.

**Risk:** medium.

### Phase 7 — `v_` video buffers

Off-screen buffer operations — blits and patch drawing (no SDL).

| C files |
|---------|
| `v_video.c`, `v_diskicon.c` |

**Parity gate:** pixel-buffer byte comparison.

**Risk:** low.

### Phase 8 — `net_` networking

Serialization first, then loopback, then client/server.

| C files |
|---------|
| `net_structrw.c`, `net_packet.c`, `net_loop.c`, client/server modules |

**Parity gate:** loopback parity before client/server migration.

**Risk:** high.

### Phase 9 — `i_` platform layer

SDL and OS integration — defer; migrate only tractable files.

| C files | Notes |
|---------|-------|
| `i_timer.c`, `i_system.c` | Candidates |
| `i_video.c`, `i_sound.c`, etc. | Likely stay in C |

**Parity gate:** per-file evaluation.

**Risk:** high.

### Phase 10 — Gameplay and rendering

Main loop, simulation, renderer, and game logic — the bulk of remaining work.

| Prefixes | Scope |
|----------|-------|
| `p_*` | Player / physics |
| `r_*` | Renderer |
| `g_*`, `s_*`, `hu_*`, `st_*` | Game logic, sound, HUD, status bar (per game) |

**Parity gate:** demo-playback end-of-level checksum.

**Risk:** critical — ~200k+ lines across four games.

## Phase dependencies

```
Phase 0 (scaffold)
  └── Phase 1 (math/RNG)
        └── Phase 3 (m_ utilities)
              ├── Phase 4 (WAD) ◄── Phase 2 (SHA-1)
              └── Phase 6 (Dehacked)
Phase 5 (zone) ──► Phase 4, Phase 10
Phase 7 (video) ──► Phase 10
Phase 8 (net) ──► Phase 10
Phase 9 (platform) ──► Phase 10
Phase 4, 5, 6, 7, 8, 9 ──► Phase 10 (gameplay/render)
```

## What stays in C (for now)

Even after Phases 1–4 land, these remain in C until a later phase explicitly targets them:

- **Trig LUTs** — `finesine`, `finetangent`, `gammatable` (referenced by ~40 files)
- **RNG index globals** — `rndindex`, `prndindex` (external linkage in `doomstat.h`)
- **Platform mmap backends** — `w_file_posix.c`, `w_file_win32.c`
- **SDL / video / sound** — `i_*` platform layer (Phase 9)
- **Zone allocator** — `z_zone.c` (Phase 5)
- **Gameplay and rendering** — `p_*`, `r_*`, `g_*` and related (Phase 10)

## Related docs

- [cdoom-rust/README.md](cdoom-rust/README.md) — workspace layout and build commands
- [README.md](README.md) — project quick start
- `.cursor/rules/cdoom-migration-workflow.mdc` — agent workflow rules
- `.cursor/skills/migrator/SKILL.md` — step-by-step module migration guide
