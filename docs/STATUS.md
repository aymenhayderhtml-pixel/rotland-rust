# STATUS (read this first; keep it under ~80 lines)

Updated: 2026-11-21 by Claude (after R13).

## Where we are
- Version 0.1 = Godot parity in Rust, done (R0-R10). History: `docs/archive/RUST_QUEUE.md`.
- Working on **0.2**. Plan and owner decisions: `docs/PLAN_0.2.md`. Locked choices: `docs/DECISIONS.md`.
- Open Godot gaps (short list): `docs/PARITY_OPEN.md`. Worker mistakes: `docs/JESTER.md`.

## Latest numbers (release, camp, plugged in, 2026-11-20; noisy, other jobs were running)
avg 95-126 FPS, p99 16-29 ms, worst 1% 26-36 FPS (stutters still below the 30 minimum).
60 FPS triangle limit (bench): 380k-700k; frame budget 228k tris / 121k shadow / 120 draws (`data/budgets.json`, `docs/BUDGETS.md`).

## Last round: R13 (0.2 week 1, day 4) DONE, all 8 lanes by Gemini 3.8 Flash, no stalls
- R13-01 `world/src/city/` + `data/city.json`: placement 1.5 km from camp (least forest of 8 directions), 10x10 blocks, districts, streets with 1-in-4 blocked (always drivable), lots facing streets. Pure, 9 tests. NOT wired into the game yet.
- R13-02..05: 8 buildings + `_lod1` (apartment, office, shop, pharmacy, police, hospital, warehouse, ruin). Not yet in `build_buildings.py` or baked.
- R13-06/07: 12 street props (lamp, bus stop, mailbox, dumpster, trash bags, barricade, rubble, 2 car wrecks, fallen sign, broken fence, jersey barrier). Not yet in the build list or baked.
- R13-08: 8 city loot tables in `data/loot_tables.json`.
- What worked: Blender lanes ran only short jobs + the fast scope check; Blender serialized by `/tmp/rotland-blender.lock`.

## Next (from PLAN_0.2.md)
1. Stutter: re-measure on a quiet laptop (nothing else running), then find the between-frames stall (present mode / vsync, compositor, physics collider rebuild in `physics/src/world.rs`). Re-run `--bench` quietly too.
2. Day 5: wire the city: add models to the build lists and bake (check `--report-budgets`); world gen places city lots/props and flattens ground; asphalt/sidewalk ground paint; the road from camp; city spawn density (2x) + loot table per building; minimap streets. Then `--at=X,Z` screenshot at the core + `--stats` FPS check (worst 1% must stay >= 30). Day 6: fixes/variants. Day 7: owner play test.

## Workers (who does what: PLAN_0.2.md "AI roles")
- DeepSeek: `opencode` / `opencode/deepseek-v4.1-flash-free`: free usage ran out on 2026-11-20.
- Gemini 3.8 Flash: `antigravity` / `gemini-3.8-flash`: SHORT jobs only; it can't wait on long builds or runs (stops with "I'll wait").
- Haiku (Agent tool): long build/run jobs; build+run inside ONE flock (shared release binary gets overwritten).
- Neon: chat only. A subagent builds `neon/out/<task>.prompt.md` with `tools/neon_pack.py`, the owner pastes it into AI Studio and saves the answer as `neon/in/<task>.md`, then a worker runs `tools/neon_apply.py` + the gate.
- Workers run `tools/gate.sh --task <file>` and loop until `GATE: PASS` (usage: `tools/README_TOOLS.md`).

## Open questions for the owner
- Final game name.

