# R36 prompt (paste everything inside the box into Hydrogen in Antigravity)

```
You are the LEAD for round R36 of Rotland (/home/aymen/Documents/rotland-rust): WEAPONS + GUN FEEL. Read tasks/R36-common.md, then tasks/R36-round.md and its 5 task files. Nothing else unless a task needs it.

SAVE TOKENS (the owner pays for every one):
- 6 subagents, all started at once: gun feel (01), weapon models + icons (02), new weapons logic + data (03 logic), new weapons wiring in rotland (03 wiring), animations + grips (04), explosion + backpack (05). Give each only: the subagent rules and "R36 shared names" from R36-common.md, and its task file path.
- Do not read docs, CODEMAP or big files "to understand"; grep for what you need.
- Run the gate in the foreground and wait. Fix small errors yourself; don't restart subagents for one-line fixes.
- Look at each screenshot once; retake only if you changed something.
- Short reports, no code pasted into chat.

Steps:
1. Setup from R36-common.md (worktree ../rotland-r36, branch r36, shared CARGO_TARGET_DIR).
2. Start the 6 subagents.
3. When all report: add the shared-file lines. Blender ONCE each, under `flock /tmp/rotland-blender.lock`: weapons + icons, then characters (commands in the task files; 0 Traceback; old models must not change except the characters). Bake once.
4. `tools/gate.sh --task tasks/R36-round.md` until GATE: PASS (a scope-only failure about the Files: line is OK; note it).
5. `cargo build --profile play`, take the 4 check shots in R36-round.md with target/play/rotland and look at them. Fix anything clearly wrong (gun floating off the hand, backpack inside the body).
6. `tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland`: zombies scene must not drop more than 5 FPS from R35 (68.7 avg).
7. Fill "## Lead notes" (each task done or not, last 5 gate lines, picture paths, FPS). ONE commit on r36 (git add only changed files). Never push, never touch main.
8. Tell me "R36 done" in 5 lines or less.
```
