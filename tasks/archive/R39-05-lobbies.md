# R39-05: lobby shell, lobby props, build runner (Blender)

Owns: tools/blender/towers/shell.py, tools/blender/towers/lobby_props.py (+ lobby_props_b.py if it passes ~300 lines), tools/blender/towers/stack_preview.py, tools/blender/build_r39_models.py (the one build runner for all of R39; R39-02..04 do not touch it), assets/models/buildings/lobby_*.glb, assets/models/_previews/lobby_*.png and stack_*.png. Shared lines for the lead: the palette names in "Palette" below (appended to `COLORS_C` in tools/blender/palette.py, then `python3 tools/blender/palette.py`); no other shared file.
Read: tools/blender/common.py (`Model`: `add`, `add_collider`, `add_part`, `add_empty`, `top_preview`, `grip`, `finalize_and_export`; `run_model`; `make_box`, `make_frustum`, `make_icosphere`, `combine`, `transform`, `paint_faces`), tools/blender/buildings/kit.py (`Kit.box`, `solid`, `collider`, `wall`, `frame`, `boards`, `barrel`, `FLOOR_TOP`, `FRONT_VIEW`; note `DOOR_H` is 2.2 m and the lobby door is taller, so do not use it), tools/blender/buildings/apartment.py (the lobby mailbox and stair pattern), tools/blender/buildings/office.py (`main()` pattern, rubble and storefront), tools/blender/metro/jam_util.py (`box`, `pbox`, `bevel_bm`), tools/blender/build_r38_models.py (the `GROUPS` / `FAILED` runner pattern), tools/blender/palette.py (`COLORS`, `COLORS_C`), docs/ART_PIPELINE.md, data/budgets.json (`buildings` 80000).

Goal: the lobby every tower has (walk in from the street), the six lobby props, the shared lobby shell that R39-02..04 copy, and the one build runner for all R39 models.

## Shared kit rules (identical in R39-02, 03, 04, 05)
- Footprint `size` x `size` (16, 24 or 32 m), centred on X=0, Y=0. Z=0 is the ground. Front = +Y = the street.
- Origin: run every tower model with `grip=(0.0, 0.0, 0.0)` so the origin is the footprint centre at ground level. Overhangs (fire escapes, balconies) may stick out on +Y only, at most 1.2 m, and must not move the origin.
- Exact heights: floor module z 0..`floor_height` (3.8 m); base z 0..`base_height` (7.6 m, 2 floors); crown z 0..`crown_height` (per style, see the table). Each module has a slab band at its bottom and top edges, so stacks have no gap.
- Floor modules and crowns are visual only (no colliders). Only the base has colliders (from the lobby shell, plus its props). R39-06 adds the solid roof box.
- LOD1 (`<name>_lod1`): same silhouette and facade colours, plain boxes, no door, no colliders, no small detail, about 1/10 of LOD0 triangles.
- Facade windows are visual decals: dark boxes proud of the wall by 0.02 m. Only the door gap is a real hole.
- Budgets (LOD0 / LOD1): floor 16 m 3000 / 300; floor 24 m 4500 / 450; floor 32 m 6000 / 600 (`floor_b` = `floor`); base 15000 / 1500; crown 8000 / 800; landmark crown 25000 / 2500; lobby props 6000 each, no LOD1.
- Category: `buildings` for every model in this round (lobby props included), so the glbs land in assets/models/buildings/.
- Palette: only names in tools/blender/palette.py (sheets A, B, C), plus the "Palette" names below once the lead has added them.
- Each style script has `build_all()` returning a list of `(name, ok)`, and a `__main__` block. Run one style alone with `flock /tmp/rotland-blender.lock blender -b --python tools/blender/towers/<style>.py`.

## Lobby shell (the spec R39-02..04 copy) — `shell.py`
- Constants (with `# new: DECISIONS 2026-11-26`): `WALL_T = 0.3`, `FLOOR_T = 0.1` (the same raised floor as kit.FLOOR_TOP), `HALL_TOP = 7.3`, `BASE_H = 7.6`, `DOOR_W = 2.8`, `DOOR_H = 3.0`, `STAIR_W = 1.2`, `STAIR_H = 2.2`.
- `def door_opening(size)` returns the front door box `(lo, hi)` (x -1.4..1.4, z 0.1..3.1, on the +Y wall). `def front_face_y(size)` returns `size / 2`.
- `def lobby_shell(k, size, facade_color)` on a `kit.Kit`:
  - Floor slab over the whole footprint, z 0..0.1 (visual + collider).
  - Four walls, thickness 0.3 m, full base height, with `k.wall(..., openings=[door])` on the front so the colliders have the door gap. The gap is 2.8 m wide and 3.0 m tall, so the player walks straight in.
  - Hall is open: no interior walls. Ceiling slab z 7.3..7.6 (visual only). Hall walls use the `rock_light` plaster colour inside.
  - Stair door: closed, on the -Y (back) wall, centred on X=0, 1.2 m x 2.2 m. Exported as `add_part("stair_door", bm)` (a separate node, so R43 can hide or move it) plus `add_collider("stair_door", bm)`.
- The base's own facade code (R39-02..04) adds trims, windows and signs outside the door gap, never inside it.

