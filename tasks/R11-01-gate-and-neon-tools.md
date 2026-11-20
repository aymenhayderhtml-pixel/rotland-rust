# R11-01: gate script + Neon pack/apply scripts

Read first: `AGENTS.md`, then `docs/PLAN_0.2.md` sections "Workflow v2" and "Neon copy-paste flow".
Exception for this task: you MAY run your scripts and cargo commands to test them, always under `flock /tmp/rotland-heavy.lock`.

Files: tools/gate.sh, tools/gate_checks.py, tools/neon_pack.py, tools/neon_apply.py, tools/README_TOOLS.md, .gitignore, tasks/R11-01-gate-and-neon-tools.md

Python: stdlib only, run with `python3 -I`. Bash: `set -euo pipefail`. Keep each file under ~300 lines.

## 1. `tools/gate.sh [--task tasks/X.md] [--base main] [--test <filter>]`
Runs from any git worktree of this repo. Steps, stop at first failing step, print at most ~30 lines total:
1. `export CARGO_TARGET_DIR=/home/aymen/Documents/rotland-rust/target` unless already set.
2. Everything heavy under `flock /tmp/rotland-heavy.lock`.
3. `cargo clippy --all-targets --message-format=short -- -D warnings`: on fail print only the first 12 `error`/`warning` lines (file:line: message), plus a count.
4. `cargo test <filter>` (filter from `--test`, else from a `Test:` line in the task file, else none = whole workspace). On fail print failing test names + first 15 lines of the first panic. On pass print the total "N passed" summed over all `test result:` lines. If total is 0 when a filter was given, FAIL ("filter matched no tests").
5. `python3 -I tools/gate_checks.py --base <base> [--task X]` (section 2).
6. Last line exactly `GATE: PASS` or `GATE: FAIL (<step>)`. Exit code 0/1.
7. With `--task`, replace (or append) a `## Gate report` section at the end of the task file: date, base, files changed (`git diff --name-only <base>...HEAD` plus uncommitted), test count, each step PASS/FAIL. Only the script writes this section.

## 2. `tools/gate_checks.py --base main [--task tasks/X.md]`
Works on the diff `git diff <base>` (committed + uncommitted vs merge-base). Checks:
- **scope**: if the task has a `Files:` line (comma separated, may contain globs like `world/src/city/*`), every changed path must match one; the task file itself is always allowed. Print offending paths.
- **numbers**: for added lines (`+`) in `*.rs` under `game/src/`, `world/src/`, `humanoid/src/` (skip files/dirs named `tests`, `*_tests.rs`, and lines after a `#[cfg(test)]` in the same file), find numeric literals (ints and floats, with optional `_f32` etc. suffix; ignore inside strings and `//` comments). Exempt values: 0, 1, 2, 3, 4, 0.5, 0.25, 10, 100, 1000, 0.001, 1e-3, 1e-4, 1e-5, 1e-6, 180, 360, and array/tuple index usage `[n]` / `.n`. Any other number needs a tag on the same line (or the line directly above):
  - `// godot: <path>.cs:<line>`: open `/home/aymen/Documents/zombi servival/<path>` (also try `scripts/<path>`); FAIL if file missing or the number (compare as float, also accept `<n>f`) does not appear within ±3 lines of `<line>`.
  - `// data: <file>.json <key>`: FAIL if `data/<file>.json` missing or the key string is not in it.
  - `// new: DECISIONS <date>`: FAIL if `docs/DECISIONS.md` has no row with that date.
  Print up to 10 offenders as `path:line  number  reason`.
- **unused**: for each added `pub fn NAME`, `pub struct NAME`, `pub enum NAME`, `pub const NAME`, `pub trait NAME` in `*.rs`, `git grep -w NAME` across `*.rs`; FAIL if it appears only in its own file (excluding the definition line) AND nowhere else. (Same-file use is fine; then it's not "pub-needed" but we don't fail that.) Actually rule: FAIL only if the name has zero uses anywhere except its definition line.
- Each check prints `scope: PASS|FAIL`, `numbers: ...`, `unused: ...`. Exit 1 if any fails.
- Add a self-test: `python3 -I tools/gate_checks.py --selftest` runs a few built-in cases for the number parser and tag checker (no repo needed) and prints `selftest: N ok`.

## 3. `tools/neon_pack.py tasks/X.md [--out neon/out/X.prompt.md]`
Builds one paste-ready markdown file for Gemini Neon (chat only, no file access, ~500k context; warn on stderr if the output is over ~400k characters... measure: print size in chars and rough tokens = chars/4).
Reads these optional lines from the task file (comma separated paths/globs, relative to repo; `godot:` paths relative to `/home/aymen/Documents/zombi servival`):
- `Read:` full text of each file, in a fenced block with the path as heading.
- `API:` only signatures: lines matching `pub fn|pub struct|pub enum|pub trait|pub const|pub type|impl ` plus `pub` fields inside structs, with line numbers.
- `Data:` full JSON files.
- `Godot:` full C# files from the Godot repo.
- `Files:` the files Neon may write.
Output order: (1) a fixed rules header (write in the file): Rust 2024, glam 0.34, egui 0.37, no invented APIs (only use what is shown under API/Read), no unused code, numbers from Data/Godot with the source-tag comment format from section 2, Godot yaw convention forward = (-sin yaw, 0, -cos yaw), clippy-clean (let-chains, no hand-written 0.7071/3.14), tests for logic; (2) the whole task file; (3) Files list; (4) Read, API, Data, Godot sections; (5) the required ANSWER FORMAT: for each file, a line `=== FILE: <path> ===`, the complete file content (no ``` fences needed, but tolerate them), then `=== END ===`; files must be complete, never diffs; after all files, a `=== NOTES ===` block with wiring notes.

## 4. `tools/neon_apply.py neon/in/X.md --task tasks/X.md [--dry-run]`
Parses the `=== FILE: path === ... === END ===` blocks (strip a single surrounding ``` fence line if present), refuses any path not matching the task's `Files:` list or containing `..` or absolute paths, writes files (creating dirs), prints each path + line count, and appends the `=== NOTES ===` text under `## Neon notes` in the task file. `--dry-run` only prints. Exit 1 on any refused path (write nothing in that case).

## 5. Other
- `.gitignore`: add `neon/`.
- `tools/README_TOOLS.md`: short usage for all four tools (under 60 lines, plain English).
- Test your work: run `tools/gate_checks.py --selftest`; run `tools/gate.sh --test minimap` on current main (must end `GATE: PASS`); make a tiny fake task file in /tmp with `Files:`/`Read:`/`API:` and run `neon_pack.py` and `neon_apply.py --dry-run` on a hand-written sample answer. Do not leave test files in the repo.

## Worker notes
(files changed, how you tested with real output, known issues)
