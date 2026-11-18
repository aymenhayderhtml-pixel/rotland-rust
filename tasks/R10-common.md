# Rules for every R10 worker task (2026-11-18)

Read `AGENTS.md` first, then this file, then your own `tasks/R10-XX-*.md`. Read `docs/CODEMAP.md` once to find files; then open only what your task needs.

Everything in `tasks/R9-common.md`, `R8-common.md`, `R7-common.md` still applies, **except the paths**: the repo moved. There is no `rust/` folder any more. `rust/rotland/src/main.rs` is now `rotland/src/main.rs`, `rust/game/tests/` is now `game/tests/`, and so on.

The Godot reference (frozen) is at `../zombi servival/` (full path `/home/aymen/Documents/zombi servival/`) (C# in `scripts/`, data in `data/`). "Port, don't invent": every number you write comes from a Godot file, a `data/*.json` file, or the issue text. If you think a value should change, write that in your notes, not in the code.

## Lanes and phases
- **Phase A** (start now, in parallel): R10-01 to R10-08 and R10-11.
- **Phase B** (start only after Claude has merged R10-01 into `main`): R10-09, R10-10. They work inside the new files R10-01 creates.
- R10-01 is the only lane allowed to edit `rotland/src/main.rs`. Every other lane: **never edit main.rs** (or the files R10-01 creates). Write the hook you need under "Wiring for Claude".
- Edit **only** the files your task lists under "You own". If you need a change in a file you don't own, write it in your notes.

## Build rule for R10 (changed from R9)
To cut down Claude's fix-up work, workers **may** now check their own code if their tool can run commands on this machine:
```bash
flock /tmp/rotland-heavy.lock cargo check -p <your crate> --all-targets
flock /tmp/rotland-heavy.lock cargo test -p <your crate> <your test name filter>
```
- Always through `flock`. Never run the game window, never `cargo build --release`, never the baker (Claude bakes), never `pkill`.
- If you can't run commands, don't worry: write carefully and say "not checked" in your notes.
- Never claim "tests pass" unless you ran them and paste the last lines of the output in your notes.

## Mistakes that keep happening (docs/JESTER.md #1-#33) — don't repeat them
1. Never call a function you haven't seen. Open the type's file and copy its real signature (glam has `distance`, not `distance_to`).
2. Before saying how another system works, open it and quote the line in your notes.
3. Don't build a second copy of state another system already keeps. Search first.
4. Add only the public API your task needs. Test-only helpers go under `#[cfg(test)]` (clippy fails on dead code).
5. Use `if let .. && let ..` chains, not nested `if`s that clippy collapses. `gen` is a reserved word. Enum variants use `::`.
6. Godot yaw: forward = (-sin yaw, 0, -cos yaw). East (+X) is yaw -PI/2. Use the shared helper if R10-01 added one.
7. egui is 0.37: copy patterns from `rotland/src/hub_ui/` before using any egui call.
8. Find files with `paths::repo_path("data/...")`, never `env!("CARGO_MANIFEST_DIR")`.
9. A key in the settings table is not a feature: check the code that reads it.
10. Files ~300 lines max. One topic per file. If a file you own is already over, split it as part of your task.

## Git
- You work in your own git worktree on branch `r10-XX` (the owner made it for you).
- When done: **one commit** on your branch, message starts with `R10-XX:`. Never push, never touch `main`, never merge.

## Worker notes (fill this in at the bottom of your task file, then commit)
- **Files changed** (each with one line on what changed)
- **New public API** (signatures)
- **Wiring for Claude**: exact calls to add, in which file, when (each frame / on key / on load), with which inputs
- **Tests**: names, and every number a test checks + where that number comes from (Godot file:line or json key)
- **Checked?**: the cargo command you ran and its last lines, or "not checked"
- **Questions / known issues**
