#!/usr/bin/env bash
# Measure incremental build, test, and clippy speeds after a 1-line source change.
#
# usage: tools/build_speed.sh [--release] [--out <file>]
set -euo pipefail

MEASURE_RELEASE=0
OUT_FILE=""
ORIG_ARGS=("$@")

while [ $# -gt 0 ]; do
  case "$1" in
    --release) MEASURE_RELEASE=1; shift ;;
    --out)
      if [ -z "${2:-}" ]; then
        echo "build_speed: --out requires a file path" >&2
        exit 2
      fi
      OUT_FILE="$2"
      shift 2
      ;;
    -h|--help)
      echo "usage: tools/build_speed.sh [--release] [--out <file>]"
      exit 0
      ;;
    *)
      echo "build_speed: unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [ -n "$OUT_FILE" ]; then
  case "$OUT_FILE" in /*) ;; *) OUT_FILE="$PWD/$OUT_FILE" ;; esac
fi

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$ROOT" ]; then
  echo "build_speed: not inside a git repository" >&2
  exit 1
fi

lock_held_by_ancestor() {
  local pid=$PPID
  while [ "$pid" -gt 1 ] 2>/dev/null; do
    if tr '\0' '\n' < "/proc/$pid/cmdline" 2>/dev/null | awk 'NR==1 && $0 !~ /(^|\/)flock$/ {exit 1} $0=="/tmp/rotland-heavy.lock" {found=1} END {exit !found}'; then
      return 0
    fi
    pid=$(awk '{print $4}' "/proc/$pid/stat" 2>/dev/null || echo 1)
  done
  return 1
}

if [ "${ROTLAND_GATE_LOCKED:-}" != "1" ] && lock_held_by_ancestor; then
  export ROTLAND_GATE_LOCKED=1
fi
if [ "${ROTLAND_GATE_LOCKED:-}" != "1" ]; then
  if ! command -v flock >/dev/null 2>&1; then
    echo "build_speed: flock not installed" >&2
    exit 1
  fi
  export ROTLAND_GATE_LOCKED=1
  SELF="$(readlink -f "$0")"
  exec flock /tmp/rotland-heavy.lock bash "$SELF" "${ORIG_ARGS[@]}"
fi

cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/home/aymen/Documents/rotland-rust/target}"
if [ -x "$ROOT/tools/claim_target.sh" ]; then
  "$ROOT/tools/claim_target.sh"
fi

TMPDIR_PROBE="$(mktemp -d)"
ACTIVE_PROBE_FILE=""
ACTIVE_PROBE_BAK="$TMPDIR_PROBE/backup.rs"

restore_probe() {
  if [ -n "$ACTIVE_PROBE_FILE" ] && [ -f "$ACTIVE_PROBE_BAK" ]; then
    cp "$ACTIVE_PROBE_BAK" "$ACTIVE_PROBE_FILE"
    ACTIVE_PROBE_FILE=""
  fi
}

cleanup() {
  restore_probe
  rm -rf "$TMPDIR_PROBE"
}
trap cleanup EXIT INT TERM

emit_line() {
  local line="$1"
  printf '%s\n' "$line"
  if [ -n "$OUT_FILE" ]; then
    mkdir -p "$(dirname "$OUT_FILE")"
    printf '%s\n' "$line" >> "$OUT_FILE"
  fi
}

probe_file() {
  local path="$1"
  local label="$2"
  shift 2

  if [ ! -f "$path" ]; then
    echo "build_speed: missing file: $path" >&2
    exit 1
  fi

  cp "$path" "$ACTIVE_PROBE_BAK"
  ACTIVE_PROBE_FILE="$path"

  local ts
  ts="$(date +%s%N)"
  printf '\n#[allow(dead_code)]\nconst _BUILD_SPEED_PROBE: u64 = %s;\n' "$ts" >> "$path"

  local start_ns end_ns elapsed status=0
  start_ns="$(date +%s%N)"
  if "$@" >"$TMPDIR_PROBE/cargo.log" 2>&1; then
    status=0
  else
    status=$?
  fi
  end_ns="$(date +%s%N)"

  restore_probe

  if [ "$status" -ne 0 ]; then
    echo "build_speed: command failed during '$label': $*" >&2
    tail -n 40 "$TMPDIR_PROBE/cargo.log" >&2 || true
    exit "$status"
  fi

  elapsed="$(awk -v s="$start_ns" -v e="$end_ns" 'BEGIN { printf "%.2f", (e - s) / 1000000000 }')"
  emit_line "| $label | ${elapsed}s |"
}

# Warm up baseline dev build
cargo build >"$TMPDIR_PROBE/warmup.log" 2>&1

emit_line "| Case | Time |"
emit_line "| --- | --- |"

probe_file "game/src/lib.rs" "cargo build (1-line game/src/lib.rs)" cargo build
probe_file "rotland/src/main.rs" "cargo build (1-line rotland/src/main.rs)" cargo build
probe_file "engine/src/lib.rs" "cargo build (1-line engine/src/lib.rs)" cargo build
probe_file "humanoid/src/lib.rs" "cargo build (1-line humanoid/src/lib.rs)" cargo build
probe_file "game/src/lib.rs" "cargo test --no-run (1-line game/src/lib.rs)" cargo test --no-run
probe_file "game/src/lib.rs" "cargo test (1-line game/src/lib.rs)" cargo test
probe_file "game/src/lib.rs" "cargo clippy --all-targets (1-line game/src/lib.rs)" cargo clippy --all-targets

if [ "$MEASURE_RELEASE" -eq 1 ]; then
  cargo build --release >"$TMPDIR_PROBE/warmup_release.log" 2>&1
  probe_file "game/src/lib.rs" "cargo build --release (1-line game/src/lib.rs)" cargo build --release

  if grep -q '^\[profile\.play\]' "$ROOT/Cargo.toml"; then
    cargo build --profile play >"$TMPDIR_PROBE/warmup_play.log" 2>&1
    probe_file "game/src/lib.rs" "cargo build --profile play (1-line game/src/lib.rs)" cargo build --profile play
  fi
fi
