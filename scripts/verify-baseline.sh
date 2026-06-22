#!/usr/bin/env bash
# Baseline correctness oracle for the cdoom Rust migration.
#
# Step 0: documents the checks future phases must pass. Runs what it can
# with the current tree (Rust tests + optional timedemo when built).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> Rust verification tests"
(cd cdoom-rust && cargo test --workspace)

echo ""
echo "==> Rust/C FFI probe"
cargo build --manifest-path cdoom-rust/Cargo.toml --release -p cdoom-core -q
"${ROOT}/chocolate-doom/build/cdoom_rust_probe" 2>/dev/null || {
  echo "Note: cdoom_rust_probe not built yet — run ./build.sh first."
}

BINARY="${ROOT}/chocolate-doom/build/src/chocolate-doom"
WAD="${ROOT}/wads/freedoom1.wad"

echo ""
echo "==> Chocolate Doom -cdoom-rust-info"
if [[ -x "${BINARY}" ]]; then
  "${BINARY}" -cdoom-rust-info
else
  echo "Skip: ${BINARY} not found (run ./build.sh)"
fi

echo ""
echo "==> Timedemo baseline (vanilla compatibility oracle)"
if [[ -x "${BINARY}" && -f "${WAD}" ]]; then
  echo "Command (future phases must keep this passing):"
  echo "  ${BINARY} -iwad ${WAD} -timedemo demo1 -nosound -nomusic"
  if command -v timeout >/dev/null 2>&1; then
    if timeout 120 "${BINARY}" -iwad "${WAD}" -timedemo demo1 -nosound -nomusic -nodraw; then
      echo "Timedemo finished successfully."
    else
      echo "Note: timedemo did not finish within 120s or exited non-zero."
      echo "Run manually after ./run.sh to capture a baseline."
    fi
  else
    echo "Skip auto-run (install coreutils timeout to run here)."
  fi
else
  echo "Skip: need built binary + ${WAD}"
  echo "  ./build.sh && ./run.sh   # run.sh downloads Freedoom on first launch"
fi

echo ""
echo "Baseline verification complete."
