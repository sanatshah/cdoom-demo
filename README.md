# cdoom

Local [Chocolate Doom](https://github.com/chocolate-doom/chocolate-doom) setup built from source, with free [Freedoom](https://freedoom.github.io/) game data.

## Prerequisites

- macOS with Xcode Command Line Tools (`xcode-select --install`)
- [Homebrew](https://brew.sh/)

## Quick start

```bash
./build.sh   # install deps and compile
./run.sh     # download Freedoom WADs (first run) and launch
```

## Playing Doom II (Freedoom Phase 2)

```bash
./run.sh -iwad wads/freedoom2.wad
```

## Controls

| Key | Action |
|-----|--------|
| Arrow keys | Move / turn |
| Ctrl | Fire |
| Space | Use / open doors |
| Shift | Run |
| Tab | Map |
| Esc | Menu |

## Project layout

```
cdoom/
  chocolate-doom/   # vendored upstream Chocolate Doom source
  cdoom-rust/       # Rust migration workspace (strangler-fig graft)
  wads/             # Freedoom WADs, auto-downloaded
  build.sh          # installs deps + compiles
  run.sh            # ensures WAD present, launches the game
  scripts/          # baseline verification for the Rust migration
```

## Rust migration (Step 0)

The `cdoom-rust/` workspace builds a static library linked into Chocolate Doom. See [cdoom-rust/README.md](cdoom-rust/README.md).

```bash
./chocolate-doom/build/src/chocolate-doom -cdoom-rust-info   # linked Rust version
./scripts/verify-baseline.sh                                 # migration oracle
```

## Notes

- `chocolate-doom/` is vendored upstream source from [Chocolate Doom](https://github.com/chocolate-doom/chocolate-doom).
- Game data comes from Freedoom (free and open source). No proprietary WADs are downloaded.
