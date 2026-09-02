# cdoom migration module registry

Canonical list of migratable units. Status is computed at runtime by `scripts/module-status.sh` — do not hand-edit status here.

| id | phase | c_files | rust_module | cmake_flag | deps |
|---|---|---|---|---|---|
| m_fixed | 1 | `chocolate-doom/src/m_fixed.c` | `cdoom-core/src/math.rs` | `USE_RUST_M_FIXED` | — |
| m_bbox | 1 | `chocolate-doom/src/m_bbox.c` | `cdoom-core/src/math.rs` | `USE_RUST_M_BBOX` | — |
| tables | 1 | `chocolate-doom/src/tables.c` | `cdoom-core/src/math.rs` | `USE_RUST_TABLES` | — |
| m_random | 1 | `chocolate-doom/src/doom/m_random.c`, `heretic/m_random.c`, `hexen/m_random.c`, `strife/m_random.c` | `cdoom-core/src/random.rs` | `USE_RUST_M_RANDOM` | — |
| sha1 | 2 | `chocolate-doom/src/sha1.c` | `cdoom-core/src/sha1.rs` | `USE_RUST_SHA1` | — |
| m_argv | 3 | `chocolate-doom/src/m_argv.c` | `cdoom-core/src/m_argv.rs` | `USE_RUST_M_ARGV` | phase 1 |
| m_misc | 3 | `chocolate-doom/src/m_misc.c` | `cdoom-core/src/m_misc.rs` | `USE_RUST_M_MISC` | phase 1 |
| w_lump_hash | 4 | `chocolate-doom/src/w_wad.c` (hash) | `cdoom-core/src/w/lump_hash.rs` | `USE_RUST_W_LUMP_NAME_HASH` | sha1, m_argv, m_misc |
| w_checksum | 4 | `chocolate-doom/src/w_checksum.c` | `cdoom-core/src/w/checksum.rs` | `USE_RUST_W_CHECKSUM` | sha1 |
| w_file_stdc | 4 | `chocolate-doom/src/w_file_stdc.c` | `cdoom-core/src/w/file_stdc.rs` | `USE_RUST_W_FILE_STDC` | — |
| w_file | 4 | `chocolate-doom/src/w_file.c` | `cdoom-core/src/w/file.rs` | `USE_RUST_W_FILE` | w_file_stdc |
| w_wad | 4 | `chocolate-doom/src/w_wad.c` | `cdoom-core/src/w/wad.rs` | `USE_RUST_W_WAD` | w_file, w_lump_hash |
| w_wad_cache | 4 | `chocolate-doom/src/w_wad.c` (cache) | `cdoom-core/src/w/wad_cache.rs` | `USE_RUST_W_WAD_CACHE` | w_wad |
| w_merge | 4 | `chocolate-doom/src/w_merge.c` | `cdoom-core/src/w/merge.rs` | `USE_RUST_W_MERGE` | w_wad |
| w_main | 4 | `chocolate-doom/src/w_main.c` | `cdoom-core/src/w/main.rs` | `USE_RUST_W_MAIN` | w_wad, w_merge |
| p_rejectpad | 12 | `chocolate-doom/src/p_rejectpad.c` | `cdoom-core/src/p_rejectpad/mod.rs` | `USE_RUST_P_REJECTPAD` | — |

## Name aliases

The migrator resolves these to `id`:

| User input | Resolves to |
|---|---|
| `m_fixed`, `m_fixed.c`, `USE_RUST_M_FIXED`, `FixedMul` | `m_fixed` |
| `m_bbox`, `m_bbox.c`, `USE_RUST_M_BBOX` | `m_bbox` |
| `tables`, `tables.c`, `USE_RUST_TABLES`, `SlopeDiv` | `tables` |
| `m_random`, `m_random.c`, `USE_RUST_M_RANDOM`, `random` | `m_random` |
| `sha1`, `sha1.c`, `USE_RUST_SHA1` | `sha1` |
| `m_argv`, `m_argv.c`, `USE_RUST_M_ARGV` | `m_argv` |
| `m_misc`, `m_misc.c`, `USE_RUST_M_MISC` | `m_misc` |
| `w_lump_hash`, `lump_hash`, `USE_RUST_W_LUMP_NAME_HASH` | `w_lump_hash` |
| `w_checksum`, `w_checksum.c`, `USE_RUST_W_CHECKSUM` | `w_checksum` |
| `w_file_stdc`, `w_file_stdc.c`, `USE_RUST_W_FILE_STDC` | `w_file_stdc` |
| `w_file`, `w_file.c`, `USE_RUST_W_FILE` | `w_file` |
| `w_wad`, `w_wad.c`, `USE_RUST_W_WAD` | `w_wad` |
| `w_wad_cache`, `USE_RUST_W_WAD_CACHE` | `w_wad_cache` |
| `w_merge`, `w_merge.c`, `USE_RUST_W_MERGE` | `w_merge` |
| `w_main`, `w_main.c`, `USE_RUST_W_MAIN` | `w_main` |
| `p_rejectpad`, `p_rejectpad.c`, `USE_RUST_P_REJECTPAD`, `PadRejectArray`, `rejectpad` | `p_rejectpad` |

## Deferred (not selectable yet)

These appear in [MIGRATION.md](../../../MIGRATION.md) but are not leaf-ready:

- Phase 5: `z_zone.c`, `z_native.c`
- Phase 6: `deh_*.c`
- Phase 7: `v_video.c`, `v_diskicon.c`
- Phase 8: `net_*.c`
- Phase 9: `i_*.c` (most)
- Phase 10: `p_*`, `r_*`, `g_*`, game logic
- Phase 3 deferred: `m_cheat.c`, `m_controls.c`, `m_config.c`
- Stays in C: trig LUTs in `tables.c`, RNG index globals, mmap backends
