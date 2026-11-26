# R40-05: Survivor trace models (Blender: car walls, graffiti, posters, rooftop camp, lobby dressing)

Owns: tools/blender/traces/ (new folder: one script per group, `graffiti_font.py` (own copy of the letters, see below), `traces.py` with `build_all()`), tools/blender/build_r40_models.py (NEW: the shared build script; its GROUPS list starts with `("traces", traces.build_all)`; other R40 tasks add one line each), assets/models/props/ (only the new `car_wall_stack`, `graffiti_*`, `poster_wall`, `rooftop_camp`, `boarded_door`, `sandbags_line`, `body_sheet_b`, `lobby_column`, `lobby_ceiling_hang`, `lobby_light_broken`, `lobby_rug`, `lobby_chair_tipped`, `lobby_debris` .glb files), assets/models/_previews/ (only their PNGs). Not owned: `tools/blender/metro/jam_font.py` (read it, do not edit it), `tools/blender/palette.py` (lead), every other file.
Read: tasks/R40-common.md ("R40 shared names", survivor trace bullets, "Owner decisions" traces list), docs/ART_PIPELINE.md, tools/blender/common.py (`Model`, `add`, `add_collider`, `add_part`, `finalize_and_export`, `make_box`, `paint_faces`, `combine`, `transform`, `run_model`), tools/blender/metro/jam_font.py (`FONT`, `text_bms(text, centre_x, base_z, letter_w, letter_h, y, color, depth, bevel)`: the gantry lettering the R38 signs use), tools/blender/metro/jam_util.py (`pbox`, `bevel_bm`, `bar`, `box`, `add_bbox_collider`), tools/blender/metro/jam_cars.py and jam_car_shell.py (car shells for `car_wall_stack`: reuse the builder, do not edit it), tools/blender/metro/jam_props_bodies.py (`_sheet`: the R38 body under a sheet; `body_sheet_b` is a second pose of it), tools/blender/props/barricade.py (`_make_sandbag`), tools/blender/towers/roof_parts.py (`water_tank`, `vent_stack`, `antenna`), tools/blender/towers/lobby_props.py (`build_desk`: how a lobby prop is built), tools/blender/towers/shell.py (`front_face_y`: lobby front wall position, for the scale of lobby pieces).

## Origin and orientation (all models)
- Upright pieces (car wall, rooftop camp, doors, columns, body): origin at the bottom centre, Z=0 is the ground or floor, front = +Y.
- Wall decals and posters (`graffiti_*`, `poster_wall`): flat sheets in the XZ plane, 2-3 cm off the wall at +Y, origin at the bottom centre on the wall, so R40-06 can place them flat on a facade with the facade yaw.
- Lobby pieces: Z=0 is the lobby floor (the 0.1 m raised slab is not part of them).

