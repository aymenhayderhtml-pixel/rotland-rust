# R13 common rules (city, day 4). Read this, then your own task file.

Spec: `docs/CITY_SPEC.md` (the ruined city "Ashford"). Rules: `AGENTS.md`.

## Everyone
- Work only on the files in your task's `Files:` line. Never edit `build_buildings.py`, `build_lods.py`, `build_all.py`, `palette.py`, `common.py`, `kit.py`, or any file another lane owns. Claude adds your models to the build lists at merge.
- **Never end your turn while a command is still running.** Every command here takes under ~3 minutes; wait for its output in the same turn. If a tool returns before a command finishes, poll it until it ends. A turn that ends with "I'll wait" counts as a failed task.
- When done: fill `## Worker notes` in your task file (what you made, the triangle counts the script printed, known issues), then `git add` only your files and commit once to your branch. Never push, never touch `main`.

## Blender lanes (buildings, props)
- Run Blender ONLY like this (one Blender at a time across all lanes):
  `flock /tmp/rotland-blender.lock blender -b --python tools/blender/<folder>/<name>.py`
- Copy the style of the existing scripts: buildings use `kit.Kit` (see `tools/blender/buildings/house_small.py`, `gas_station.py`, `shed.py`); props use `common` (see `tools/blender/props/car_wreck.py`, `road_sign.py`). Use only color names that already exist in `tools/blender/palette.py`. If you need a color that isn't there, use the closest one and write it in Worker notes.
- Each model script must print the triangle count and return 0 only if it is within its budget (the existing `common.run_model(NAME, CATEGORY, BUDGET, build, ...)` does this).
- **LOD1** (buildings only): a second script `<name>_lod1.py` that writes model `<name>_lod1`: same footprint, same outline and roof shape, same main colors, no interior, no window frames, no small details. The game switches to it automatically by name.
- **No coplanar faces:** windows, frames, signs, stripes and any detail on a wall must stick out at least 2 cm from the wall surface (never end exactly on the wall face), or the game flickers (z-fighting; Blender previews don't show it).
- Look at the preview PNG that the script writes into `assets/models/_previews/` (open it as an image). Fix anything that looks broken: floating parts, holes, inside-out faces, wrong scale (a door is about 1 m wide and 2.1 m tall; one floor is about 3 m).
- Buildings need the same markers as today's buildings: a `lot_size` empty (scale = lot width x depth), `loot_N` empties (inside, on the floor), `spawn_N` empties (zombie spots), and colliders for walls like `kit.solid`/`kit.wall` make. Doors: openings in walls, about 1.2 m wide, so the player can walk in.
- Check your work with: `python3 -I tools/gate_checks.py --base main --task tasks/<your task>.md` (a fast scope check; it must print `scope: PASS`). You do NOT run `tools/gate.sh` or cargo.

## Rust/data lanes
- Run `tools/gate.sh --task tasks/<your task>.md` and fix until the last line is `GATE: PASS`. It can wait in a queue for the lock for a few minutes. Wait for it.
- Every number in `world/src` or `game/src` code (not in tests) needs a tag on its line: `// data: city.json <key>` (preferred: put the number in the JSON) or a math exemption (see `tools/README_TOOLS.md`).
