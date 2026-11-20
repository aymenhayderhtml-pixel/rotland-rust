# Rotland 0.2 plan (locked with the owner 2026-11-20)

## Last week in short (2026-11-13 to 11-20)
- Godot to Rust migration done: 165 commits, rounds R0-R10. Nearly every Godot feature runs in Rust.
- Speed: Godot ~20 FPS. Rust camp, debug build, 1920x986: **avg 106 FPS, median 8.9 ms, p99 27 ms, worst 1% 29 FPS**; 116k triangles + 88k shadow triangles, 51 + 28 draw calls (`--seconds=12 --stats`, seed 42, measured 2026-11-20).
- Builds 4 min -> ~5 s (line-tables-only debug, one test program per crate). `target/` 79 GB -> 3.5 GB.
- Waste we fix now: write-only workers guessed APIs; R10's 11 lanes gave ~35 compile errors that Opus fixed by hand (JESTER #35). Docs read at every session start cost ~30k tokens.
- The "150 GB a day" was almost surely disk writes from builds (550 MB test programs x many programs x fresh worktree builds), not internet. Already fixed by the build changes. Owner checks which monitor tab showed it; if it was Network, Claude investigates.

## Owner decisions (2026-11-20)
| Topic | Decision |
|---|---|
| 0.2 first focus | **New places.** First place: **ruined city** |
| World layout | **Big regions**: a city is large and far; travel between places is a real trip and the car matters |
| Base | **Both, camp first**: pay-to-grow camp upgrades first, player-built base later in 0.2 |
| Spare speed (~3x) | **Better-looking models** (buildings, characters, cars, weapons) |
| Gemini Neon | **Chat only** (AI Studio, no files): copy-paste flow with pack/apply scripts |
| Hard rule kept | Never under 30 FPS. Worst 1% is 29 today, so the stutter fix is mandatory |

## AI roles and token rules
Owner's quality ranking: Opus 5.5 > Gemini Neon > Haiku 5.5 > DeepSeek 4.1 > Gemini 3.8 Flash.

| AI | Budget | Use it for | Never for |
|---|---|---|---|
| Opus (orchestrator) | Scarce. Aim for no more than ~13% of the weekly limit per day | Plans, task specs, design calls, final review, wiring into `rotland/` | Fixing compile errors, reading long files, mechanical edits |
| Gemini Neon (AI Studio) | Large, chat only | Hard or visual code: UI screens, new systems, Blender scripts for hero models | Anything that needs to look around the repo |
| DeepSeek 4.1 (OpenCode) | 60M tokens a day, refreshed daily | Main repo worker: implements tasks, applies and fixes Neon output until the gate passes, reviews Flash diffs | Design decisions |
| Gemini 3.8 Flash (Antigravity) | Unlimited | Work a script or picture can prove: data JSON, tests, Blender props checked by render, doc moves, simple refactors | Unverified claims (its report is ignored; only the gate counts) |
| Haiku | Shares Opus's limit | Rare tiny lookups only | Anything DeepSeek or Flash can do |

## Workflow v2: the gate (replaces "workers never build")
1. **`tools/gate.sh`** is the only build command any worker runs. Under `flock /tmp/rotland-heavy.lock`, with the shared `CARGO_TARGET_DIR=/home/aymen/Documents/rotland-rust/target` (third-party crates built once for everyone), it runs:
   - `cargo clippy --all-targets -- -D warnings`, which also compiles the tests
   - `cargo test <filter from the task>`
   - **scope check:** files changed vs the task's `Files:` list
   - **number check:** every new number literal in non-test `game/`, `world/`, `humanoid/` code needs a source tag on its line: `// godot: <file>.cs:<line>` (the script opens the Godot file and finds the number), `// data: <file>.json <key>`, or `// new: DECISIONS <date>` for new content. Small math numbers (0, 1, 2, 0.5, -1, 1e-6 and the like) are exempt. Numbers in `data/*.json` are fine (Opus reviews data diffs).
   - **unused check:** new `pub fn`/`pub struct`/`pub const` with no use outside its own file
   - **report:** writes a "Gate report" block into the task file (files changed, test counts, PASS/FAIL). A worker's own words are not a report.
   - Output is capped at about 30 lines (the first errors only) to save tokens.
