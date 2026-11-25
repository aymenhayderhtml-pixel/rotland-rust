# R35 prompt (paste everything inside the box into Hydrogen in Antigravity)

```
You are the LEAD for round R35 of Rotland (/home/aymen/Documents/rotland-rust): ALL-OUT DETAIL. Read tasks/LEAD.md, then tasks/R35-round.md and its 6 task files. Nothing else unless a task needs it.

SAVE TOKENS (the owner pays for every one):
- Exactly 6 subagents, one per task file, all started at once. Give each only: the LEAD.md subagent rules, the "R35 shared names" section, and its task file path.
- Do not read docs, CODEMAP or big files "to understand"; grep for what you need.
- Run the gate in the foreground and wait. Fix small errors yourself; don't restart subagents for one-line fixes.
- Look at each screenshot once; retake only if you changed something.
- Short reports, no code pasted into chat.

Steps:
1. Setup from LEAD.md (worktree ../rotland-r35, branch r35, shared CARGO_TARGET_DIR).
2. Start the 6 subagents.
3. When all report: add the shared-file lines and the call sites they report (append_town_contents lines, mod lines, frame-loop calls). Run `tools/gate.sh --task tasks/R35-round.md` until GATE: PASS (a scope-only failure about the Files: line is OK; note it).
4. Do "Checks after the gate" in R35-round.md (Blender once, bake once, shots, FPS; lower data densities if a scene is under 60 avg / 30 worst 1%).
5. Fill "## Lead notes" (each task done or not, last 5 gate lines, FPS per scene, picture paths). ONE commit on r35 (git add only changed files). Never push, never touch main.
6. Tell me "R35 done" in 5 lines or less.
```
