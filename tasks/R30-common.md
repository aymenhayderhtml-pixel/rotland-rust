# R30 common rules: ONE lead + subagents, ONE build (2026-11-25)

Why: in earlier rounds every worker built and tested on its own, so the slow laptop built the game again and again. This round has **one lead** (you, the Antigravity chat the owner pastes into). The lead hands out work to **subagents that only write code**. Then the lead builds and tests **once**.

## Setup (lead, once)
```bash
cd /home/aymen/Documents/rotland-rust
git worktree add ../rotland-r30 -b r30 main
cd ../rotland-r30
export CARGO_TARGET_DIR=/home/aymen/Documents/rotland-rust/target   # reuse the built crates; never build from nothing
```
Everyone (lead and subagents) works in `/home/aymen/Documents/rotland-r30`, on branch `r30`. **Do not give subagents their own copies of the folder.** They edit different files in the same folder, so there is nothing to merge.

## Subagent rules (copy these into every subagent's instructions)
- Read `AGENTS.md`, `tasks/R30-common.md`, then your task file. Open only the files your task needs (`docs/CODEMAP.md` tells you where things are).
- Edit ONLY the files your task file says you own. **Shared files** (`rotland/src/app/state.rs`, `rotland/src/app/mod.rs`, `rotland/src/main.rs`, `game/src/lib.rs`, `game/tests/all/main.rs`) belong to the lead. Put the exact lines you need there in your report, and the lead adds them.
- **Never run `cargo`, `tools/gate.sh`, `blender` or the game.** Write careful code instead: match the real function names and types you read in the repo, and don't guess an API. Read the function before you call it.
- Port, don't invent: new numbers go in the task's `data/*.json` file, and every number in Rust code gets a tag `// data: <file> <key>` or `// new: DECISIONS 2026-11-25`.
- Files under ~300 lines. Pure logic in `game/` with unit tests in `game/tests/all/<name>.rs`.
- When done, fill `## Worker notes` in your task file (what changed, lines the lead must add to shared files, how to see it in the game, anything unsure) and report back to the lead. Don't commit.
- DO NOT touch `tools/blender/vehicles/*` or `assets/models/vehicles/*`. Never `pkill -f`. Never write to real save folders.

## Lead: build once, fix, finish
1. **Phase 1:** start the subagents for R30-01, R30-02 and R30-03 at the same time. **Phase 2:** after ALL of them have reported and you have added their shared-file lines, start the R30-04 split subagent(s). The split moves code that phase 1 just changed, so it must go last.
   Start the subagents. Tasks that don't share files run at the same time. Inside one task, the pure-logic part (game/, data/, tests) can go first, then the wiring part (rotland/).
2. When all subagents have reported: add their shared-file lines yourself.
3. If any `tools/blender/**` file changed, rebuild those models ONCE (see the task file for the command). Check the log has no `Traceback`.
4. Build + test ONCE: `tools/gate.sh --task tasks/R30-round.md` in the FOREGROUND, and wait for it (it can take minutes; do not stop or end your turn while it runs). Read every error, fix it yourself (or send one subagent the exact error), and run the gate again. Loop until the last line is `GATE: PASS`.
5. Fill `## Lead notes` in `tasks/R30-round.md`: each task done or not, the last 5 lines of the final gate run, and anything Claude must check.
6. ONE commit on branch `r30` (`git add` only the files you changed, never `git add -A` or `commit -a`). Never push, never touch `main`. Then tell the owner "R30 done".