2. Workers loop until the gate says PASS, then commit once to their branch. Opus reviews only PASSED work.
3. **At most 4 workers at once** (4-core laptop). Fewer, better lanes beat 11 lanes.
4. Opus still does one final merge + build + test + clippy + screenshot pass per round.

## Neon copy-paste flow
1. `tools/neon_pack.sh tasks/<task>.md` writes one paste-ready file, `neon/out/<task>.prompt.md`: rules summary, the task, the full text of the files listed under `Read:`, the `pub` signatures of files under `API:`, data JSON, and the Godot source.
2. The owner pastes it into AI Studio. Neon must answer with blocks like `=== FILE: path ===` ... `=== END ===`.
3. The owner saves the answer as `neon/in/<task>.md`. `tools/neon_apply.py` writes the files, refusing any path not in the task's `Files:` list.
4. A DeepSeek worker runs the gate and fixes the code until it passes. `neon/` is git-ignored.

## Docs cleanup (token savings)
- New `docs/STATUS.md` (no more than ~80 lines): the only file read at session start (now, next, open questions, latest numbers). CLAUDE.md/AGENTS.md point to it instead of RUST_QUEUE + PARITY + DECISIONS.
- Move history out: RUST_QUEUE.md history, RUST_PLAN.md and the done tasks R4-R10 go to `docs/archive/` and `tasks/archive/`.
- RUST_PARITY.md keeps only the open rows (missing / logic only); done rows go to the archive.
- GEMINI.md: the JESTER lessons that the gate now catches are dropped from the prompt, because the script enforces them.

## Next week (2026-11-21 to 11-27)
| Day | Work | Who |
|---|---|---|
| 1 | Gate script, Neon pack/apply, docs cleanup + STATUS.md, update AGENTS/CLAUDE/GEMINI to workflow v2 | DeepSeek writes the scripts from Opus's spec, Opus reviews; Flash moves docs; Opus writes STATUS.md |
| 2 | **Speed budget:** `--bench` scene that adds copies of objects until the frame time reaches 16.7 ms on the laptop (the real triangle budget); find and fix the 27 ms stutter spikes; write `data/budgets.json` (triangles per model type, per frame, shadows); the baker refuses models over their cap | Opus measures and sets budgets; DeepSeek bench + stutter fix |
| 2 | Small dev panel (F1): teleport, set time, spawn zombie/item, FPS graph; `--at=x,z` flag for city screenshots | Neon (UI) -> DeepSeek applies |
| 3 | **City spec** (Opus, owner checks): region size and distance from camp, road grid, block sizes, building list, loot tiers, zombie density, night rules. Numbers go to `data/city.json` and DECISIONS | Opus |
| 3-6 | City lanes (at most 4 at once): (a) world gen: city region in the biome map + roads + blocks, pure with tests (`world/`); (b) city buildings: 6-8 detailed models within budget, with LOD (apartment, shop, gas station, police, hospital, warehouse, ruin); (c) street props + asphalt/sidewalk ground paint (lamps, wrecks, barriers, trash, the ground outside the city stays green); (d) data: city loot tables, spawning density, map/minimap roads | (a) DeepSeek, (b) Neon Blender scripts + Flash variants, checked by render, (c) Flash, (d) Flash data + DeepSeek map |
| 7 | Owner play test in a release build, fixes, commit, cleanup (only `main` remains) | Opus |

## Rest of 0.2 (after next week, order to confirm)
1. Camp upgrades (pay to grow: walls, workshop, guards, traders, look).
2. More places as big regions: lake & coast (water drawing, docks, fishing, boat later), mountains, swamp.
3. Better-looking models pass: characters, cars, weapons within the measured budget.
4. Player-built base (place walls, doors, buildings).
5. Builder tools: model viewer, automatic screenshot comparison ("ground stays green" check), data checker for `data/*.json`.
6. Bigger systems: horde flow fields, a director that paces zombie pressure, city traffic wrecks/roadblocks.
7. Open GitHub items: #5 weapons, #6 guild, #8 portrait, #17 dirty water, #19 move game logic out of `rotland/app/`.
