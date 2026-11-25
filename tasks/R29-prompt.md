# R29 prompt: paste this into ONE Antigravity chat (Gemini 4 Hydrogen)

```
You are the LEAD for round R29 of the Rotland game (Rust repo at /home/aymen/Documents/rotland-rust).

First read: AGENTS.md, then tasks/R29-common.md (the rules for this round; follow them exactly), then tasks/R29-round.md and the three task files it lists:
- tasks/R29-01-downed.md  (knocked down, crawl, revive)
- tasks/R29-02-rescue.md  (rescue survivors in the world)
- tasks/R29-03-tower.md   (solid camp guard towers)

How we work this round (the laptop is slow, so the game builds only ONCE):
1. Make ONE worktree as R29-common.md says (../rotland-r29, branch r29, CARGO_TARGET_DIR shared). Do not give subagents their own folder copies.
2. Start Hydrogen subagents in that same folder. Use at least: downed logic, downed wiring, rescue logic, rescue wiring, tower (the wiring ones start after their logic one reports). Give each one the "Subagent rules" from R29-common.md word for word, plus its task file and the part it does. Subagents write code only: no cargo, no gate, no blender, no game.
3. When they report, add their shared-file lines yourself, rebuild the settlement model once (command in R29-03), then run `tools/gate.sh --task tasks/R29-round.md` in the foreground and wait for it. Fix the errors and run it again until the last line is GATE: PASS.
4. Fill "## Lead notes" in tasks/R29-round.md (each task done or not, last 5 lines of the final gate, anything Claude must check). Make ONE commit on branch r29 (git add only your changed files). Never push, never touch main.
5. Tell me: "R29 done" plus a 3-line summary.
```
