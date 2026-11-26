# R39-03: Glass and Slab tower kit (Blender)

Owns: tools/blender/towers/glass.py, tools/blender/towers/slab.py (and glass_parts.py / slab_parts.py only if a script passes ~300 lines), assets/models/buildings/tw_glass_24_*.glb, tw_glass_32_*.glb, tw_slab_24_*.glb, tw_slab_32_*.glb (base, floor, floor_b, crown, each with `_lod1`), and their previews in assets/models/_previews/ (`tw_glass_*.png`, `tw_slab_*.png`). Not owned: `towers/shell.py`, `towers/roof_parts.py`, `towers/stack_preview.py`, `build_r39_models.py` (R39-04/05), the other style scripts (R39-02/04), palette.py (lead), Rust files. Shared lines for the lead: none.
Read: the rules block below, tools/blender/common.py (`Model` with `add`, `add_collider`, `add_part`, `top_preview`, `grip`; `run_model`; `make_box`, `paint_faces`, `make_frustum`), tools/blender/buildings/kit.py (`Kit.box`, `solid`, `collider`, `wall`, `frame`, `boards`, `roof`, `ridge_cap`, `posts`, `FLOOR_TOP`, `FRONT_VIEW`), tools/blender/buildings/office.py (storefront glass panes, the glass band windows, broken glass), tools/blender/buildings/apartment.py (balcony-like belt courses and the front stair pattern), tools/blender/buildings/warehouse.py (large plain panel walls, for scale only), tools/blender/metro/jam_util.py (`pbox`, `bevel_bm`), tools/blender/palette.py (`COLORS`, `COLORS_C`), tasks/R39-04-kit-deco-landmarks.md (`roof_parts.py`: `vent_stack`, `antenna`, `ac_unit`, `roof_deck`).

Goal: the New York office curtain wall (Core) and the 70s concrete slab block (Midtown). Both must be dense and dead close up.

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
| glass | 24, 32 | 3.8 | 7.6 | 4.5 |
| slab | 24, 32 | 3.8 | 7.6 | 3.0 |

## Models (each: base, floor, floor_b, crown, and `_lod1` of each)
- `tw_glass_24_*`, `tw_glass_32_*`, `tw_slab_24_*`, `tw_slab_32_*`. Budgets: see the rules block.
- 24 m = 6 bays per face, 32 m = 8 bays (a bay is 4 m). Glass mullions every 1.5 m inside each bay.

## Glass (`glass.py`): curtain-wall office
- Frame: `metal` mullions (0.12 m) and transoms, `chrome` cap rails. Panes are recessed 0.1 m, `glass_tint` on most.
- Dead panes: about 20% `glass_dark` (dark, dirty), about 10% missing (the slot shows `soot`), some cracked (a thin dark diagonal box on the pane).
- Spandrel bands (`concrete_grey`, 0.9 m tall) between floors, with a `rust` drip under some.
- Floor bays on the front and sides: every pane is a separate part (not one glass sheet).
- Base: glass lobby. Revolving door frame (`metal`) in the door gap of the lobby shell; glass doors broken (frame only, no glass collider); a sign band (`chrome`, no words).
- Crown (4.5 m): a glass penthouse with bent mullions and two panes gone, a roof plant box (`metal`), `ac_unit`, `vent_stack` and `antenna` from roof_parts.
- `floor_b`: a different dead-pane pattern (other panes missing, a row of cracks).

## Slab (`slab.py`): 70s concrete block
- Facade `concrete_grey`, with `concrete_stain` streaks (thin vertical boxes, 0.04 m proud) running down from each balcony and window sill.
- Balconies: on about one bay in three, a slab 2.4 m wide x 1.2 m deep on +Y (within the 1.2 m overhang limit), `metal` rails, and junk: a bag, a bicycle (`metal` frame, `gun_black` tyres), a dead potted plant (`dry_brush`), a box, a `tarp_blue` tarp half off.
- Windows: concrete frames and strip windows (`glass_tint`), some boarded (`kit.boards`), some `soot` holes.
- Satellite dishes: a few on balconies and two on the crown (white disc, `metal` bracket).
- Base: concrete lobby with a rough portico (concrete slab roof on two square posts, outside the door gap), lobby shell from R39-05.
- Crown (3.0 m): flat roof with a `metal` rail, two `vent_stack`s, a lift motor room (box, `concrete_grey`, one door, one dead window), `ac_unit`.
- `floor_b`: a different balcony set (fewer balconies, one tarp removed, one stain pattern changed).

## Quality and checks
- Every floor module: panes or windows in every bay with frame and sill (glass), or concrete frame and balcony (slab), on the front and sides. The back face may be simpler. If over budget, reduce the back face first, never the street face.
- Look at each preview. Look at the stack previews from R39-05: no gap, the spandrel and slab bands line up between floors.
- Do not use `kit.DOOR_H` for the front door; use the lobby shell.

## Done when
- The 16 glass and 16 slab glbs (base, floor, floor_b, crown + LOD1, per size) exist within budget; the build log has no `Traceback` and no `FAILED`.
- Glass reads as a broken office (mullions, dead panes, cracks) and slab as a 70s block (balconies with junk, stains, dishes) in close previews.
- Stack previews for glass 24 and 32 and slab 24 and 32 show no gap.

## Worker notes
