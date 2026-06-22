# cdoom

Local [Chocolate Doom](https://github.com/chocolate-doom/chocolate-doom) setup built from source, with free [Freedoom](https://freedoom.github.io/) game data.

## Prerequisites

- macOS with Xcode Command Line Tools (`xcode-select --install`)
- [Homebrew](https://brew.sh/)

## Quick start

```bash
./build.sh   # install deps, clone source, compile
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
  chocolate-doom/   # cloned upstream source (gitignored)
  wads/             # Freedoom WADs, auto-downloaded (gitignored)
  build.sh          # installs deps + clones + compiles
  run.sh            # ensures WAD present, launches the game
```

## Notes

- `chocolate-doom/` is a plain git clone of the upstream repo, not a submodule.
- Game data comes from Freedoom (free and open source). No proprietary WADs are downloaded.
