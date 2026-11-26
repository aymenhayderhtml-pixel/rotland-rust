# R39-04: Deco style, four landmark crowns, shared roof parts (Blender)

Owns: tools/blender/towers/deco.py, tools/blender/towers/landmarks.py, tools/blender/towers/roof_parts.py (shared roof kit: water tanks, AC units, vents, antennas, used by every crown), assets/models/buildings/tw_deco_*.glb, tw_landmark_*.glb (each with `_lod1`), assets/models/_previews/tw_deco_*.png, tw_landmark_*.png. Not owned: `tools/blender/build_r39_models.py` and `towers/shell.py` (R39-05), the other style scripts (R39-02, 03), palette.py (lead), Rust files. Shared lines for the lead: none. Merge order: R39-05 `shell.py` and this task's `roof_parts.py` before R39-02 and R39-03 build (they import roof_parts).
Read: tasks/R39-05-lobbies.md ("Shared kit rules", "Palette"), tools/blender/common.py (`Model` with `add_part`, `top_preview`, `grip`; `run_model`; `make_frustum`, `make_box`, `make_icosphere`, `add_quad`, `combine`), tools/blender/buildings/kit.py (`Kit.box`, `solid`, `collider`, `wall`, `roof`, `ridge_cap`, `chimney`, `frame`, `FLOOR_TOP`, `FRONT_VIEW`), tools/blender/buildings/office.py (pre-war stone and cornice language), tools/blender/buildings/water_tower.py (existing tank), tools/blender/buildings/radio_tower.py (existing mast, the closest to `tw_landmark_mast`), tools/blender/metro/jam_util.py (`pbox`, `bevel_bm`).

Goal: the Deco tower (dressed stone, vertical piers, chevron bands, setbacks), the four landmarks that read from far away, and the roof kit that R39-02 and R39-03 crowns also use.

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

## Models and sizes
| Model | Size | Height | Notes |
|---|---|---|---|
| tw_deco_24_base, _floor, _floor_b, _crown (+ _lod1 each) | 24 m | base 7.6, floor 3.8, crown 5.5 | normal Deco tower |
| tw_deco_32_base, _floor, _floor_b, _crown (+ _lod1 each) | 32 m | same | big Deco tower |
| tw_landmark_spire | 24 m crown footprint | crown 18 m | ribbed spire, finial |
| tw_landmark_mast | 24 m | crown 30 m | the tallest: lattice mast, platform, radio hut, cable runs (R43 goal) |
| tw_landmark_stepped | 24 m | crown 12 m | ziggurat cap of three steps |
| tw_landmark_glass_top | 24 m | crown 10 m | glass crown, shattered |
Setback sections use `tw_deco_24_floor` / `_floor_b` on top of a 32 m Deco tower (no extra ids). Landmark crowns sit on top of a 24 m setback section, so their footprint is 24 m. Crowns are 24 m square in footprint, origin at ground centre (`grip=(0,0,0)`), z 0 at the bottom of the crown.

## Deco look (`deco.py`)
- Dressed stone (`limestone`, `limestone_dark`) with vertical piers every bay (a bay is 4 m) that stick out 0.25 m, and recessed window bays between them.
- Chevron spandrels: a zig-zag band of 3 to 4 stacked boxes between floors, above each window row, `limestone_dark` and `mortar`.
- Floor modules: windows with stone sills and lintels; dead glass (`glass_dark`, some `soot`), a few boarded (`kit.boards`), some cracked (thin diagonal boxes). `floor_b` has a different broken pattern.
- Base: the lobby shell from R39-05 (`lobby_shell(k, size, "limestone")`) plus a stone arch round the front door (`kit.frame` plus a stepped arch of boxes).
- Crown: stepped parapet with a cornice of 3 ledges, a ribbed roof, one water tank and two AC units from `roof_parts.py`, two antenna stubs. Height exactly 5.5 m.
- Quality: every floor module has piers, chevrons, sills and lintels in every bay on the front and sides. The back face may be simpler. If over budget, reduce the back face first, never the street face.

