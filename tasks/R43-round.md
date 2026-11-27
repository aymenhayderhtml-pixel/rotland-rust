# R43 round: New Halden climbing. Lead file for the gate

Files: world/*, game_zombie/*, game_core/*, game_world/*, game/*, rotland/*, rotland_ui/*, engine/*, physics/*, data/*, tools/blender/*, tools/README_TOOLS.md, assets/models/*, assets/icons/*, assets/textures/*, assets/models.pack, docs/WORLD.md, tasks/R43-*
Rules: `tasks/LEAD.md` + `tasks/R43-common.md` (what is true today, owner decisions, "R43 shared names": follow them EXACTLY).

## Phase 0 (lead)
- FPS before: `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland` -> `/tmp/r43/perf_before.txt` (metro ONLY).
- Shots before: `--metro=core`, `--metro=top`, `--metro=landmark` (hour 10) -> `/tmp/r43/before_*.png`.
- `assets/models.pack` is not in git: bake once in the worktree before the first run.

## Tasks (9)
| Task | What | after |
|---|---|---|
| `R43-01-climb-plan.md` | climbable towers, stair modules, stop floors, hatch, mast, collider boxes | none (writes `roof_deck_y` + types first) |
| `R43-02-rooftops-fire-escapes.md` | plank/ladder links between roofs, fire-escape tower picks + ladders | imports R43-01 types |
| `R43-03-stair-models.md` | Blender: stair modules, digits, signs, ladder segment, hatch; `build_r43_models.py` | none |
| `R43-04-floor-interiors.md` | Blender: office / flat / camp floors (24 + 32), `data/climb_floors.json` | none |
| `R43-05-rooftop-kit.md` | Blender: planks, step crate, radio console; brick fire-escape colliders + ladder empties | none |
| `R43-06-floor-zombies-loot.md` | poi.json entries, `wakeHeightM`, 4 loot tables | none |
| `R43-07-radio-goal.md` | radio logic + data + play glue + save | none |
| `R43-08-wiring.md` | contents, hulls, ladders, fire-escape colliders, radio hookup | after 01-07 |
| `R43-09-flags-map.md` | 7 flags, map labels, docs | after 08 |

Start 01-07 at once (7 subagents). Tell R43-02 that R43-01 writes its types first. Tell R43-03/04/05 the builder names (`interiors.build_all()`, `rooftop_kit.build_all()`). Start 08 when 01-07 have reported, then 09.

## Checks after the gate (`target/play/rotland`, `--save-dir=/tmp/r43-saves`, pictures to `/tmp/r43/`, hour 10)
- **Each pose:** `--metro=climb`, `climb_stairs`, `climb_floor`, `climb_roof`, `roof_planks`, `fire_escape`, `radio`. Look at every picture:
  - The stairwell reads as a dirty concrete stairwell (rails, standpipe, floor number, jammed doors), not a grey box.
  - Each stop floor reads as an office / apartments / a last stand.
  - The roof view shows the city skyline.
- **Walk up the stairs:** from `climb_stairs`, a 4-second `--autowalk` run. Use the stats: the player's y rises about 1.9 m (flight A), they are on the floor at the end, and they are not stuck or falling. Then the same from the floor landing of a stop floor through the open door into the floor.
- **No falling through:**
  - From `climb_floor`, `--autowalk` 4 s across the floor: `playerOnFloor` true, y unchanged.
  - Run every pose for 300 frames: no pose ends with the player falling (y far below the start).
- **The roof hatch:** from the top floor at the hatch, `--climb` (it grabs the nearest ladder) + `--frames=600`: the player ends on the roof deck (y = the deck).
- **Planks:** from `roof_planks`, `--autowalk` 5 s: the player crosses to the other roof (the stats position is on the far roof, y = its deck or the plank).
- **Fire escape:** from `fire_escape`, walk to the drop ladder + `--climb`: on the first landing. Then the shot from a landing up the zig-zag.
- **Radio:**
  - At `radio`, `--hold-e --frames=400`: the message lines show (a shot mid-message) and the crate is open/unlocked.
  - A second run with the same save: "Only static now."
- **Floor zombies:**
  - `climb_floor` + `--shot-frame=900`: zombies awake on THAT floor (stats `poiZombiesAwake` > 0, nearest zombie under ~15 m, at the floor's y).
  - `climb` (the lobby) + `--shot-frame=900`: the floors above do NOT wake (`poiZombiesAwake` 0 unless a lobby-level POI).
- **The map** (`--map`) shows "Radio Mast" and the "Stairs" labels.
- Nothing z-fights (digits, signs, the hatch plane); no props floating or sunk; the ladder segments line up.
- **FPS after:** `--set metro` -> `/tmp/r43/perf_after.txt`. Pass = **average at least 20**; write the 1% low for information. If it is under 20, lower the stair module tris first; never strip the city.

## Lead notes

## Gate report
