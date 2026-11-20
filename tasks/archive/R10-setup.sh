#!/usr/bin/env bash
# Makes one separate work folder (git worktree) per R10 lane, next to this repo.
# Usage:  bash tasks/R10-setup.sh A     -> Phase A lanes (01-08, 11)
#         bash tasks/R10-setup.sh B     -> Phase B lanes (09, 10), only after Claude merged R10-01
set -euo pipefail
cd "$(dirname "$0")/.."
phase="${1:-A}"
if [ "$phase" = "A" ]; then lanes="01 02 03 04 05 06 07 08 11"; else lanes="09 10"; fi
for n in $lanes; do
  dir="../rotland-r10-$n"
  if [ -d "$dir" ]; then echo "exists: $dir"; continue; fi
  git worktree add "$dir" -b "r10-$n" main
  echo "made:   $dir  (branch r10-$n)"
done