## Blender run and checks
- Run: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r40_models.py`. The subagent MAY run Blender this way (owner decision for R40). Read the log: no `Traceback`, no `FAILED`.
- Look at EVERY preview (3/4 view; `_top` for the rooftop camp). Fix anything that reads as a box, a floating piece or a flat grey blob.

## Models and shapes
- **car_wall_stack** (props; 8000): 2-3 wrecks stacked (2 on the ground, 1 on top, tilted), joined by planks (`wood_crate`, `bark_light`) and sheet metal (`metal`, `rust_dark`). Width 10-12 m across the street, height about 3.5 m, depth 4 m. One side has a ladder (`bark_light` rungs on two rails, 0.3 m spacing) up to the top of the wall, the other side is closed. Origin: bottom centre of the wall. Colliders: one convex `wall` box covering the full wall (solid, blocks the street), and one convex `ramp` wedge on the ladder side (walkable slope up to the top, no steeper than 40 degrees). Everything else is visual.
- **graffiti_dead_inside, graffiti_turn_back, graffiti_x, graffiti_arrow, graffiti_no_food, graffiti_went_north** (props; 300 each): flat decal planes, 2 cm off the wall (y +0.02 to +0.03, the front is +Y), origin at the bottom centre of the sheet on the wall. Letters use `red_paint` (main), `white` (outline strokes), `soot` (drips) via `text_bms` from a local copy of the font: `tools/blender/traces/graffiti_font.py` copies `jam_font.FONT` and ADDS the missing letters `B`, `F`, `K`, `U` (strokes in the same unit cell). Text: "DEAD INSIDE", "TURN BACK", "NO FOOD", "WE WENT NORTH". `graffiti_x` is a big X (two bars), `graffiti_arrow` is an arrow with a head. Letter height 0.5-0.8 m, sheet about 4-6 m wide. Add 2-3 drips (thin `soot` boxes) under the letters. Decals have NO colliders.
- **poster_wall** (props; 1500): a cluster of 4-6 missing-person posters on a 2 x 1.5 m board: flat sheets (`white` / `concrete_grey` paper), each with a grey photo rectangle showing a generic head-and-shoulders silhouette (no real faces, no names, just a "MISSING" word from the font in `red_paint`), 2 cm off the board. Origin bottom centre. No colliders.
- **rooftop_camp** (props; 6000): one 6 x 6 m model: a tent (`canvas_khaki` / `tarp_blue`, a ridge pole), a tarp lean-to (`tarp_blue`) over a mattress (`white` with `bark_light` frame), 3-4 barrels (`red_paint`, `metal`), a washing line (two posts, thin `leather` strands) with 3 cloth pieces (`shirt_red`, `shirt_blue`, `white`), a fire barrel (`metal` with `soot` inside) and a stack of crates (`wood_crate`). Origin bottom centre. Collider: one convex `tent` for the tent only (the player can walk around the rest).
- **boarded_door** (props; 1500): a door frame 1.2 m wide, 2.2 m tall, with 3 crossed planks (`bark_light`, `wood_crate`) and 2 nails (`metal`), frame `bark_dark`, 0.1 m proud of a wall. Collider: one `board` box (solid).
- **sandbags_line** (props; 1500): a row of 8-10 sandbags (from `barricade._make_sandbag` style, `canvas_khaki`), two rows high, 4 m long. Colliders: one `row` box (solid, low, 0.8 m tall).
- **body_sheet_b** (props; 8000): a second pose of the R38 body under a sheet (`_sheet` in jam_props_bodies.py: arms out, the sheet folded differently, one shoe out, a dark stain `blood_dark` under the head). Same origin and collider style as the R38 body (read `jam_props_bodies.py`).
- **Lobby dressing** (props, all with origin at the bottom centre; interior pieces, used inside tower lobbies, so they fit a 3 m hall):
  - `lobby_column` (1500): a square column 0.6 x 0.6 m, 6.8 m tall (the hall is 7.3 m high), `limestone` / `concrete_grey` with a chipped corner and a cap. Collider: one `col` box (solid).
  - `lobby_ceiling_hang` (1500): a sagging ceiling panel 2.5 x 1.2 m hanging by 2 wires, `concrete_grey` with `rust_dark` stains. Visual only, no collider.
  - `lobby_light_broken` (800): a cluster of 2 broken light fixtures on wires with `glass_dark` shards. Visual only.
  - `lobby_rug` (400): a flat 2 x 1.4 m rug, `red_paint`/`brick_dark` pattern, 1 cm thick. No collider.
  - `lobby_chair_tipped` (1000): one chair tipped on its side (`wood_crate`, `leather` seat). Collider: none (it is small).
  - `lobby_debris` (2000): a pile of 15-25 pieces (plaster chunks `concrete_grey`, glass `glass_dark`, a book `white`, a shoe `shoe_dark`). Visual only.

## Budgets and tags
- Triangles, LOD0 only (`Model` checks): car_wall_stack 8000, graffiti_* 300 each, poster_wall 1500, rooftop_camp 6000, boarded_door 1500, sandbags_line 1500, body_sheet_b 8000, lobby pieces as listed.
- Each number gets `# new: DECISIONS 2026-11-26` or `# data: budgets.json props`.

