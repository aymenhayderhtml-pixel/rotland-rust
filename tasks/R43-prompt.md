# R43 prompt (paste everything inside the box into Hydrogen in Antigravity)

```
You are the LEAD for round R43 of Rotland (/home/aymen/Documents/rotland-rust): NEW HALDEN CLIMBING. This round makes the city vertical:
- 5 climbable towers (4 normal + the tallest landmark), with a real stairwell from the lobby to the top floor
- open stop-off floors (offices, apartments, a survivors' last stand) with loot and zombies that wake only when you are on that floor
- a ladder up through a roof hatch
- plank bridges and ladders to nearby roofs
- 8 brick towers with working fire escapes
- the radio on the mast of the tallest tower: the goal, a message and a top loot crate

QUALITY FIRST and DETAIL FIRST.

FPS RULE (owner): measure ONLY the new city: `tools/perf_scenes.sh --set metro` before and after. NEVER run `--set all`. Pass = AVERAGE FPS at least 20. Write the 1% low for information only.

First read: AGENTS.md, tasks/LEAD.md, docs/PLAN_1.1.md, tasks/R43-common.md ("What is true today", owner decisions, "R43 shared names": follow them EXACTLY), tasks/R43-round.md and its 9 task files. Read docs/CODEMAP.md once and the R42 Lead notes + "Claude review fixes" in tasks/R42-round.md. Look at world/src/metro/towers.rs, tower_parts.rs, rotland/src/metro_render/hulls.rs, rotland/src/streaming_metro.rs, game_player/src/ladders/, tools/blender/towers/ before handing out work.

1. Setup from LEAD.md: ONE worktree ../rotland-r43, branch r43, shared CARGO_TARGET_DIR. Bake assets/models.pack once in the worktree (it is not in git). Do Phase 0 yourself.
   - Only you build. Subagents never run cargo, the gate or the game.
   - Blender subagents (03, 04, 05) may run Blender under `flock /tmp/rotland-blender.lock` and must look at their previews (cut-aways, the 3-module stair stack, the fire-escape stack).
2. Start subagents for R43-01 to R43-07 at once.
   - Start R43-08 when they have all reported, then R43-09.
   - Give each one: the LEAD.md subagent rules, the whole R43-common.md, its task file, and the existing files to copy the style of.
3. REVIEW each diff and EVERY model preview yourself. Check:
   - Reject plain grey boxes: the stairwell must read as a dirty concrete stairwell, each floor as an office / apartments / a last stand.
   - The stair ramps line up module to module, and every walk surface has a collider.
   - Non-climb towers are unchanged. Numbers live in data, with tags. world/tests/golden.rs passes UNEDITED. Files are under ~300 lines.
4. Add the shared-file lines. Blender ONCE under the lock: `tools/blender/build_r43_models.py` (0 Traceback, ends with R43 BUILD OK). Then bake (`cargo run --release -p baker`).
5. `tools/gate.sh --task tasks/R43-round.md` in the foreground until GATE: PASS.
6. `cargo build --profile play`, then EVERY check in R43-round.md. Look at every picture.
   - The walk, climb, plank and radio checks must be proven by the stats (position y, playerOnFloor, poiZombiesAwake), not only by pictures.
   - Fix and re-shoot until each check passes.
7. Fill "## Lead notes": tasks done, review fixes, the last 5 gate lines, picture paths, metro FPS before/after with average and 1% low, the stats numbers of each walk/climb check, and anything Claude must check. Make ONE commit on r43 (git add only changed files). Never push, never touch main.
8. Tell me "R43 done" plus a short summary.

Safety: never `pkill -f`; never write to real save folders; heavy commands under `flock /tmp/rotland-heavy.lock`; never add sounds.
```
