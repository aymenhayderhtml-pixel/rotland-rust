# STATUS (read this first; keep it under ~80 lines)

Updated: 2026-11-21 by Claude (after R13).

## Where we are
- Version 0.1 = Godot parity in Rust, done (R0-R10). History: `docs/archive/RUST_QUEUE.md`.
- Working on **0.2**. This week: `docs/PLAN_0.2_PUSH.md` (6-day push). Earlier plan: `docs/PLAN_0.2.md`. Locked choices: `docs/DECISIONS.md`.
- Open Godot gaps (short list): `docs/PARITY_OPEN.md`. Worker mistakes: `docs/JESTER.md`.

## Latest numbers (release, plugged in, 2026-11-21)
Camp: avg 114 FPS, worst 1% 68. Ashford core (`--at=-384,1792`, seed 42, 18 zombies): avg 90, p99 15 ms, worst 1% 49-55; 211k tris (budget 228k), 159k shadow tris (over the 121k budget, FPS still fine), 56 draw calls.
60 FPS triangle limit (bench): 380k-700k (`data/budgets.json`, `docs/BUDGETS.md`).

## Now: 0.2 push (docs/PLAN_0.2_PUSH.md), day 1 done 2026-11-21 (owner out of tokens; resume when told "continue")
- MERGED in main: R16 foundations (camp upgrades core, build-mode core, weapons + projectiles, hordes + director + special zombie data, lake layout, water renderer, LOD fade, big map shows Ashford, HUD polish), R16-N1 Neon menus, R17 models (camp walls/gates, towers/workshop/doctor/mechanic, weapons + icons, brute/screamer/crawler, base parts + furniture, lake buildings/props), lake in the world, HUD wiring. 648 tests pass. 178 models baked.
- NOT merged: branch `neon/camp-board` (Neon camp board screen, needs wiring: R18-01 merges it).
- R18-01 (camp board wiring) STALLED uncommitted in its worktree (it ended its turn while its gate ran): on resume run its gate there, commit, merge. Finished on branches: R18-02, 03, 04, 05, 06, 07, 08, 09 (all t3/* branches) (03 build mode keeps state in globals: move it into app state + save).
- RUNNING when the session ended: R18-01..09 (camp board wiring, camp layout, build mode, weapons in play, hordes in play, lake gameplay, crawler fix, LOD fade cost, guild core). On resume: read each `tasks/R18-*.md` Worker notes + gate report, review, merge, apply their "Wiring notes for Claude", full gate, release build, screenshots, FPS quiet test, push, clean worktrees.
- Known: water drawn as 8 m strips over the lake only (beach now dry), BUT docks sit at ground height under the water (dock y must be water_level + ~0.4, in `world/src/lake/place.rs`), the shore edge is blocky (8 m strips), and a thin white line shows across the screen at `--at=470,-1420` (probably a water strip edge seen from far; check); crawler pose wrong (R18-07); FPS worries: city core worst 1% 21, lake 17 under heavy load (R18-08 + a quiet re-measure first thing).
- Tools fixed today: gate scope reads `Files:` from main; `tools/claim_target.sh` rebuilds our crates every gate (shared target mixed worktrees); gate never deadlocks inside an outer flock.
- Seed 42 places: city core (-384, 1792), lake first dock (528, -1478). Dev panel: City core / City entry / Camp road / Lake teleports.

## Next (from PLAN_0.2.md)
1. Stutter: re-measure on a quiet laptop (nothing else running), then find the between-frames stall (present mode / vsync, compositor, physics collider rebuild in `physics/src/world.rs`). Re-run `--bench` quietly too.
2. Day 6 DONE: owner checked Ashford (arrow-key look, steady dev panel, city teleports, lamps/bus stops facing, building LOD1 at 70 m). Owner sees no more issues. Optional: dithered LOD fade if the 70 m switch still shows. Day 7: owner play test + stutter recording (`--spikes=/tmp/spikes.txt`, camp + city spots).

## Workers (who does what: PLAN_0.2.md "AI roles")
- DeepSeek: `opencode` / `opencode/deepseek-v4.1-flash-free`: free usage ran out on 2026-11-20.
- Gemini 3.8 Flash: `antigravity` / `gemini-3.8-flash`: SHORT jobs only; it can't wait on long builds or runs (stops with "I'll wait").
- Haiku (Agent tool): long build/run jobs; build+run inside ONE flock (shared release binary gets overwritten).
- Neon: chat only. A subagent builds `neon/out/<task>.prompt.md` with `tools/neon_pack.py`, the owner pastes it into AI Studio and saves the answer as `neon/in/<task>.md`, then a worker runs `tools/neon_apply.py` + the gate.
- Workers run `tools/gate.sh --task <file>` and loop until `GATE: PASS` (usage: `tools/README_TOOLS.md`).

## Open questions for the owner
- Final game name.

