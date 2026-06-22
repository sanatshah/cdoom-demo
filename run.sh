#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

BINARY="$ROOT/chocolate-doom/build/src/chocolate-doom"
WAD_DIR="$ROOT/wads"
WAD1="$WAD_DIR/freedoom1.wad"

if [ ! -x "$BINARY" ]; then
  echo "Chocolate Doom binary not found. Run ./build.sh first." >&2
  exit 1
fi

download_freedoom() {
  echo "==> Downloading Freedoom WADs..."
  mkdir -p "$WAD_DIR"

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  local api_url="https://api.github.com/repos/freedoom/freedoom/releases/latest"
  local zip_url
  zip_url="$(curl -fsSL "$api_url" | python3 -c "
import json, sys
release = json.load(sys.stdin)
for asset in release.get('assets', []):
    name = asset.get('name', '')
    if name.startswith('freedoom-') and name.endswith('.zip'):
        print(asset['browser_download_url'])
        break
")"

  if [ -z "$zip_url" ]; then
    echo "Failed to find Freedoom release download URL." >&2
    exit 1
  fi

  echo "    Downloading from: $zip_url"
  curl -fsSL -o "$tmpdir/freedoom.zip" "$zip_url"
  unzip -q "$tmpdir/freedoom.zip" -d "$tmpdir"

  local wad1_src wad2_src
  wad1_src="$(find "$tmpdir" -name 'freedoom1.wad' -print -quit)"
  wad2_src="$(find "$tmpdir" -name 'freedoom2.wad' -print -quit)"

  if [ -z "$wad1_src" ] || [ -z "$wad2_src" ]; then
    echo "Failed to find freedoom1.wad or freedoom2.wad in release archive." >&2
    exit 1
  fi

  cp "$wad1_src" "$WAD_DIR/freedoom1.wad"
  cp "$wad2_src" "$WAD_DIR/freedoom2.wad"
  echo "    Installed WADs to $WAD_DIR/"
}

if [ ! -f "$WAD1" ]; then
  download_freedoom
fi

exec "$BINARY" -iwad "$WAD1" "$@"
