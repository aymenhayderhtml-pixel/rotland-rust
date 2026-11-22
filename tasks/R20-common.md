# R20 common rules (0.2 push, day 2 afternoon: fix the owner's reported issues)

Read `tasks/R13-common.md` (Rust lanes / Blender lanes) and `tasks/R19-common.md` (all its rules still apply). The owner's bug list: `docs/ISSUES.md` (read only your issue's row: it already names the files and lines; the picture is in `docs/issues/`).
- Edit ONLY the files on your `Files:` line (`dir/*` = any file in that folder, new files included). Never edit `Files:`, `docs/ISSUES.md` or other docs.
- A change you need in a file you don't own: write it exactly in `## Worker notes` under "Wiring notes for Claude".
- Files under ~300 lines. Every number in game code: `// data: <file> <key>`, `// godot: File.cs:line`, or `// new: DECISIONS 2026-11-22` (pure layout/tuning).
- Rust lanes: add tests for the pure logic. Run `tools/gate.sh --task tasks/<your task>.md` (never inside flock, never end your turn while it runs) until the last line is `GATE: PASS`.
- Blender lanes: no cargo. Build with `flock /tmp/rotland-blender.lock blender -b --python <script>`; look at the preview PNG yourself; check scope with `python3 -I tools/gate_checks.py --base main --task tasks/<your task>.md` (`scope: PASS`).
- DO NOT touch `tools/blender/buildings/settlement.py`, `assets/models/buildings/settlement.glb`, `tools/blender/vehicles/car_parts.py`, or `assets/models/vehicles/car_hatchback_*` (someone has unsaved work there).
- Don't stop to "wait": keep going until done. Fill `## Worker notes` (what changed, how to check it in the game), commit once to your branch.
