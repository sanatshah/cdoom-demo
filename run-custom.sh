#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WAD="$ROOT/wads/custom.wad"

if [ ! -f "$WAD" ]; then
  python3 "$ROOT/tools/make_custom_map.py" -o "$WAD"
fi

exec "$ROOT/run.sh" -file "$WAD" -warp 1 1 -skill 3 "$@"
