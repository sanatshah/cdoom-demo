#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo "==> Installing build dependencies via Homebrew..."
brew install cmake fluid-synth libpng libsamplerate ninja rust sdl2 sdl2_mixer sdl2_net

if [ ! -d chocolate-doom ]; then
  echo "Error: chocolate-doom/ source directory not found." >&2
  exit 1
fi

echo "==> Configuring and building..."
mkdir -p chocolate-doom/build
cd chocolate-doom/build
cmake .. -G Ninja -DCMAKE_BUILD_TYPE=Release
ninja

echo ""
echo "Build complete: $ROOT/chocolate-doom/build/src/chocolate-doom"
echo "Rust probe:       $ROOT/chocolate-doom/build/cdoom_rust_probe"
echo "Run the game with: ./run.sh"
echo "Verify baseline:  ./scripts/verify-baseline.sh"