## Landmarks (`landmarks.py`)
- `tw_landmark_spire`: a 24 m base stone plinth (1.5 m high) with 8 tapering ribs up to 15.5 m, a 2.5 m spire, a finial and two lamp rings. Dead windows on the ribs. 18 m total.
- `tw_landmark_mast`: four square lattice legs (`metal`, struts every 1.5 m) tapering to a mast, a 6 x 6 m platform at 18 m with railings, a radio hut (box with a flat roof, `rock_mid`, a door), a dish, and 4 cable runs from the platform to the crown base (thin boxes). Radio hut is `add_part("radio_hut", bm)` (separate node, R43 goal). 30 m total.
- `tw_landmark_stepped`: three stepped boxes 24, 18, 12 m wide, each with a cornice, a small spire on top. 12 m total.
- `tw_landmark_glass_top`: a stone ring, then a glass crown (`glass_tint`, `glass_dark`) with the top third missing: jagged shards as thin boxes, bent steel frames (`metal`, `rust`). 10 m total.
- Mast, stepped and glass top each get one `water_tank` from roof_parts at their base; the spire gets none.

## Shared roof kit (`roof_parts.py`, used by R39-02 and R39-03 crowns)
- `water_tank(k, x, y, z0, r=1.2, h=2.4)`: wooden tank (`bark_light`) on 4 steel legs (`metal`), conical roof (`bark_dark`), hoops (`metal`). Dead, with a broken hoop.
- `ac_unit(k, x, y, z0)`: box with a grille (`rock_dark`), a fan ring, two rusty corner posts (`rust`).
- `vent_stack(k, x, y, z0, h)`: pipe (`metal`) with a cap and a bend.
- `antenna(k, x, y, z0, h)`: thin mast with two cross bars.
- `roof_deck(k, size, z, color)`: roof slab over the footprint, 0.2 m thick, with a parapet.
- Each function takes a `kit.Kit` and adds visual parts only (roof parts have no colliders).

## Budgets and checks
- Check the preview of every model, and the landmark crowns from a far view (they must read as a silhouette: name, shape, dead top).
- Stack check: the tw_deco_24 and tw_deco_32 stack previews from `stack_preview` (R39-05) show no gap; the 24 m setback on the 32 m tower also has no gap.

## Done when
- The 16 Deco glbs, the 8 landmark glbs and their LOD1s exist and are within budget; roof_parts loads without error when R39-02 or R39-03 import it.
- The landmark mast has a platform, a radio hut and cable runs, visible in its preview; the spire reads as a spire, the glass top has shards.
- The Deco stack preview shows piers and chevrons on the whole height, no gap.

## Worker notes
- Implemented `tools/blender/towers/roof_parts.py`: `water_tank`, `ac_unit`, `vent_stack`, `antenna`, `roof_deck` (all visual-only).
- Implemented `tools/blender/towers/deco.py`: `tw_deco_24_{base,floor,floor_b,crown}` and `tw_deco_32_{base,floor,floor_b,crown}` (+ `_lod1` each, 16 `.glb`s total). Features 4 m vertical piers, chevron spandrels, stone sills/lintels, deterministic broken/boarded/cracked windows, stepped Art Deco front portal arch on `lobby_shell`, and 5.5 m crown with `roof_parts`.
- Implemented `tools/blender/towers/landmarks.py`: `tw_landmark_{spire,mast,stepped,glass_top}` (+ `_lod1` each, 8 `.glb`s total, 24 m footprint). `tw_landmark_mast` exports `radio_hut` as a separate node via `add_part("radio_hut", bm)`, has 4 lattice legs, 6x6 m platform with railings, dish, and 4 cable runs.
- Ran `deco.py` and `landmarks.py` under `flock /tmp/rotland-blender.lock`: 16/16 Deco OK, 8/8 Landmarks OK (0 Tracebacks, all well within triangle budgets). Inspected previews for Deco base/base_top/floor/crown (24 & 32 m) and all 4 landmarks.
- Shared-file lines: none. Note: `stack_preview.py` was not run here (owned by R39-05 / `build_r39_models.py`).
