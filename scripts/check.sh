#!/usr/bin/env bash
set -e

LOCK="/tmp/rotland-heavy.lock"

exec flock "$LOCK" bash -c '
set -e
echo "==> Running cargo build..."
if ! cargo build; then
    echo "FAIL: cargo build failed"
    exit 1
fi

echo "==> Running cargo test..."
if ! cargo test; then
    echo "FAIL: cargo test failed"
    exit 1
fi

echo "==> Running cargo clippy..."
if ! cargo clippy --all-targets -- -D warnings; then
    echo "FAIL: cargo clippy failed"
    exit 1
fi

echo "OK: all checks passed"
'
