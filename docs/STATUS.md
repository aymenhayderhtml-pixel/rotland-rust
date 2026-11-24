# STATUS (read this first; keep it under ~80 lines)

Updated: 2026-11-24 by Claude (after R27).

## Where we are
- Version 0.1 = Godot parity in Rust, done (R0-R10). History: `docs/archive/RUST_QUEUE.md`.
- Working on **0.2**. This week: `docs/PLAN_0.2_PUSH.md` (6-day push). Earlier plan: `docs/PLAN_0.2.md`. Locked choices: `docs/DECISIONS.md`.
- Open Godot gaps (short list): `docs/PARITY_OPEN.md`. Worker mistakes: `docs/JESTER.md`.

## Latest numbers (release, plugged in, 2026-11-21)
Camp: avg 114 FPS, worst 1% 68. Ashford core (`--at=-384,1792`, seed 42, 18 zombies): avg 90, p99 15 ms, worst 1% 49-55; 211k tris (budget 228k), 159k shadow tris (over the 121k budget, FPS still fine), 56 draw calls.
60 FPS triangle limit (bench): 380k-700k (`data/budgets.json`, `docs/BUDGETS.md`).

## Now: 0.2 push (docs/PLAN_0.2_PUSH.md), day 1 done 2026-11-21 (owner out of tokens; resume when told "continue")
- MERGED in main: R16 foundations (camp upgrades core, build-mode core, weapons + projectiles, hordes + director + special zombie data, lake layout, water renderer, LOD fade, big map shows Ashford, HUD polish), R16-N1 Neon menus, R17 models (camp walls/gates, towers/workshop/doctor/mechanic, weapons + icons, brute/screamer/crawler, base parts + furniture, lake buildings/props), lake in the world, HUD wiring. 648 tests pass. 178 models baked.
- R18 (all 9) MERGED 2026-11-21 evening ("merge now"): 720 tests pass, 178 models baked. Quick checks: horde test 121 FPS avg / 79 worst 1% at the camp (quiet).
- Stutter (2026-11-22): CAUSE FOUND + FIXED: zombie flow field rebuild took 10-15 ms every 0.5 s; now 0.1 ms still / 2.5 ms walking (STUTTER.md). Spike log now splits own work vs gap. Remaining slow frames are GPU-bound (`surface acquire`); re-measure quiet.
- R19 (2026-11-22, all 9 MERGED): camp board layout (Claude fixed sideways card text), build mode panel/tint + base saved (state in PlaySide, raw-pointer global removed), camp upgrade models placed (`--camp-levels=W,G,K,T`), guild tab wired by Claude (`--camp-screen=guild`, actions, save `extra["guild"]`, kill + morning hooks), HUD horde/special/camp-attack/wave warnings, lake: docks above water + shore-following watertight water mesh, lake on big map + minimap, z-fight cleanup (city buildings ~99% less; camp models OK). 760 tests pass. FPS: Ashford core avg 75 / worst 1% 40, camp 111 / 72.
- White line + water over the player FIXED (Claude): lit pass discarded depth, the water pass read garbage depth; now stored.
- R20 (2026-11-22, all 9 MERGED): owner issues 001-008 (docs/ISSUES.md) fixed except 003 tower colliders (settlement.py has unsaved work); hotbar now decides the hand item; z-fight: boathouse, house_small, barn, cabin, gas_station, fish_shack and all props under 0.05 m2. 773 tests pass. Owner binary: target/play/rotland.
- R21-R25 (2026-11-23, Neon writes, Haiku wires): fishing + swim hands, zombies attack built parts (`data/siege.json`), teammates follow/guard/fight and are down until morning (`data/companions.json`), guild deliver/explore/clear + tracker, brute charge/screamer call/crawler lunge (`data/special_zombies.json`). 849 tests pass. NOT yet play-tested by the owner: siege hits, hand-in click, special zombie moves, followers beside the player. Known: "Task ready" notice overlaps the "IN CAMP" label; `game/src/zombie/specials.rs` is 531 lines (split it).
- Small leftovers: tire_pile tyres look like solid discs (6 cm black hole); ladder prompt and car health not yet seen in a screenshot.
- NEXT round: guild fetch ("deliver") and explore tasks have no hooks yet; recruits in the 3D world.
- Z-fighting flicker: apartment fixed (window details now 2 cm out of the wall). CHECK the other 7 city buildings + camp/lake buildings for details flush with walls (office glass bands, police stripes, hospital stripes, shop signs...).
- Seen in screenshots, TO FIX next: camp board screen layout broken (cards far too big, cut off at 1920x986: `--camp-screen=upgrades`); build mode shows the ghost floor but no part/cost panel and no green/red tint (`--build-test`); build mode keeps state in globals (move into app state + save); base parts not saved yet; camp upgrade models not placed in the camp yet (call `world::camp_upgrade_props` when levels change); guild has no UI/NPC yet (logic only); HUD events for hordes/camp attack not hooked.
- Known: water drawn as 8 m strips over the lake only (beach now dry), BUT docks sit at ground height under the water (dock y must be water_level + ~0.4, in `world/src/lake/place.rs`), the shore edge is blocky (8 m strips), and a thin white line shows across the screen at `--at=470,-1420` (probably a water strip edge seen from far; check); crawler pose wrong (R18-07); FPS worries: city core worst 1% 21, lake 17 under heavy load (R18-08 + a quiet re-measure first thing).
- Tools fixed today: gate scope reads `Files:` from main; `tools/claim_target.sh` rebuilds our crates every gate (shared target mixed worktrees); gate never deadlocks inside an outer flock.
- Seed 42 places: city core (-384, 1792), lake first dock (528, -1478). Dev panel: City core / City entry / Camp road / Lake teleports.

- 2026-11-24: A12 actions, A13 melee v2 (combos, weapon weight, shove tap / block hold, DECISIONS 2026-11-24), A14 directional flinch, running-legs fix (foot IK off at run). R27 (6 Gemini lanes via Antigravity, prompts in tasks/R27-prompts.md): A15 wounds, A16 directional deaths, A18 teammates/guards on the new animator, guild deliver/explore hooks, Task-ready overlap + specials.rs split, city z-fight. 1033 tests. Not yet seen in game: block pose with the bat, combos, wounds, deaths, teammate weapons, guild hand-in.
- Animation plan left: A20 (finger bones, real grips, polish). Downed/revive not started (owner decision).

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