## Lobby props (`lobby_props.py`, one build function each, each its own model)
- `lobby_desk`: reception counter 2.6 x 0.9 x 1.1 m, top `rock_light`, front `bark_light`, a dead monitor (`gun_black`), a phone (`white`), papers. Collider: the counter box.
- `lobby_elevators`: a bank of two steel-door cars, 4.0 x 0.6 x 3.0 m. Left door jammed 0.4 m open, dark shaft behind (`soot`), dead floor panel, call buttons, an "OUT OF ORDER" stripe (no words). Colliders: door leaves and frame; the shaft has none.
- `lobby_mailboxes`: wall unit 3.0 x 0.4 x 2.0 m, a 4 x 3 grid of metal boxes (`metal`), three doors hanging open, key slots. Collider: the unit.
- `lobby_bench`: 2.0 x 0.5 x 0.45 m, `bark_light` slats (one broken), `metal` legs. Collider: the seat box.
- `lobby_rubble`: 2.5 x 2.0 m pile up to 0.35 m: broken concrete (`rock_mid`), plaster (`white`), red brick bits (`brick_red`), a fallen ceiling tile. Collider: one 0.35 m box.
- `lobby_plant_dead`: planter 0.9 x 0.9 x 0.7 m (`rock_mid`), dead shrub stems (`bark_dark`, `soil_dark`), dry leaves (`dry_brush`). Collider: the planter.
- Quality: each prop has at least 10 separate parts (no single plain box), bevel the panels with `jam_util.pbox`.

## Stack preview and build runner
- `towers/stack_preview.py`: after the glbs exist, imports one base, two floors (`floor`, `floor_b`) and one crown of each style and size, stacks them at the exact heights (heights read from each glb's bounding box, not typed in), joins them, and renders `assets/models/_previews/stack_<style>_<size>.png` with `common.render_preview`. Not exported.
- `build_r39_models.py`: `GROUPS` = brick, stone, glass, slab, deco, landmarks (from `towers/`), then lobby_props. Each module's `build_all()` gives `(name, ok)`. Prints `R39 <group>: <name> OK|FAILED`, then runs `stack_preview`. Prints `R39 BUILD FAILED: ...` and exits 1 on any failure, like `build_r38_models.py`. Run: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r39_models.py`.

## Palette (lead appends to `COLORS_C`, names only the kit tasks use)
`brick_red` (136, 72, 54); `brick_dark` (96, 52, 42); `brownstone` (116, 84, 62); `mortar` (176, 170, 158); `limestone` (206, 198, 178); `limestone_dark` (168, 160, 144); `glass_dark` (44, 60, 68); `concrete_grey` (150, 150, 146); `concrete_stain` (120, 114, 100); `rust_dark` (96, 56, 34); `soot` (58, 46, 40); `grime_green` (60, 84, 48); `tarp_blue` (58, 86, 112). Sheet C cells 16-28 (the free grey cells); existing cells keep their UVs.

## Tests and checks
- No Rust, no cargo. The lead runs the gate. Kit workers check the build log (no `Traceback`, no `FAILED`, every model within budget) and LOOK at each preview.
- Door check: the front preview shows the open door gap; the top preview (`top_preview=True` on bases) shows the open hall.

## Done when
- `shell.py`, the six lobby glbs (+ previews), `stack_preview.py`, `build_r39_models.py` exist; the build log has no `Traceback` and no `FAILED`.
- Every lobby prop looks dense and dead in its preview; the desk, elevators and mailboxes read at a glance.
- All ten stack previews (one per style and size: brick 16 and 24, stone, glass, slab and deco 24 and 32) show no gap and no floating part (checked by eye).

## Worker notes
- **Files created:**
  - `tools/blender/towers/shell.py`: `WALL_T`, `FLOOR_T`, `HALL_TOP`, `BASE_H`, `DOOR_W`, `DOOR_H`, `STAIR_W`, `STAIR_H` (with `# new: DECISIONS 2026-11-26`), `front_face_y(size)`, `door_opening(size)`, `lobby_shell(k, size, facade_color)`. Creates floor slab (`0..0.1`), 4 exterior walls (`0.3 m` thick, `0.1..7.3`) with front door gap (`x -1.4..1.4, z 0.1..3.1`) on `+Y`, `rock_light` interior plaster lining, `roof=True` ceiling slab (`7.3..7.6`), and `stair_door` separate part + collider on `-Y`. Verified with a Blender smoke test on `size=16`.
  - `tools/blender/towers/lobby_props.py` & `tools/blender/towers/lobby_props_b.py`: all 6 lobby props (`buildings` category, 6000 tri budget, >= 10 beveled parts each using `jam_util.pbox`).
  - `tools/blender/towers/stack_preview.py`: imports `base`, `floor`, `floor_b`, `crown` GLBs per style+size (10 combinations), stacks by reading GLB Z bounds, joins visuals, and renders `assets/models/_previews/stack_<style>_<size>.png`.
  - `tools/blender/build_r39_models.py`: `GROUPS` for `brick`, `stone`, `glass`, `slab`, `deco`, `landmarks`, `lobby_props`, followed by `stack_preview`.
  - `assets/models/buildings/lobby_*.glb` and `assets/models/_previews/lobby_*.png` (all 6 built and visually inspected).
- **Triangle counts (all OK, budget 6000):**
  - `lobby_desk`: 1944 tris
  - `lobby_elevators`: 2160 tris
  - `lobby_mailboxes`: 3564 tris
  - `lobby_bench`: 2268 tris
  - `lobby_rubble`: 1620 tris
  - `lobby_plant_dead`: 1504 tris
- **Preview observations:**
  - All 6 previews inspected: `lobby_desk`, `lobby_elevators` (front view, jammed left door with dark shaft, call buttons, indicators, warning stripe, zero z-fighting), `lobby_mailboxes` (4x3 grid, 3 open doors, brass locks, labels), `lobby_bench` (3 metal legs, broken drooping front slat), `lobby_rubble` (stepped mound, concrete, plaster, red bricks, fallen ceiling tile + T-bar), `lobby_plant_dead` (planter, soil, connected dead twigs, dry leaves).
  - `stack_preview.py` and `build_r39_models.py` cannot be run in full until R39-02, R39-03, and R39-04 land.
