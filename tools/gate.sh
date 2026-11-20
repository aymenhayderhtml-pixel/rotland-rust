#!/usr/bin/env bash
# Rotland gate: the only build command workers run.
# clippy (-D warnings) -> cargo test -> task scope / number / unused checks.
# Heavy work runs under flock with the shared CARGO_TARGET_DIR.
#
# usage: tools/gate.sh [--task tasks/X.md] [--base main] [--test <filter>]
set -euo pipefail

TASK=""
BASE="main"
FILTER=""
ORIG_ARGS=("$@")

while [ $# -gt 0 ]; do
  case "$1" in
    --task)   TASK="${2:-}"; shift 2 ;;
    --base)   BASE="${2:-}"; shift 2 ;;
    --test)   FILTER="${2:-}"; shift 2 ;;
    -h|--help)
      echo "usage: tools/gate.sh [--task tasks/X.md] [--base main] [--test <filter>]"
      exit 0 ;;
    *) echo "gate: unknown argument: $1" >&2; exit 2 ;;
  esac
done

if [ -n "$TASK" ]; then
  case "$TASK" in /*) ;; *) TASK="$PWD/$TASK" ;; esac
fi

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$ROOT" ]; then
  echo "GATE: FAIL (not a git worktree)"
  exit 1
fi

# One lock for every heavy command.
if [ "${ROTLAND_GATE_LOCKED:-}" != "1" ]; then
  if ! command -v flock >/dev/null 2>&1; then
    echo "GATE: FAIL (flock not installed)"
    exit 1
  fi
  export ROTLAND_GATE_LOCKED=1
  SELF="$(readlink -f "$0")"
  exec flock /tmp/rotland-heavy.lock bash "$SELF" "${ORIG_ARGS[@]}"
fi

cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/home/aymen/Documents/rotland-rust/target}"

# Test filter: --test wins, then a `Test:` line in the task file, else whole suite.
if [ -z "$FILTER" ] && [ -n "$TASK" ] && [ -f "$TASK" ]; then
  FILTER="$(sed -n -E 's/^[[:space:]]*[Tt]est:[[:space:]]*(.*)$/\1/p' "$TASK" \
    | head -1 | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
fi

LOGDIR="$(mktemp -d)"
trap 'rm -rf "$LOGDIR"' EXIT

DATE="$(date +%Y-%m-%d)"
FAILED=""
CLIPPY_ST=SKIP
TEST_ST=SKIP
CHECKS_ST=SKIP
TESTS=0

# ---------------------------------------------------------------- clippy
if cargo clippy --all-targets --message-format=short -- -D warnings \
     >"$LOGDIR/clippy.log" 2>&1; then
  CLIPPY_ST=PASS
  echo "clippy: PASS"
else
  CLIPPY_ST=FAIL
  N="$(grep -cE '^[^ ]+:[0-9]+:[0-9]+: (error|warning)' "$LOGDIR/clippy.log" || true)"
  echo "clippy: FAIL ($N error/warning lines)"
  if [ "$N" -gt 0 ]; then
    grep -E '^[^ ]+:[0-9]+:[0-9]+: (error|warning)' "$LOGDIR/clippy.log" | head -12 || true
  else
    head -8 "$LOGDIR/clippy.log" || true
  fi
  FAILED=clippy
fi

# ---------------------------------------------------------------- tests
if [ "$CLIPPY_ST" = PASS ]; then
  if [ -n "$FILTER" ]; then
    cargo test "$FILTER" >"$LOGDIR/test.log" 2>&1 && TEST_OK=1 || TEST_OK=0
  else
    cargo test >"$LOGDIR/test.log" 2>&1 && TEST_OK=1 || TEST_OK=0
  fi
  TESTS="$(grep -E '^test result:' "$LOGDIR/test.log" \
    | sed -E 's/.* ([0-9]+) passed.*/\1/' | awk '{s+=$1} END {print s+0}')"
  NOMATCH=0
  if [ "$TEST_OK" = 1 ] && [ -n "$FILTER" ] && [ "$TESTS" -eq 0 ]; then
    TEST_OK=0
    NOMATCH=1
    echo "test: FAIL (filter matched no tests: $FILTER)"
  fi
  if [ "$TEST_OK" = 1 ]; then
    TEST_ST=PASS
    echo "test: PASS ($TESTS passed${FILTER:+, filter $FILTER})"
  else
    TEST_ST=FAIL
    if [ "$NOMATCH" = 0 ]; then
      grep -E '^test .+ \.\.\. FAILED$' "$LOGDIR/test.log" \
        | sed -E 's/^test (.+) \.\.\. FAILED$/\1/' | head -8 | sed 's/^/fail: /' || true
      if grep -q '^---- ' "$LOGDIR/test.log"; then
        awk '/^---- .* stdout ----/{f=1} f{c++; if(c<=10) print; else exit}' "$LOGDIR/test.log"
      else
        head -8 "$LOGDIR/test.log" || true
      fi
    fi
    FAILED=test
  fi
fi

# ---------------------------------------------------------------- checks
if [ "$TEST_ST" = PASS ]; then
  if [ -n "$TASK" ]; then
    python3 -I tools/gate_checks.py --base "$BASE" --task "$TASK" >"$LOGDIR/checks.log" 2>&1 \
      && CHECKS_ST=PASS || CHECKS_ST=FAIL
  else
    python3 -I tools/gate_checks.py --base "$BASE" >"$LOGDIR/checks.log" 2>&1 \
      && CHECKS_ST=PASS || CHECKS_ST=FAIL
  fi
  if [ "$CHECKS_ST" = PASS ]; then
    cat "$LOGDIR/checks.log"
  else
    head -12 "$LOGDIR/checks.log" || true
    FAILED=checks
  fi
fi

# ---------------------------------------------------------------- report
CHANGES="$(
  { git diff --name-only "$BASE...HEAD" 2>/dev/null || true
    git diff --name-only 2>/dev/null || true
    git ls-files --others --exclude-standard 2>/dev/null || true
  } | sort -u
)"

if [ -z "$FAILED" ]; then
  FINAL="GATE: PASS"
  GATE_ST=PASS
  RC=0
else
  FINAL="GATE: FAIL ($FAILED)"
  GATE_ST="FAIL ($FAILED)"
  RC=1
fi

if [ -n "$TASK" ] && [ -f "$TASK" ]; then
  python3 -I - "$TASK" "$GATE_ST" "$DATE" "$BASE" "$TESTS" \
    "$CLIPPY_ST" "$TEST_ST" "$CHECKS_ST" "$CHANGES" <<'PY'
import sys

path, gate, date, base, tests, clippy, test, checks, changes = sys.argv[1:10]
with open(path, encoding="utf-8") as f:
    lines = f.read().splitlines()

keep = []
for line in lines:
    if line.strip() == "## Gate report":
        break
    keep.append(line)
while keep and not keep[-1].strip():
    keep.pop()

files = [c for c in changes.splitlines() if c.strip()]
out = keep + [
    "",
    "## Gate report",
    f"- date: {date}",
    f"- base: {base}",
    f"- files changed ({len(files)}): " + (", ".join(files) if files else "none"),
    f"- tests passed: {tests}",
    f"- clippy: {clippy}",
    f"- test: {test}",
    f"- checks: {checks}",
    f"- gate: {gate}",
    "",
]
with open(path, "w", encoding="utf-8") as f:
    f.write("\n".join(out))
PY
fi

echo "$FINAL"
exit "$RC"
