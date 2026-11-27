# R43-03: Stairwell models + stair extras (+ the round's Blender build script)

Owns:
- `tools/blender/towers/stairs.py` (NEW: stair modules), `tools/blender/towers/stair_extras.py` (NEW: digits, signs, ladder segment, hatch)
- `tools/blender/build_r43_models.py` (NEW: builds everything of R43-03/04/05, ends with `R43 BUILD OK`)
- `assets/models/buildings/tw_stair_*.glb`, `assets/models/props/{stair_digit_*,stair_sign_roof,roof_ladder_seg,roof_hatch}.glb` (+ previews)

Not owned: interiors (R43-04), rooftop kit / brick fire escapes (R43-05; their builders are imported by your build script: agree the function names in Worker notes: `interiors.build_all()`, `rooftop_kit.build_all()`).
Read: tasks/R43-common.md (the shaft frame and stair module numbers: follow them EXACTLY), `tools/blender/common.py` (`add_collider`, `add_empty`, palette), `tools/blender/towers/shell.py` + `lobby_props*.py` (the lobby look, the base floor height), `tools/blender/towers/roof_parts.py`, `tools/blender/build_r42_models.py` (script style).

## Stair modules (Blender: origin = shaft centre on the floor landing; game -Z = Blender +Y front)
All share one builder with options. 3.8 m tall, outer 4.4 x 6.4 m, walls 0.2 m.
- **Geometry:**
  - The floor landing, flight A, the mid landing and flight B exactly at the common numbers.
  - Real visual steps (about 0.19 m risers) with nosing.
  - Steel handrails on the open side of each flight + a wall rail.
  - A landing slab underside, so looking up shows concrete, not void.
- **Walls:**
  - Painted concrete inside: two-tone, a stripe at 1.2 m.
  - Scuffs, cracked plaster, a fire hose cabinet on some.
  - An emergency light box; the pipes of a standpipe running up one corner (continuous module to module).
- **Door (-Z wall, x -0.7..+0.7, 2.2 m):**
  - `tw_stair_flight`: a dented steel door shut in its frame, a chair/plank wedged against it, debris.
  - `tw_stair_flight_b`: the same + rubble on flight A's lower steps (walkable: keep it below 0.1 m on the ramp line, or just beside it), a blood smear, a dropped backpack, papers.
  - `tw_stair_flight_z`:
    - as `_b`, + a body on the mid landing + `spawn_1` (floor landing) and `spawn_2` (mid landing) empties, 0.1 m above each landing
    - no lod1 is needed if the POI path doesn't require one (check `rotland/src/props.rs` `PropLibrary::get` for buildings and say)
  - `tw_stair_open`: the door swung open 100 degrees, a green "EXIT" sign box above it, a wedge.
  - `tw_stair_top`: the floor landing + open door + a ceiling slab at 3.6-3.8 m, no flights. A painted "ROOF ACCESS ->" arrow pointing at the hatch side (+X).
- **Colliders** (`add_collider`):
  - 4 walls, with the door gap open on `_open`/`_top` and closed on the jammed ones
  - the floor landing, the mid landing
  - ONE ramp per flight: a thin wedge slab under the step noses, its top surface on the line from (y 0) to (y 1.9), so the player walks up smoothly
  - the jammed door
- **Budgets:** about 6-9k tris per module (they repeat up to 40 times, so this is what keeps FPS); 2 materials max from the palette.
- **Previews:**
  - A cut-away (one side wall hidden) of each variant.
  - Plus a stack preview of 3 `tw_stair_flight` modules: the standpipe and the rails line up, and flight B of the lower module meets the next module's floor landing with no step or gap.

## Stair extras
- `stair_digit_0..9`: stencil-painted digits, 0.35 m tall, as a flat 0.01 m plate. No collider; under 100 tris each.
- `stair_sign_roof`:
  - A plywood board on two legs (or nailed to a post), hand-painted "ROOF" with a big arrow up, 1.3 m tall.
  - A spray-painted arrow plate the code can put on the lobby wall too. If that's cheap, make it a second model `stair_arrow_spray` and say so.
- `roof_ladder_seg`: a steel ladder segment, 1.0 m tall, 0.5 m wide, rungs every 0.3 m, wall brackets. It tiles vertically with no visible seam. No collider (ladders are climbed through the rail).
- `roof_hatch`:
  - A raised hatch curb (1.4 x 1.4 m, 0.5 m tall) with the lid flipped open.
  - Inside the curb, a dark plane just above the deck, so the hole reads as open.
  - Colliders only on the curb's 4 sides, leaving the inner 1.2 x 1.2 m free.

## Build script (`build_r43_models.py`)
- Calls your builders + `interiors.build_all()` (R43-04) + `rooftop_kit.build_all()` (R43-05) + the brick tower rebuild that R43-05 names.
- 0 Traceback; ends with the line `R43 BUILD OK`.
- Run only under `flock /tmp/rotland-blender.lock`.

## Done when
The models with good previews (look at them!); the build script runs clean for your part. Fill `## Worker notes`: model list, tris per model, the base floor height offset the lobby module must use.

## Worker notes
