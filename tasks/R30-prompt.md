# R30 prompt: paste this into ONE Antigravity chat (Gemini 4 Hydrogen)

```
You are the LEAD for round R30 of the Rotland game (Rust repo at /home/aymen/Documents/rotland-rust).

First read: AGENTS.md, then tasks/R30-common.md (the rules for this round; follow them exactly), then tasks/R30-round.md and the four task files it lists:
- tasks/R30-01-test-flags.md  (test flags so screenshots can show downed / revive / rescue)
- tasks/R30-02-team-fixes.md  (rescued survivors start at their spot; teammates kneel to revive)
- tasks/R30-03-stutter.md     (measure and fix the stutter)
- tasks/R30-04-split.md       (split the 5 biggest files; PHASE 2, only after the other three are in)

How we work (the laptop is slow, so the game builds only ONCE):
1. Make ONE worktree as R30-common.md says (../rotland-r30, branch r30, CARGO_TARGET_DIR shared). No separate folder copies for subagents.
2. Phase 1: start Hydrogen subagents for R30-01, R30-02 (one for the survivor start, one for the kneel clip) and R30-03 at the same time. Give each the "Subagent rules" from R30-common.md word for word, plus its task file. Subagents write code only: no cargo, no gate, no blender, no game. For R30-03 YOU do the "before" measurement first (steps in the task file).
3. When phase 1 is in and you've added the shared-file lines: Phase 2, start the split subagents for R30-04 (one per big file is fine).
4. Rebuild the characters once (command in R30-02), then run `tools/gate.sh --task tasks/R30-round.md` in the foreground and wait for it. Fix errors and run it again until the last line is GATE: PASS. Then do the R30-03 "after" measurement.
5. Fill "## Lead notes" in tasks/R30-round.md (each task done or not, last 5 lines of the final gate, before/after stutter numbers, anything Claude must check). ONE commit on branch r30 (git add only your changed files). Never push, never touch main.
6. Tell me: "R30 done" plus a short summary.
```
