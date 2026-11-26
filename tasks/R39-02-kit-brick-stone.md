# R39-02: Brick and Stone tower kit (Blender)

Owns: tools/blender/towers/brick.py, tools/blender/towers/stone.py (and brick_parts.py / stone_parts.py only if a script passes ~300 lines), assets/models/buildings/tw_brick_16_*.glb, tw_brick_24_*.glb, tw_stone_24_*.glb, tw_stone_32_*.glb (base, floor, floor_b, crown, each with `_lod1`), and their previews in assets/models/_previews/ (`tw_brick_*.png`, `tw_stone_*.png`). Not owned: `towers/shell.py`, `towers/roof_parts.py`, `towers/stack_preview.py`, `build_r39_models.py` (R39-04/05), the other style scripts (R39-03/04), palette.py (lead), Rust files. Shared lines for the lead: none.
Read: the rules block below, tools/blender/common.py (`Model` with `add`, `add_collider`, `add_part`, `top_preview`, `grip`; `run_model`; `make_box`, `make_frustum`, `paint_faces`), tools/blender/buildings/kit.py (`Kit.box`, `solid`, `collider`, `wall`, `frame`, `boards`, `roof`, `ridge_cap`, `posts`, `FLOOR_TOP`, `FRONT_VIEW`), tools/blender/buildings/apartment.py (belt courses, window bays, stoop and the rusty fire escape: the closest existing look), tools/blender/buildings/office.py (pre-war base, storefront, rubble), tools/blender/metro/jam_util.py (`pbox`, `bevel_bm`), tools/blender/palette.py (`COLORS`, `COLORS_C`), tasks/R39-04-kit-deco-landmarks.md (`roof_parts.py`: `water_tank`, `ac_unit`, `vent_stack`, `antenna`, `roof_deck`).

Goal: two dead New York facades. Brick is the brownstone and tenement (Edge and Midtown). Stone is the pre-war office (Midtown and Core). Both must be dense and dramatic close up: no plain boxes.

