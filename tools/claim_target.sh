#!/usr/bin/env bash
# Run INSIDE the heavy lock, from a checkout, before any cargo build that uses the shared target dir.
# Cargo names our own crates' build outputs the same in every git worktree, so a build in one
# worktree can silently reuse another worktree's compiled engine/game/world code. When the last
# checkout that built here was a different one, drop our own crates' outputs (third-party crates stay).
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
TARGET="${CARGO_TARGET_DIR:-/home/aymen/Documents/rotland-rust/target}"
MARK="$TARGET/.last-build-root"
if [ "$(cat "$MARK" 2>/dev/null || true)" != "$ROOT" ]; then
  (cd "$ROOT" && cargo clean -q -p engine -p rotland -p pack -p baker -p world -p physics -p game \
     -p humanoid -p paths -p save -p audio --target-dir "$TARGET" 2>/dev/null || true)
  (cd "$ROOT" && cargo clean -q --release -p engine -p rotland -p pack -p baker -p world -p physics -p game \
     -p humanoid -p paths -p save -p audio --target-dir "$TARGET" 2>/dev/null || true)
  mkdir -p "$TARGET"
  echo "$ROOT" > "$MARK"
fi
