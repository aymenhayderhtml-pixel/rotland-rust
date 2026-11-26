# R40 prompt (paste everything inside the box into Hydrogen in Antigravity)

```
You are the LEAD for round R40 of Rotland (/home/aymen/Documents/rotland-rust): NEW HALDEN, NATURE TAKES IT BACK. R38 placed the city, R39 built the towers; this round makes it a dead, overgrown city like a famous zombie TV show's: vines on walls, weeds and trees breaking the asphalt, wild lots, moss on cars, one fallen tower across a street, one leaning tower, one burnt-out block, stacked-car walls, spray-painted warnings, missing-person posters, rooftop camps, bodies under sheets. It also fixes the R39 leftovers: bare lobbies, towers too clean, plain roofs, and the landmark view. QUALITY FIRST and DETAIL FIRST.

FPS RULE (owner): measure ONLY the new city: `tools/perf_scenes.sh --set metro` before and after. NEVER run `--set all`. Pass = AVERAGE FPS at least 20. Write the 1% low for information only (it is mostly spikes from the AI tools running on the laptop).

First read: AGENTS.md, tasks/LEAD.md, docs/PLAN_1.1.md, tasks/R40-common.md (owner decisions + "R40 shared names": follow them EXACTLY), tasks/R40-round.md and its 8 task files. Read docs/CODEMAP.md once and the R39 Worker notes in tasks/archive/R39-*.md. Look at world/src/metro/, rotland/src/metro_render/, tools/blender/towers/ before handing out work.

1. Setup from LEAD.md (ONE worktree ../rotland-r40, branch r40, shared CARGO_TARGET_DIR). Do Phase 0 yourself. Only you build; subagents never run cargo, the gate or the game. Blender subagents may run Blender under `flock /tmp/rotland-blender.lock` and must look at their previews.
2. Start subagents for R40-01 to R40-07 at once. Start R40-08 when 01-07 have reported. Give each the LEAD.md subagent rules, the whole R40-common.md, its task file, and the existing files to copy the style of.
3. REVIEW each diff and EVERY model preview yourself. Reject low-detail boxes. Check that vines, graffiti and posters sit 2-5 cm off walls (no z-fighting), rubble is walkable, `_floor_c` stacks gap-free, numbers live in data, world/tests/golden.rs passes UNEDITED, and files are under ~300 lines.
4. Add the shared-file lines. Blender ONCE under the lock: `tools/blender/build_r40_models.py` (0 Traceback), then bake (`cargo run --release -p baker`).
5. `tools/gate.sh --task tasks/R40-round.md` in the foreground until GATE: PASS.
6. `cargo build --profile play`, then every check in R40-round.md. Look at every picture; fix and re-shoot until it looks like nature is taking the city back while it still reads as a big city.
7. Fill "## Lead notes" (tasks done, review fixes, last 5 gate lines, picture paths, metro FPS before/after with average and 1% low, anything Claude must check). ONE commit on r40 (git add only changed files). Never push, never touch main.
8. Tell me "R40 done" plus a short summary.

Safety: never `pkill -f`; never write to real save folders; heavy commands under `flock /tmp/rotland-heavy.lock`; never add sounds.
```
