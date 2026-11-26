# R34 prompt (paste everything inside the box into Hydrogen in Antigravity)

```
You are the LEAD for round R34 of Rotland (/home/aymen/Documents/rotland-rust). Read tasks/LEAD.md, then tasks/R34-round.md and its 3 task files. Nothing else unless a task needs it.

SAVE TOKENS (the owner pays for every one):
- Exactly 3 subagents (A rivers, B roads+towns, C speed). Give each only: the LEAD.md subagent rules, its task file path, and for C your Phase 0 numbers. No other context.
- Do not read docs, CODEMAP or big files "to understand"; grep for what you need.
- Run the gate in the foreground and wait. If it fails, fix small errors yourself; do not restart subagents for one-line fixes.
- Look at each screenshot once. Only retake a shot if you changed something.
- Short reports: no long summaries, no pasting code into chat.

Steps:
1. Setup from LEAD.md (worktree ../rotland-r34, branch r34, shared CARGO_TARGET_DIR).
2. Start A and B now. Do Phase 0 from R34-round.md, then start C with the numbers.
3. When all report: add shared-file lines, run `tools/gate.sh --task tasks/R34-round.md` until GATE: PASS (a scope-only failure about the Files: line is OK; note it).
4. Do the "Checks after the gate" in R34-round.md.
5. Fill "## Lead notes" (each task done or not, last 5 gate lines, before/after FPS, picture paths). ONE commit on r34 (git add only changed files). Never push, never touch main.
6. Tell me "R34 done" in 5 lines or less.
```
