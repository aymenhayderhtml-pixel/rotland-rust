# R42 prompt (paste everything inside the box into Hydrogen in Antigravity)

```
You are the LEAD for round R42 of Rotland (/home/aymen/Documents/rotland-rust): NEW HALDEN BIG PLACES + THE FAILED ARMY LINE. The city has four empty reserved plots; this round fills them:
- a hospital (ER canopy, ambulances, triage tents, a big pile of body bags)
- a police station (police cars, barricades, cells, an armoury)
- a fire station (three bays with fire trucks)
- the failed army line across an avenue: sandbags, T-walls, razor wire, a tank, burnt trucks, a crashed helicopter, evacuation tents, loudspeakers

Every main building can be walked into on the ground floor. Top loot and heavy danger, with new zombie looks: soldiers, police, medics, firefighters. QUALITY FIRST and DETAIL FIRST.

FPS RULE (owner): measure ONLY the new city: `tools/perf_scenes.sh --set metro` before and after. NEVER run `--set all`. Pass = AVERAGE FPS at least 20. Write the 1% low for information only.

First read: AGENTS.md, tasks/LEAD.md, docs/PLAN_1.1.md, tasks/R42-common.md ("What is true today", owner decisions, "R42 shared names": follow them EXACTLY), tasks/R42-round.md and its 9 task files. Read docs/CODEMAP.md once and the R41 Lead notes + "Claude review fixes" in tasks/R41-round.md. Look at world/src/metro/, world/src/poi.rs, game_zombie/src/poi_zombies/, tools/blender/buildings/kit.py, tools/blender/common.py before handing out work.

1. Setup from LEAD.md: ONE worktree ../rotland-r42, branch r42, shared CARGO_TARGET_DIR. Do Phase 0 yourself.
   - Only you build. Subagents never run cargo, the gate or the game.
   - Blender subagents (03, 04, 05, 06, 07) may run Blender under `flock /tmp/rotland-blender.lock` and must look at their previews, including the cut-away of each walk-in ground floor.
2. Start subagents for R42-01 to R42-08 at once.
   - Start R42-09 when they have all reported.
   - Give each one: the LEAD.md subagent rules, the whole R42-common.md, its task file, and the existing files to copy the style of.
3. REVIEW each diff and EVERY model preview yourself. Check:
   - Reject low-detail boxes: the buildings must read as a hospital, a police station and a fire station from the outside AND inside, and the army line as a battle that was lost.
   - Walk-in floors have colliders and open door gaps.
   - Numbers live in data, with tags. Spawning outside New Halden is unchanged.
   - world/tests/golden.rs passes UNEDITED. Files are under ~300 lines.
4. Add the shared-file lines. Blender ONCE under the lock: `tools/blender/build_r42_models.py` (0 Traceback, ends with R42 BUILD OK). Then bake (`cargo run --release -p baker`).
5. `tools/gate.sh --task tasks/R42-round.md` in the foreground until GATE: PASS.
6. `cargo build --profile play`, then EVERY check in R42-round.md, each place inside and out. Look at every picture.
   - Zombies must really come at you near the places: check the stats, not only the pictures.
   - Fix and re-shoot until each place reads at a glance.
7. Fill "## Lead notes": tasks done, review fixes, the last 5 gate lines, picture paths, metro FPS before/after with average and 1% low, and anything Claude must check. Make ONE commit on r42 (git add only changed files). Never push, never touch main.
8. Tell me "R42 done" plus a short summary.

Safety: never `pkill -f`; never write to real save folders; heavy commands under `flock /tmp/rotland-heavy.lock`; never add sounds.
```
