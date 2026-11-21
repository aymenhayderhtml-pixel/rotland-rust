# STATUS (read this first; keep it under ~80 lines)

Updated: 2026-11-21 by Claude (after R13).

## Where we are
- Version 0.1 = Godot parity in Rust, done (R0-R10). History: `docs/archive/RUST_QUEUE.md`.
- Working on **0.2**. Plan and owner decisions: `docs/PLAN_0.2.md`. Locked choices: `docs/DECISIONS.md`.
- Open Godot gaps (short list): `docs/PARITY_OPEN.md`. Worker mistakes: `docs/JESTER.md`.

## Latest numbers (release, plugged in, 2026-11-21)
Camp: avg 114 FPS, worst 1% 68. Ashford core (`--at=-384,1792`, seed 42, 18 zombies): avg 90, p99 15 ms, worst 1% 49-55; 211k tris (budget 228k), 159k shadow tris (over the 121k budget, FPS still fine), 56 draw calls.
60 FPS triangle limit (bench): 380k-700k (`data/budgets.json`, `docs/BUDGETS.md`).

## Last round: R14 (day 5) DONE: Ashford is in the game (4 Gemini lanes + Claude prep)
- Claude: city attached to `WorldGenerator::from_data_dir` (`city()`), straight camp road, max 4 lots per block (was ~11; 380 lots), models in build lists, baked (130 models).
- R14-01 city buildings as POIs (poi.json, chancePerChunk 0) + street props (`world/src/city/place.rs`); R14-02 flat city ground + asphalt/sidewalk/lot paint + camp road (`city/ground.rs`, `data/city_ground.json`); R14-03 2x zombies, cap 32 by day, 60% runners in the core at night; R14-04 streets on the minimap.
- Gate fix: scope check reads `Files:` from the merge-base task (JESTER 43).
- Seen, not fixed: lot ground looks mostly green (paint blend weak); sidewalk edges zigzag (2 m ground cells); one `Aborted` on exit in 4 runs (intermittent, unknown).
- R13 (day 4): 8 buildings + LOD, 12 street props, city layout, city loot tables.

## Next (from PLAN_0.2.md)
1. Stutter: re-measure on a quiet laptop (nothing else running), then find the between-frames stall (present mode / vsync, compositor, physics collider rebuild in `physics/src/world.rs`). Re-run `--bench` quietly too.
2. Day 6: owner looks at Ashford; fixes from that (lot ground color, sidewalk look, building color variants). Day 7: owner play test in a release build.

## Workers (who does what: PLAN_0.2.md "AI roles")
- DeepSeek: `opencode` / `opencode/deepseek-v4.1-flash-free`: free usage ran out on 2026-11-20.
- Gemini 3.8 Flash: `antigravity` / `gemini-3.8-flash`: SHORT jobs only; it can't wait on long builds or runs (stops with "I'll wait").
- Haiku (Agent tool): long build/run jobs; build+run inside ONE flock (shared release binary gets overwritten).
- Neon: chat only. A subagent builds `neon/out/<task>.prompt.md` with `tools/neon_pack.py`, the owner pastes it into AI Studio and saves the answer as `neon/in/<task>.md`, then a worker runs `tools/neon_apply.py` + the gate.
- Workers run `tools/gate.sh --task <file>` and loop until `GATE: PASS` (usage: `tools/README_TOOLS.md`).

## Open questions for the owner
- Final game name.