## Rules copied from R39-05 (keep identical in R39-02, 03, 04)
- Footprint `size` x `size` (16, 24 or 32 m) centred on X=0, Y=0. Z=0 is the ground. Front = +Y = the street. Run every tower with `grip=(0.0, 0.0, 0.0)` so the origin stays at the footprint centre; overhangs (+Y only, at most 1.2 m) must not move it.
- Exact heights: floor module z 0..3.8; base z 0..7.6 (2 floors); crown z 0..`crown_height` (style value). Each module has slab bands at its top and bottom edges, so stacks have no gap.
- Floor modules and crowns are visual only (no colliders). Base colliders come from the lobby shell and its props only. R39-06 adds the roof box.
- LOD1 (`<name>_lod1`): same silhouette and colours, plain boxes, no door, no colliders, about 1/10 of LOD0 triangles.
- Facade windows are visual decals (dark boxes 0.02 m proud). Only the door gap is a real hole.
- Budgets LOD0 / LOD1: floor 16 m 3000 / 300; floor 24 m 4500 / 450; floor 32 m 6000 / 600 (`floor_b` = `floor`); base 15000 / 1500; crown 8000 / 800; landmark crown 25000 / 2500.
- Lobby shell: `towers/shell.py` `lobby_shell(k, size, facade_color)`, written by R39-05 (merge it before you build). Walls 0.3 m thick; floor slab z 0..0.1; front door gap 2.8 m wide x 3.0 m tall (x -1.4..1.4, z 0.1..3.1) on +Y, so the lobby is walkable; open hall (no interior walls), ceiling closed at z 7.3..7.6; closed stair door on the -Y back wall, 1.2 x 2.2 m (`add_part("stair_door")` plus collider `stair_door`). Do not use `kit.DOOR_H` (2.2 m) for the front door.
- Each style script has `build_all()` returning `[(name, ok)]` and a `__main__` block. Palette names: only those in palette.py, plus R39-05's "Palette" list once the lead appends it.
- Blender runs: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/towers/<style>.py`. Look at every preview. Stack check: `tools/blender/towers/stack_preview.py` (R39-05).

## Styles (proposed, goes into `data/metro.json` styles through R39-01)
| style | sizes | floor_height | base_height | crown_height |
|---|---|---|---|---|
| brick | 16, 24 | 3.8 | 7.6 | 3.5 |
| stone | 24, 32 | 3.8 | 7.6 | 5.0 |

## Models (each: base, floor, floor_b, crown, and `_lod1` of each)
- `tw_brick_16_*` and `tw_brick_24_*`, `tw_stone_24_*` and `tw_stone_32_*`. Budgets: see the rules block.
- 16 m = 4 bays per face, 24 m = 6 bays, 32 m = 8 bays (a bay is 4 m).

## Brick (`brick.py`): brownstone and tenement
- Facade `brick_red`, piers between bays in `brick_dark`, base and window surrounds in `brownstone`. Mortar lines (`mortar`) on the lintels and cornice only, not the whole wall.
- Stone lintels (`limestone`) over each window, stone sills, a projecting cornice (0.6 m, `limestone`) at the crown.
- Iron fire escape on the front (+Y): per floor a 1.2 m landing with a rail (`metal`), zig-zag stairs (diagonal boxes) between landings, one escape on a side face. Overhang on +Y at most 1.2 m.
- Some windows hold a window AC unit (box 0.6 x 0.5 x 0.5 m, `metal`, `rock_dark` grille). Some are boarded (`kit.boards`, `bark_light`), some are empty holes (`soot` fill), some have a cracked pane (a thin `glass_dark` diagonal). Rust streaks (`rust_dark`) run below sills.
- Base: lobby shell with brick facade, a 3-step `brownstone` stoop up to the door (the stoop sits on +Y, within 1.2 m), a stone arch frame over the door (`kit.frame` plus boxes).
- Crown (3.5 m): parapet of brick with a cornice, a water tank and one AC unit (roof_parts), a vent stack. The crown has no door or windows.
- `floor_b`: a different broken-window pattern (other bays boarded, a fire escape landing missing).

## Stone (`stone.py`): pre-war office
- Facade `limestone`, rusticated base floor (`limestone_dark` grooves: a joint line every 0.6 m, deep 0.05 m), pilasters (0.4 m proud, full height) between bays, window frames in `limestone_dark`.
- Windows: `glass_tint` panes with a grey mullion, some `glass_dark` (dead), some boarded, some with a crack. Balcony rails on a few floors (`metal`).
- Base: lobby shell with a limestone frame, two columns (`limestone`) flanking the door, an entablature over the door gap (no part inside the gap). Broken glass doors: frame only, no glass collider.
- Crown (5.0 m): heavy cornice with dentils (a row of small boxes), a balustrade (`limestone`), two flagpoles on the corners (thin `metal` poles, a torn `red_paint` flag), a water tank and a vent stack from roof_parts.
- `floor_b`: broken pattern (missing pilaster top, a cornice piece fallen, one boarded row).

## Quality and checks
- Every floor module: in every bay, window with frame, sill and lintel (brick) or frame and mullions (stone), on the front and sides. The back face may be simpler. If over budget, reduce the back face first, never the street face.
- Look at each preview. Look at the stack previews (base + 3 floors + crown) from R39-05: no gap, no floating part, and the cornice reads.
- Do not use `kit.DOOR_H` for the front door; use the lobby shell.

## Done when
- The 16 brick and 16 stone glbs (8 models per size: base, floor, floor_b, crown + LOD1) exist within budget; the build log has no `Traceback` and no `FAILED`.
- Brick reads as brownstone (lintels, stoop, fire escape, AC units, boards) and stone as an office (cornice, rustication, flagpoles) in close previews.
- Stack previews for brick 16 and 24 and stone 24 and 32 show no gap.

## Worker notes
