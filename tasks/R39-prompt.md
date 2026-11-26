# R39 prompt (paste everything inside the box into Hydrogen in Antigravity)

```
You are the LEAD for round R39 of Rotland (/home/aymen/Documents/rotland-rust): NEW HALDEN TOWERS, the second round of v1.1. R38 placed the city with plain placeholder block towers; this round replaces them with a rich, detailed New-York-style tower kit (brick tenements with fire escapes, stone pre-war offices, shattered glass towers, 70s concrete slabs, Art Deco), 3-5 landmark skyscrapers (~150 m) and walk-in lobbies. QUALITY FIRST and DETAIL FIRST: the owner hated the old sparse look and accepts down to 20 FPS in the slowest moments.

First read: AGENTS.md, tasks/LEAD.md, docs/PLAN_1.1.md, tasks/R39-common.md (owner decisions + "R39 shared names": follow them EXACTLY), tasks/R39-round.md and its 8 task files. Read docs/CODEMAP.md once and the R38 Worker notes in tasks/archive/R38-*.md. Look at world/src/metro/, rotland/src/metro_render/, rotland/src/props.rs and tools/blender/metro/ before handing out work.

1. Setup from LEAD.md (ONE worktree ../rotland-r39, branch r39, shared CARGO_TARGET_DIR). Do Phase 0 yourself. Only you build; subagents never run cargo, the gate or the game. Blender subagents may run Blender under `flock /tmp/rotland-blender.lock` and must look at their previews.
2. Start subagents for R39-01 to R39-07 at once (06 after 01 reports). Start R39-08 when 01-07 have reported. Give each the LEAD.md subagent rules, the whole R39-common.md, its task file, and the existing files to copy the style of.
3. REVIEW each diff and EVERY model preview yourself. Reject low-detail boxes (the owner's #1 complaint): demand cornices, frames, fire escapes, AC units, broken glass, roof tanks. Check stacked floors have no gaps, numbers live in data files, world/tests/golden.rs passes UNEDITED, files are under ~300 lines.
4. Add the shared-file lines. Blender ONCE under the lock: `tools/blender/build_r39_models.py` (0 Traceback), then bake (`cargo run --release -p baker`).
5. `tools/gate.sh --task tasks/R39-round.md` in the foreground until GATE: PASS.
6. `cargo build --profile play`, then every check in R39-round.md. Look at every picture; fix and re-shoot until the streets look like a dead New York, lobbies are walkable, the skyline reads from far, and every scene's 1% low is at least 20 FPS.
7. Fill "## Lead notes" (tasks done, review fixes, last 5 gate lines, picture paths, FPS before/after, anything Claude must check). ONE commit on r39 (git add only changed files). Never push, never touch main.
8. Tell me "R39 done" plus a short summary.

Safety: never `pkill -f`; never write to real save folders; heavy commands under `flock /tmp/rotland-heavy.lock`; never add sounds.
```