## Done when
- All 18 listed models exist as `assets/models/props/<name>.glb` (6 graffiti, 12 others), within budget, with no `Traceback` or `FAILED`.
- Graffiti letters read clearly from the street in the preview; car wall reads as stacked wrecks with a ladder; the rooftop camp reads as a camp; the lobby pieces fit a hall.
- `build_r40_models.py` runs all groups; other R40 tasks add one line each to its GROUPS list.

## Worker notes

- **Files created / changed:**
  - `tools/blender/traces/graffiti_font.py` (copy of `jam_font.FONT` plus `B`, `F`, `G`, `K`, `M`, `U`, with 2-tri `+Y`-facing `quad_bar` option in `text_bms` to keep decals under 300 tris)
  - `tools/blender/traces/decals.py` (`graffiti_dead_inside`, `graffiti_turn_back`, `graffiti_no_food`, `graffiti_went_north`, `graffiti_x`, `graffiti_arrow`, `poster_wall`)
  - `tools/blender/traces/car_wall.py` (`car_wall_stack`, reusing `jam_cars.CARS` and `jam_car_shell` with solid-side `strip_doors=True` on two wrecks and 10-seg wheels)
  - `tools/blender/traces/camp.py` (`rooftop_camp`, `boarded_door`, `sandbags_line`, `body_sheet_b`)
  - `tools/blender/traces/lobby_dressing.py` (`lobby_column`, `lobby_ceiling_hang`, `lobby_light_broken`, `lobby_rug`, `lobby_chair_tipped`, `lobby_debris`)
  - `tools/blender/traces/traces.py` (`build_all()`)
  - `tools/blender/build_r40_models.py` (shared R40 build script with `GROUPS = [("traces", traces.build_all)]`)
  - 18 `.glb` files in `assets/models/props/` and 19 preview PNGs (including `rooftop_camp_top.png`) in `assets/models/_previews/`.
- **Blender run & triangle counts (`R40 BUILD OK`, zero `Traceback` or `FAILED`):**
  - `car_wall_stack`: 7814 / 8000 (`wall` solid box + `ramp` convex wedge at 31.1 deg slope on +X/+Y side)
  - `graffiti_dead_inside`: 162 / 300; `graffiti_turn_back`: 126 / 300; `graffiti_no_food`: 102 / 300; `graffiti_went_north`: 158 / 300; `graffiti_x`: 14 / 300; `graffiti_arrow`: 18 / 300 (all flat XZ planes at `y = +0.023..+0.028`, `grip=(0,0,0)` so `y=0` is the wall plane, no colliders)
  - `poster_wall`: 400 / 1500 (board at `y = +0.01..+0.025`, 5 posters at `y = +0.045..+0.053`, `grip=(0,0,0)`, no collider)
  - `rooftop_camp`: 1618 / 6000 (`tent` collider only)
  - `boarded_door`: 204 / 1500 (`grip=(0,0,0)` so frame sits 0.1 m proud of `y=0` wall, `board` box collider)
  - `sandbags_line`: 952 / 1500 (`row` box collider 0.8 m tall)
  - `body_sheet_b`: 6760 / 8000 (no collider, matching `jam_props_bodies._sheet`)
  - `lobby_column`: 616 / 1500 (`col` box collider 0.6 x 0.6 x 6.8 m)
  - `lobby_ceiling_hang`: 156 / 1500; `lobby_light_broken`: 136 / 800; `lobby_rug`: 72 / 400; `lobby_chair_tipped`: 312 / 1000; `lobby_debris`: 408 / 2000 (visual only, no colliders)
- **Wiring notes for lead / R40-06:**
  - `graffiti_*`, `poster_wall`, and `boarded_door` use `grip=(0, 0, 0)` so placing them at the wall face coordinate `(x, y, z)` with the facade yaw puts them 2-3 cm (or 0.1 m for the door frame) proud of the facade on `+Y`.
  - Other R40 Blender tasks (02, 04, 07) can append their `(group, module.build_all)` entry to `GROUPS` in `tools/blender/build_r40_models.py`.
  - All 19 preview PNGs in `assets/models/_previews/` were visually inspected; letters read cleanly left-to-right from `+Y` and no floating or flat grey blob issues remain.

