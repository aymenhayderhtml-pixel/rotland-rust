# R38 prompt (paste everything inside the box into Hydrogen in Antigravity)

```
You are the LEAD for round R38 of Rotland (/home/aymen/Documents/rotland-rust): NEW HALDEN GROUNDWORK, the first round of v1.1 (a big dying city, 2 km wide, towers up to 150 m, about 6 km from camp). QUALITY FIRST and DETAIL FIRST: the owner wants it dense and dramatic, and accepts down to 20 FPS (not 30) in the slowest moments.

First read: AGENTS.md, tasks/LEAD.md, docs/PLAN_1.1.md, tasks/R38-common.md (owner decisions + "R38 shared names": follow them EXACTLY), tasks/R38-round.md and its 8 task files. Read docs/CODEMAP.md once. Before handing out work, look at how Ashford is built and drawn (world/src/city/, rotland/src/props.rs, rotland/src/streaming.rs) and how the R33 roads and towns work (world/src/roads/, world/src/towns/), so your instructions are precise.

1. Setup from LEAD.md (ONE worktree ../rotland-r38, branch r38, shared CARGO_TARGET_DIR). Do Phase 0 from R38-round.md yourself (budgets, baseline perf, an old save). Only you build; subagents never run cargo, the gate, blender or the game.
2. Start subagents for R38-01 to R38-07 at once. Start R38-08 when 01-07 have reported (give it their Worker notes). Give each the LEAD.md subagent rules, the whole R38-common.md, its task file, and the exact existing files to copy the style of. Ask each for tests and full "## Worker notes".
3. REVIEW each diff yourself: shared names exact, numbers in data files with source tags, world/tests/golden.rs passes UNEDITED (Ashford, camp, home land unchanged), no half-done stubs, files under ~300 lines. Send work back with clear notes if it is not good enough.
4. Add the shared-file lines. Blender ONCE under `flock /tmp/rotland-blender.lock`: `tools/blender/build_r38_models.py` (0 Traceback), look at every preview, then bake.
5. `tools/gate.sh --task tasks/R38-round.md` in the foreground until GATE: PASS (a scope-only failure about the Files: line is OK; note it).
6. `cargo build --profile play`, then every check in "Checks after the gate" in R38-round.md. Look carefully at every picture; fix and re-shoot until it is right: skyline visible from far, the jam looks like a real abandoned evacuation, streets read as a big city, grass outside stays green, every scene's 1% low at least 20 FPS.
7. Fill "## Lead notes" (each task done or not, what you fixed in review, last 5 gate lines, every picture path, FPS before/after, anything Claude must check). ONE commit on r38 (git add only changed files). Never push, never touch main.
8. Tell me "R38 done" plus a short summary.

Safety: never `pkill -f`; never write to real save folders; heavy commands under `flock /tmp/rotland-heavy.lock`; never add sounds.
```
