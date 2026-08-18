#!/usr/bin/env bash
# Report migration status for each module in modules.md registry.
# Usage: ./scripts/module-status.sh [--pending-only] [--json]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../../../" && pwd)"
cd "$ROOT"

PENDING_ONLY=0
JSON=0
for arg in "$@"; do
  case "$arg" in
    --pending-only) PENDING_ONLY=1 ;;
    --json) JSON=1 ;;
  esac
done

# id|c_files|rust_module|cmake_flag|phase
MODULES=(
  "m_fixed|chocolate-doom/src/m_fixed.c|cdoom-rust/cdoom-core/src/math.rs|USE_RUST_M_FIXED|1"
  "m_bbox|chocolate-doom/src/m_bbox.c|cdoom-rust/cdoom-core/src/math.rs|USE_RUST_M_BBOX|1"
  "tables|chocolate-doom/src/tables.c|cdoom-rust/cdoom-core/src/math.rs|USE_RUST_TABLES|1"
  "m_random|chocolate-doom/src/doom/m_random.c|cdoom-rust/cdoom-core/src/random.rs|USE_RUST_M_RANDOM|1"
  "sha1|chocolate-doom/src/sha1.c|cdoom-rust/cdoom-core/src/sha1.rs|USE_RUST_SHA1|2"
  "m_argv|chocolate-doom/src/m_argv.c|cdoom-rust/cdoom-core/src/m_argv.rs|USE_RUST_M_ARGV|3"
  "m_cheat|chocolate-doom/src/m_cheat.c|cdoom-rust/cdoom-core/src/m_cheat.rs|USE_RUST_M_CHEAT|3"
  "m_misc|chocolate-doom/src/m_misc.c|cdoom-rust/cdoom-core/src/m_misc.rs|USE_RUST_M_MISC|3"
  "w_lump_hash|chocolate-doom/src/w_wad.c|cdoom-rust/cdoom-core/src/w/lump_hash.rs|USE_RUST_W_LUMP_NAME_HASH|4"
  "w_checksum|chocolate-doom/src/w_checksum.c|cdoom-rust/cdoom-core/src/w/checksum.rs|USE_RUST_W_CHECKSUM|4"
  "w_file_stdc|chocolate-doom/src/w_file_stdc.c|cdoom-rust/cdoom-core/src/w/file_stdc.rs|USE_RUST_W_FILE_STDC|4"
  "w_file|chocolate-doom/src/w_file.c|cdoom-rust/cdoom-core/src/w/file.rs|USE_RUST_W_FILE|4"
  "w_wad|chocolate-doom/src/w_wad.c|cdoom-rust/cdoom-core/src/w/wad.rs|USE_RUST_W_WAD|4"
  "w_wad_cache|chocolate-doom/src/w_wad.c|cdoom-rust/cdoom-core/src/w/wad_cache.rs|USE_RUST_W_WAD_CACHE|4"
  "w_merge|chocolate-doom/src/w_merge.c|cdoom-rust/cdoom-core/src/w/merge.rs|USE_RUST_W_MERGE|4"
  "w_main|chocolate-doom/src/w_main.c|cdoom-rust/cdoom-core/src/w/main.rs|USE_RUST_W_MAIN|4"
)

has_rust() {
  local rust_path="$1"
  [[ -f "$rust_path" ]] || grep -q "mod ${rust_path##*/}" "cdoom-rust/cdoom-core/src/lib.rs" 2>/dev/null
}

has_parity() {
  local id="$1"
  local parity_dir="cdoom-rust/cdoom-verify/src/parity"
  [[ -d "$parity_dir" ]] || return 1
  ls "$parity_dir"/*.rs 2>/dev/null | grep -qiE "${id}|${id//_/.}|math_parity|random_parity" && return 0
  grep -rq "$id" "$parity_dir" 2>/dev/null
}

has_shim() {
  local c_file="$1"
  local flag="$2"
  [[ -f "$c_file" ]] && grep -q "$flag" "$c_file" 2>/dev/null
}

has_cmake_flag() {
  local flag="$1"
  grep -rq "$flag" cdoom-rust/cmake chocolate-doom/CMakeLists.txt chocolate-doom/src 2>/dev/null
}

status_for() {
  local id c_file rust flag phase
  IFS='|' read -r id c_file rust flag phase <<< "$1"

  local rust_ok=0 parity_ok=0 shim_ok=0 cmake_ok=0
  has_rust "$rust" && rust_ok=1
  has_parity "$id" && parity_ok=1
  has_shim "$c_file" "$flag" && shim_ok=1
  has_cmake_flag "$flag" && cmake_ok=1

  if [[ $cmake_ok -eq 1 ]]; then
    echo "cutover"
  elif [[ $rust_ok -eq 1 || $parity_ok -eq 1 || $shim_ok -eq 1 ]]; then
    echo "in_progress"
  else
    echo "pending"
  fi
}

if [[ $JSON -eq 1 ]]; then
  echo "["
  first=1
  for entry in "${MODULES[@]}"; do
    IFS='|' read -r id c_file rust flag phase <<< "$entry"
    st="$(status_for "$entry")"
    [[ $PENDING_ONLY -eq 1 && "$st" != "pending" && "$st" != "in_progress" ]] && continue
    [[ $first -eq 0 ]] && echo ","
    first=0
    printf '  {"id":"%s","phase":%s,"status":"%s","c_file":"%s","rust":"%s","flag":"%s"}' \
      "$id" "$phase" "$st" "$c_file" "$rust" "$flag"
  done
  echo
  echo "]"
else
  printf "%-14s %-6s %-12s %s\n" "MODULE" "PHASE" "STATUS" "C_FILE"
  printf "%-14s %-6s %-12s %s\n" "------" "-----" "------" "------"
  for entry in "${MODULES[@]}"; do
    IFS='|' read -r id c_file rust flag phase <<< "$entry"
    st="$(status_for "$entry")"
    [[ $PENDING_ONLY -eq 1 && "$st" == "cutover" ]] && continue
    printf "%-14s %-6s %-12s %s\n" "$id" "$phase" "$st" "$c_file"
  done
fi
