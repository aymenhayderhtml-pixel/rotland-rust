# LEAD.md: master rules for every Hydrogen lead round (since 2026-11-25)

The owner pastes one short prompt: "You are the LEAD for round <R>. Read tasks/LEAD.md, then tasks/<R>-round.md." Everything else is here.

## Roles
- **Lead** (Antigravity, Gemini 4 Hydrogen High): sets up, measures if the round asks, hands out work, adds shared-file lines, **builds/tests once**, fixes, screenshots, commits.
- **Subagents** (Hydrogen, started by the lead): write code only, in the lead's folder.
- **Claude** (separate app): writes the round files, then merges the finished branch, checks it, and updates the game the owner plays.
- **Neon** (gemini.com, chat only): online research. If `/home/aymen/Documents/rotland-rust/neon/in/<R>-research.md` exists (main checkout; the folder is not in git, so it is not in your worktree), the lead reads it before handing out work and uses what fits.

## Setup (lead, once per round)
```bash
cd /home/aymen/Documents/rotland-rust
git worktree add ../rotland-<r> -b <r> main          # <r> = round name in lower case, e.g. r30.5
cd ../rotland-<r>
export CARGO_TARGET_DIR=/home/aymen/Documents/rotland-rust/target   # reuse built crates; never build from nothing
```
Everyone works in that one folder. Subagents get no copies of it, and different subagents edit different files.

## Subagent rules (copy word for word into every subagent's instructions)
- Read `AGENTS.md`, `tasks/LEAD.md` (this section), the round file and your task file. Open only the files your task needs (`docs/CODEMAP.md` tells you where things are; find code with grep, don't trust old line numbers).
- Edit ONLY the files your task says you own. **Shared files** (`rotland/src/app/state.rs`, `rotland/src/app/mod.rs`, `rotland/src/main.rs`, `*/src/lib.rs`, `game/tests/all/main.rs`, any `Cargo.toml`) belong to the lead, unless your task gives them to you. Put the exact lines you need in your report.
- **Never run `cargo`, `tools/gate.sh`, `blender` or the game.** Read a function before you call it, and match the real names and types.
- Port, don't invent: numbers go in `data/*.json`, and every number in Rust code gets `// data: <file> <key>` or `// new: DECISIONS <date>`. Perf/build tuning constants may use `// new: PERF <round>`.
- Files under ~300 lines. Pure logic in library crates with unit tests (`game` tests in `game/tests/all/<name>.rs`).
- Clip tables: EXACTLY the format in `tasks/R27-common.md`.
- When done, fill `## Worker notes` in your task file (what changed, shared-file lines, how to see it, anything unsure) and report to the lead. Don't commit.
- Never touch the owner's own work unless told to. Never `pkill -f`. Never write to real save folders (use `--save-dir=/tmp/<r>-saves`).

## Lead: the loop
1. If the round has a **Phase 0 (measure)**, do it yourself first (you may run the game for measuring/screenshots: release build, under `flock /tmp/rotland-heavy.lock`, always `--save-dir`). Then turn what you measured into subagent jobs.
2. Start the subagents, as many as the round suggests (tasks marked "after X" wait for X).
3. When they report: add the shared-file lines. If `tools/blender/**` changed, run each needed Blender build ONCE (commands in the task files) and check the logs have no `Traceback`.
4. Build + test ONCE: `tools/gate.sh --task tasks/<R>-round.md` in the FOREGROUND, and wait (it can take minutes; do not stop or end your turn). Fix the errors yourself (or send one subagent the exact error) and loop until the last line is `GATE: PASS`.
5. Do the round's checks (measurements after, screenshots). Look at every screenshot you take.
6. Fill `## Lead notes` in the round file: each task done or not, the last 5 lines of the final gate, before/after numbers, screenshot paths, and anything Claude must check.
7. ONE commit on your branch (`git add` only changed files, never `-A` / `commit -a`). Never push, never touch `main`. Tell the owner "<R> done" plus a short summary.
