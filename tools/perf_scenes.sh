#!/usr/bin/env bash
# Run benchmark scenes in release mode and print/append a markdown FPS summary table.
#
# Usage: tools/perf_scenes.sh [--bin <path>] [--seconds <n>] [--out <file>] [--scale <f>] [--set <default|biomes|all>]
set -euo pipefail

BIN="${CARGO_TARGET_DIR:-/home/aymen/Documents/rotland-rust/target}/release/rotland"
RUN_SECONDS=30
OUT_FILE=""
SCALE=""
SET="default"

usage() {
  echo "usage: tools/perf_scenes.sh [--bin <path>] [--seconds <n>] [--out <file>] [--scale <f>] [--set <default|biomes|all>]"
}

while [ $# -gt 0 ]; do
  case "$1" in
    --bin)
      if [ -z "${2:-}" ]; then
        echo "perf_scenes: --bin requires a path" >&2
        exit 2
      fi
      BIN="$2"
      shift 2
      ;;
    --seconds)
      if [ -z "${2:-}" ]; then
        echo "perf_scenes: --seconds requires a number" >&2
        exit 2
      fi
      RUN_SECONDS="$2"
      shift 2
      ;;
    --out)
      if [ -z "${2:-}" ]; then
        echo "perf_scenes: --out requires a file path" >&2
        exit 2
      fi
      OUT_FILE="$2"
      shift 2
      ;;
    --scale)
      if [ -z "${2:-}" ]; then
        echo "perf_scenes: --scale requires a number" >&2
        exit 2
      fi
      SCALE="$2"
      shift 2
      ;;
    --set)
      if [ -z "${2:-}" ]; then
        echo "perf_scenes: --set requires default, biomes, or all" >&2
        exit 2
      fi
      SET="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "perf_scenes: unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

case "$SET" in
  default) SCENES=(camp forest city night zombies) ;;
  biomes)  SCENES=(desert snow swamp mountains) ;;
  all)     SCENES=(camp forest city night zombies desert snow swamp mountains town_desert town_snow metro) ;;
  *)
    echo "perf_scenes: unknown --set value: $SET (expected default, biomes, or all)" >&2
    exit 2
    ;;
esac

if [ ! -x "$BIN" ]; then
  echo "perf_scenes: binary not found or not executable: $BIN" >&2
  exit 1
fi

TMPDIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMPDIR"
}
trap cleanup EXIT INT TERM

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

run_locked() {
  if [ "${ROTLAND_GATE_LOCKED:-}" = "1" ] || lock_held_by_ancestor; then
    "$@"
  else
    flock /tmp/rotland-heavy.lock "$@"
  fi
}

emit_line() {
  local line="$1"
  printf '%s\n' "$line"
  if [ -n "$OUT_FILE" ]; then
    mkdir -p "$(dirname "$OUT_FILE")"
    printf '%s\n' "$line" >> "$OUT_FILE"
  fi
}

COMMON_ARGS=(
  --save-dir=/tmp/r30.5-saves
  --warmup=2
  --seconds="$RUN_SECONDS"
)
if [ -n "$SCALE" ]; then
  COMMON_ARGS+=(--scale="$SCALE")
fi

run_scene() {
  local scene="$1"
  local stats_file="$TMPDIR/$scene.json"
  local spikes_file="$TMPDIR/$scene.spikes.txt"
  local -a scene_args=()

  case "$scene" in
    camp)      scene_args=(--hour=12) ;;
    forest)    scene_args=(--scene=forest --autowalk --hour=12) ;;
    city)      scene_args=(--at=-384,1792 --autowalk --hour=12) ;;
    night)     scene_args=(--at=-384,1792 --hour=23 --flashlight) ;;
    zombies)   scene_args=(--zombies=50 --autowalk --hour=12) ;;
    desert)    scene_args=(--biome=desert --autowalk --hour=12) ;;
    snow)      scene_args=(--biome=snow --autowalk --hour=12) ;;
    swamp)       scene_args=(--biome=swamp --autowalk --hour=12) ;;
    mountains)   scene_args=(--biome=mountains --autowalk --hour=12) ;;
    town_desert) scene_args=(--town=dusthollow --autowalk --hour=12) ;;
    town_snow)   scene_args=(--town=frostmere --autowalk --hour=12) ;;
    metro)       scene_args=(--metro=core --autowalk --hour=12) ;;
    *)
      echo "perf_scenes: unknown scene: $scene" >&2
      exit 1
      ;;
  esac

  run_locked "$BIN" \
    "${COMMON_ARGS[@]}" \
    "${scene_args[@]}" \
    --stats="$stats_file" \
    --spikes="$spikes_file" \
    >/dev/null
}

format_row() {
  local scene="$1"
  local stats_file="$TMPDIR/$scene.json"
  local spikes_file="$TMPDIR/$scene.spikes.txt"

  python3 - "$scene" "$stats_file" "$spikes_file" <<'PY'
import json
import re
import sys

scene, stats_path, spikes_path = sys.argv[1:4]
with open(stats_path, "r", encoding="utf-8") as f:
    stats = json.load(f)

avg_fps = float(stats.get("avgFps", 0.0))
low1_fps = float(stats.get("worst1PercentFps", 0.0))
p95_ms = float(stats.get("p95FrameMs", 0.0))
p99_ms = float(stats.get("p99FrameMs", 0.0))
_present_mode = stats.get("presentMode", "")

game = stats.get("game") or {}
draws = int(game.get("drawCalls", 0))
shadow_draws = int(game.get("shadowDrawCalls", 0))
_triangles = int(game.get("triangles", 0))

worst_ms = None
try:
    with open(spikes_path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            val = None
            if line.startswith("{"):
                try:
                    rec = json.loads(line)
                    if "total_ms" in rec:
                        val = float(rec["total_ms"])
                    elif "total" in rec:
                        val = float(rec["total"])
                except Exception:
                    val = None
            if val is None:
                m = re.search(r"total=([0-9]+(?:\.[0-9]+)?)", line)
                if m:
                    val = float(m.group(1))
            if val is not None:
                worst_ms = val if worst_ms is None else max(worst_ms, val)
except FileNotFoundError:
    pass

if worst_ms is None:
    worst_ms = p99_ms

print(f"| {scene} | {avg_fps:.1f} | {low1_fps:.1f} | {worst_ms:.1f} | {p95_ms:.1f} | {draws} ({shadow_draws}) |")
PY
}

emit_line "| Scene | Avg FPS | 1% Low FPS | Worst Frame (ms) | p95 (ms) | Draws (Shadow) |"
emit_line "|---|---|---|---|---|---|"

for scene in "${SCENES[@]}"; do
  run_scene "$scene"
  row="$(format_row "$scene")"
  emit_line "$row"
done
