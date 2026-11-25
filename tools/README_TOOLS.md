# Tools

All commands are run from the repo root. Heavy commands use the shared build
folder `/home/aymen/Documents/rotland-rust/target` and `/tmp/rotland-heavy.lock`.

## Build & performance scripts

- `cargo build --profile play` — fast incremental play build (`target/play/rotland`), recommended for daily playtesting.
- `cargo build --profile dist` — full LTO shipping/benchmark build (`target/dist/rotland`).
- `tools/build_speed.sh [--release] [--out <file>]` — measures 1-line incremental build, test, clippy, and release/play build times.
- `tools/perf_scenes.sh [--bin <path>] [--seconds <n>] [--out <file>] [--scale <f>] [--set <default|biomes|all>]` — benchmarks scenes and prints avg FPS, 1% low, worst frame ms, and draw calls. `default` runs the original 5 (`camp`, `forest`, `city`, `night`, `zombies`), `biomes` runs the 4 outer biomes (`desert`, `snow`, `swamp`, `mountains`), and `all` runs all 9.

## World & biome tools

- `cargo run --release -p world --example worldmap -- --seed=42 --size=12000 --px=1200 --out=/tmp/worldmap.png` — renders a top-down biome + hillshade PNG centred on the camp (with the `core_radius` circle, tier rings, and camp dot) and prints each biome's area share to stdout.
- `--biome=<key>` (`plains`, `forest`, `desert`, `snow`, `swamp`, `mountains`) — teleports the player on startup into that biome (outer biomes: `core_radius + blend_band + 300 m` along the sector centre angle, facing outward; `plains`/`forest`: nearest core point):
  `flock /tmp/rotland-heavy.lock target/release/rotland --save-dir=/tmp/scratch --shot=/tmp/desert.png --biome=desert --frames=30`

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

* `// godot: <file>.cs:<line>` — the number is really in that Godot file.
* `// data: <file>.json <key>` — the key is really in `data/<file>.json`.
* `// new: DECISIONS <date>` — `docs/DECISIONS.md` has a row with that date.
* `// moved: <path>` — the number was moved from that file and exists in `<base>:<path>`.

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

## Animation screenshot flags (used with `--shot=<png>`)

Always pass `--save-dir=/tmp/...` so real saves stay untouched:

* `--pose=<clip>[@<seconds>]` — freezes the player on that clip at `<seconds>` (default: clip midpoint). Upper-body clips layer over idle legs; full-body clips override the whole body:
`flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/pose.png --pose=melee_2h_b@0.35`
* `--weapon=<id>` — equips a weapon in the player's hand at startup (`bat`, `axe`, `pistol`, `rifle`, `none`...):
`flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/bat.png --weapon=bat --pose=block_2h`
* `--cam-yaw=<degrees>` — orbits the third-person camera around the player (`0` = behind, `90` = right side, `180` = front):
`flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/front.png --pose=eat --cam-yaw=180`
* `--block` — holds melee block (right mouse held):
`flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/block.png --weapon=bat --block --cam-yaw=90`
* `--sprint` (with `--autowalk`) — sprints forward:
`flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/sprint.png --autowalk --sprint --cam-yaw=90`

## Downed, revive & rescue screenshot flags (used with `--shot=<png>`)

* `--down-at=<seconds>` — deals a lethal hit through the zombie-bite downed path at `<seconds>` so the player enters the knocked-down crawl state:
`flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/down.png --down-at=1 --frames=120 --cam-yaw=90`
* `--team=<n>` — adds `<n>` free rescued-style teammates (`Follow` order) next to the player at startup (with `--down-at`, one walks over and revives):
`flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/revive.png --team=1 --down-at=1 --frames=180 --cam-yaw=90`
* `--tp-rescue` — teleports the player `6 m` from the nearest unrescued survivor spot, facing it, so the survivor and `[E] Rescue` prompt are visible:
`flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/rescue.png --tp-rescue --frames=30`
* `--hold-e` — holds the interact key (`E`) the whole run (e.g. for self-revive with `--give=bandage`):
`flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/self_revive.png --give=bandage --down-at=1 --hold-e --frames=180 --cam-yaw=90`
