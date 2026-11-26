# R18 common rules (0.2 push, day 2: make the new systems playable)

Read `tasks/R13-common.md` (Rust lanes / Blender lanes) and `tasks/R16-common.md` (foundation rules). Plan: `docs/PLAN_0.2_PUSH.md`.
- This round WIRES systems into the game. Other lanes edit other files at the same time: only edit the files on your `Files:` line. If you need a change in a file you don't own (e.g. the save file, `frame_ui.rs`, `main.rs`), write the exact change in `## Worker notes` under "Wiring notes for Claude" and Claude applies it at merge.
- Keep every new/changed file under ~300 lines; split into modules.
- Add a scripted test path where you can: a command-line test flag (like `--hub=map`, `--camp-screen=shop`) that opens your feature at start, so Claude can screenshot it. Add the flag in the file you own that parses startup args only if it is on your `Files:` line; otherwise put it in Worker notes.
- `tools/gate.sh --task <task>` until `GATE: PASS` (never wrap it in flock). Don't edit `Files:`. Commit once.
