# R31 prompt (paste everything inside the box into Hydrogen High in Antigravity)

```
You are the LEAD for round R31 of the Rotland game (Rust repo at /home/aymen/Documents/rotland-rust): NEW WORLD ENGINE + 4 NEW BIOMES (desert, snow, swamp, mountains).

Read, in this order: AGENTS.md, tasks/LEAD.md (your rules and the subagent rules), tasks/R31-round.md (owner decisions, Phase 0, the "R31 shared names", and the checks), then the 8 task files R31-01 ... R31-08.
If /home/aymen/Documents/rotland-rust/neon/in/R31-research.md exists, read it too and use what fits.

How we work (the laptop is slow, so the game builds only ONCE):
1. Make ONE worktree as LEAD.md says (../rotland-r31, branch r31 from main, CARGO_TARGET_DIR shared).
2. Do Phase 0 from R31-round.md (before screenshots + FPS table).
3. Start 8 Hydrogen subagents at the same time, one per task file. Give each the subagent rules from LEAD.md AND the "R31 shared names" section word for word, plus its task file. They write code only: no cargo, no gate, no blender, no game.
4. When all have reported: add the shared-file lines, run the Blender build once (R31-round.md "After the gate passes" step 1 can go here), then run tools/gate.sh --task tasks/R31-round.md in the FOREGROUND and wait. Fix the errors and loop until the last line is GATE: PASS. world/tests/golden.rs must pass WITHOUT editing it: the old world inside the core must not change.
5. Bake, make the world map picture, take the screenshots (before vs after in the core, one per new biome), run the FPS scenes. LOOK at every picture and fix what's clearly wrong.
6. Fill "## Lead notes" in tasks/R31-round.md. ONE commit on branch r31 (git add only your changed files). Never push, never touch main.
7. Tell me: "R31 done" plus a short summary with the FPS table and the screenshot paths.
```
