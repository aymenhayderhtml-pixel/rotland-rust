# Tools

All commands are run from the repo root. Heavy commands use the shared build
folder `/home/aymen/Documents/rotland-rust/target` and `/tmp/rotland-heavy.lock`.

## `tools/gate.sh` — the only build command a worker runs

```bash
tools/gate.sh                                  # full check + whole test suite
tools/gate.sh --task tasks/R11-01-....md       # also check scope / numbers / unused
tools/gate.sh --test minimap                   # only tests whose name matches
```

It runs clippy (fails on any warning), then the tests, then the task checks
(only files the task allows, every new game number has a source tag, no unused
public code). It stops at the first failing step and the last line is exactly
`GATE: PASS` or `GATE: FAIL (<step>)`. The test filter comes from `--test`,
else from a `Test:` line in the task file, else the whole suite. With `--task`
it also rewrites the `## Gate report` section at the end of that task file —
never edit it by hand.

## `tools/gate_checks.py` — the three checks on their own

```bash
python3 -I tools/gate_checks.py --base main --task tasks/R11-01-....md
python3 -I tools/gate_checks.py --selftest     # quick self-test, no repo needed
```

Needs a source tag on the same line (or the line above) for each new number in
`game/src`, `world/src`, `humanoid/src`:

- `// godot: <file>.cs:<line>` — the number is really in that Godot file.
- `// data: <file>.json <key>` — the key is really in `data/<file>.json`.
- `// new: DECISIONS <date>` — `docs/DECISIONS.md` has a row with that date.

## `tools/neon_pack.py` — make one file to paste into Gemini Neon

```bash
python3 -I tools/neon_pack.py tasks/R11-01-....md
```

Writes `neon/out/<task>.prompt.md` (about 2000 tokens per 8 KB). It reads the
task's `Read:`, `API:`, `Data:`, `Godot:` and `Files:` lines: full files, just
the public signatures, data JSON, and the old Godot C# source. Paste the whole
output into AI Studio.

## `tools/neon_apply.py` — write Neon's answer into the repo

```bash
# save Neon's answer as neon/in/<task>.md first
python3 -I tools/neon_apply.py neon/in/R11-01-....md --task tasks/R11-01-....md --dry-run
python3 -I tools/neon_apply.py neon/in/R11-01-....md --task tasks/R11-01-....md
```

It only writes `=== FILE: path === ... === END ===` blocks whose path is in the
task's `Files:` list; anything absolute, with `..`, or outside that list is
refused and nothing is written. The `=== NOTES ===` text goes under
`## Neon notes`. `neon/` is git-ignored; after applying run
`tools/gate.sh --task <task file>` and fix until it passes.
