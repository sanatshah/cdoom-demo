#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo "==> Installing build dependencies via Homebrew..."
brew install cmake fluid-synth libpng libsamplerate ninja sdl2 sdl2_mixer sdl2_net

echo "==> Cloning or updating chocolate-doom source..."
if [ -d chocolate-doom/.git ]; then
  git -C chocolate-doom pull --ff-only
else
  git clone https://github.com/chocolate-doom/chocolate-doom.git chocolate-doom
fi

echo "==> Configuring and building..."
mkdir -p chocolate-doom/build
cd chocolate-doom/build
cmake .. -G Ninja -DCMAKE_BUILD_TYPE=Release
ninja

echo ""
echo "Build complete: $ROOT/chocolate-doom/build/src/chocolate-doom"
echo "Run the game with: ./run.sh"
