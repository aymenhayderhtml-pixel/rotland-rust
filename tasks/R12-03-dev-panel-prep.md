# R12-03 (prep): write the Neon task for the F1 dev panel and pack it

You do NOT write the panel code. You read the code and prepare one paste-ready prompt for Gemini Neon (chat only, no file access, ~500k context; keep the pack under ~150k tokens).

Read first: `AGENTS.md`, `docs/STATUS.md`, `tools/README_TOOLS.md`, `docs/CODEMAP.md` (only the rotland/ and game/ parts).

## The feature (owner-facing dev tool, not in Godot)
F1 opens/closes an egui "Dev" window during play (doesn't pause the game, frees the mouse while open):
- FPS graph of the last 300 frames (frame ms, with 16.7 and 33.3 ms lines), triangles, draw calls, player position.
- Teleport: to camp spawn, to typed X/Z (ground height from the world), to the nearest POI.
- Time of day slider (0-24 h) and a "freeze time" checkbox.
- Spawn a zombie of a chosen type 8 m in front of the player; kill all zombies within 50 m.
- Give item: a searchable list of `data/items.json` ids + count.
- God mode toggle (no damage, needs stay full).
- Also a start flag `--at=X,Z` that puts the player there at start (for screenshots).

## What you produce
1. `tasks/R12-03-dev-panel.md`: the Neon task. Include: the feature list above; exactly which new file(s) Neon writes (prefer a new module `rotland/src/dev_panel/` with `mod.rs` + `draw.rs` + `actions.rs`, each under ~300 lines, plus pure logic with unit tests in `game/` if any) and the small edits to existing files (Neon writes full new versions of those existing files, so list them and keep them few and short; if an existing file is long, instead ask Neon for a wiring snippet in NOTES and leave the file out of Files:). Lines:
   - `Files:` every file Neon may write (+ the task file itself).
   - `Read:` full files Neon must see (the files it edits, the state structs it touches, how another egui screen is drawn as an example).
   - `API:` files whose pub signatures it calls (zombie spawning, inventory add, clock/time, player position/teleport, renderer stats, world height, POI list, egui helpers in use).
   - `Data:` items.json, zombies.json.
   - `Test:` a filter matching the new tests (e.g. `dev_panel`).
   - Find the real function names by reading code. Do not list a file you didn't open.
   - egui is 0.37 with `default_fonts`; key handling: show how F-keys are read today.
2. Run `python3 -I tools/neon_pack.py tasks/R12-03-dev-panel.md`. Check the output size it prints. If over ~150k tokens, swap whole `Read:` files for `API:` where possible.
3. Do not commit. Do not edit any code.

Final answer: the pack path, its size in tokens, the Files: list, and anything you weren't sure about.
