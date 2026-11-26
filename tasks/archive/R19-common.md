# R19 common rules (0.2 push, day 2 morning: finish and polish what R18 wired)

Read `tasks/R13-common.md` (Rust lanes / Blender lanes) and `tasks/R18-common.md` (wiring rules). Plan: `docs/PLAN_0.2_PUSH.md`. Status: `docs/STATUS.md`.
- Other lanes edit other files at the same time: edit ONLY the files on your `Files:` line (a `dir/*` entry means any file in that folder, new files included). Never edit `Files:`.
- If you need a change in a file you don't own (save crate, `main.rs`, another lane's file), write the exact change in `## Worker notes` under "Wiring notes for Claude". Claude applies it at merge.
- Keep every file under ~300 lines; split big files into modules (one topic per file).
- Every number in game code: put it in a `data/*.json` file with a `// data: <file> <key>` tag, or tag it `// new: DECISIONS 2026-11-22` if it is a pure layout/UI number (pixel sizes, margins).
- Every lane adds tests for its pure logic (layout math, state changes, save round trips).
- Never wrap `tools/gate.sh` in flock and never end your turn while a command runs. Run `tools/gate.sh --task tasks/<your task>.md` until the last line is `GATE: PASS`. Fill `## Worker notes`, commit once.
- Screen sizes to design for: 1920x986 (the owner's window) and 1280x720. Nothing may be cut off or overlap at either size.
