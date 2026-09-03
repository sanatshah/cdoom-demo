#!/usr/bin/env bash
# Cloud Agent install: bootstrap the cdoom dev environment on Cursor's default
# Ubuntu base image. Idempotent — safe to re-run on cached/partial state.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# --- System build dependencies -------------------------------------------------
# Chocolate Doom needs the SDL2 stack plus PNG/samplerate/FluidSynth, and the
# build uses CMake + Ninja. build.sh installs these via Homebrew (macOS only);
# on the Linux Cloud Agent image we use apt instead.
export DEBIAN_FRONTEND=noninteractive
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends \
  build-essential cmake ninja-build pkg-config git curl unzip ca-certificates python3 \
  libsdl2-dev libsdl2-mixer-dev libsdl2-net-dev libpng-dev libsamplerate0-dev libfluidsynth-dev

# --- Rust toolchain -------------------------------------------------------------
# cdoom-rust/rust-toolchain.toml pins channel = "stable". cbindgen's build deps
# require edition2024 -> rustc >= 1.85, so the image's default 1.83 is too old.
# Install stable only if absent: a bare re-install tries an in-place upgrade of an
# existing stable and dies renaming across overlay layers (EXDEV, os error 18).
rustup toolchain list | grep -q '^stable-' || rustup toolchain install stable --profile minimal
rustup default stable

# --- Build the project ----------------------------------------------------------
# Build the Rust staticlib first: cbindgen regenerates include/cdoom_rust.h during
# this build, and CMake configure fails if that header is missing.
cargo build --manifest-path cdoom-rust/Cargo.toml --release -p cdoom-core

# Configure + build Chocolate Doom (links libcdoom_core.a into the game binaries).
cmake -S chocolate-doom -B chocolate-doom/build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build chocolate-doom/build

echo "cdoom install complete: $ROOT/chocolate-doom/build/src/chocolate-doom"
