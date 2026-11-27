# R41 prompt (paste everything inside the box into Hydrogen in Antigravity)

```
You are the LEAD for round R41 of Rotland (/home/aymen/Documents/rotland-rust): NEW HALDEN, ZOMBIES, LOOT, SILENT LIFE. R38-R40 built the dead, overgrown city; this round fills it.
- Danger rises toward the centre, and runners rule the night.
- Slow herds of 12-30 walkers shamble down the avenues.
- Crows burst off bodies and roofs when you come close; torn flags and plastic flap in the wind; power lines are down; faded billboards show made-up ads.
- Loot depends on the district, with city-only items: army gear (body armour, helmet, army rifle, rations), night-vision goggles on batteries, rare medicine (antibiotics, surgery kit, adrenaline), gun parts and car parts.
- It also fixes the R40 leftovers: too-long files split, and the fallen tower can be climbed.

QUALITY FIRST and DETAIL FIRST.

FPS RULE (owner): measure ONLY the new city: `tools/perf_scenes.sh --set metro` before and after. NEVER run `--set all`. Pass = AVERAGE FPS at least 20. Write the 1% low for information only (it is mostly spikes from the AI tools running on the laptop).

First read: AGENTS.md, tasks/LEAD.md, docs/PLAN_1.1.md, tasks/R41-common.md ("What is true today", owner decisions and "R41 shared names": follow them EXACTLY), tasks/R41-round.md and its 10 task files. Read docs/CODEMAP.md once and the R40 Lead notes in tasks/R40-round.md. Look at world/src/metro/, game_zombie/src/ (spawner, horde), rotland/src/play.rs (spawn_tick), rotland/src/crates.rs, rotland/src/wildlife.rs, data/items.json, data/loot_tables.json before handing out work.

1. Setup from LEAD.md: ONE worktree ../rotland-r41, branch r41, shared CARGO_TARGET_DIR. Do Phase 0 yourself.
   - Only you build. Subagents never run cargo, the gate or the game.
   - Blender subagents (04, 07, 10) may run Blender under `flock /tmp/rotland-blender.lock` and must look at their previews.
2. Start subagents for R41-01 to R41-08 and R41-10 at once.
   - Start R41-09 when they have all reported.
   - Give each one: the LEAD.md subagent rules, the whole R41-common.md, its task file, and the existing files to copy the style of.
3. REVIEW each diff and EVERY model preview and icon yourself. Check:
   - Reject low-detail boxes.
   - Billboard faces, flags and wires don't z-fight.
   - Numbers live in data, with tags.
   - Spawning outside New Halden is unchanged.
   - world/tests/golden.rs passes UNEDITED.
   - Files are under ~300 lines, and play.rs does not grow.
4. Add the shared-file lines. Blender ONCE under the lock: `tools/blender/build_r41_models.py` (0 Traceback, ends with R41 BUILD OK), plus R41-10's two damage scripts. Then bake (`cargo run --release -p baker`).
5. `tools/gate.sh --task tasks/R41-round.md` in the foreground until GATE: PASS.
6. `cargo build --profile play`, then every check in R41-round.md. Look at every picture. Fix and re-shoot until:
   - the herd reads as a herd
   - the night is full of runners
   - night vision looks green and usable
   - crows sit on bodies and fly up
   - flags flap
   - poles and billboards make the streets busier
7. Fill "## Lead notes": tasks done, review fixes, the last 5 gate lines, picture paths, metro FPS before/after with average and 1% low, and anything Claude must check. Make ONE commit on r41 (git add only changed files). Never push, never touch main.
8. Tell me "R41 done" plus a short summary.

Safety: never `pkill -f`; never write to real save folders; heavy commands under `flock /tmp/rotland-heavy.lock`; never add sounds.
```
