# R43-05: Rooftop kit, radio console, walkable brick fire escapes

Owns:
- `tools/blender/towers/rooftop_kit.py` (NEW, `build_all()`; R43-03's build script calls it)
- `assets/models/props/{roof_plank_4,roof_plank_8,roof_plank_12,roof_step_crate,radio_console}.glb` (+ previews)
- `tools/blender/towers/brick.py`, `brick_parts.py` (ONLY: colliders + ladder empties for the fire escapes; the look must not change by one vertex) and the rebuilt `tw_brick_*` glbs

Not owned: stair modules (R43-03), placement (R43-01/02).
Read: tasks/R43-common.md, `tools/blender/common.py` (`add_collider`, `add_empty`), `tools/blender/towers/brick.py` + `brick_parts.py` (`add_fire_escape_floor`: faces, `fe_u`, `zig_dir`, `has_landing`, `roof=True`, the base drop ladder), `tools/blender/towers/landmarks.py` (the mast platform + `radio_hut`), `tools/blender/towers/roof_parts.py` (style).

## Rooftop props
- **`roof_plank_4/8/12`:**
  - Rough scaffold boards (2-3 boards side by side, 1.2 m wide, slight sag and offsets), nailed cross battens.
  - A rope or a scaffold-pipe rail on ONE side, with a frayed end.
  - Long along Blender Y (game Z), origin at the middle, top surface at y = 0.
  - Collider: one thin walk slab per plank model (+ the rail post sides if cheap).
- **`roof_step_crate`:** a stack of 1-2 pallets/crates as a step, 0.3-0.6 m tall, with a walkable collider.
- **`radio_console`:**
  - A field radio set on a crate table: a handset on a cord, a battery pack, a cable up toward the hut, a small notepad with frequencies.
  - It reads at 3 m. Collider on the table.

## Brick fire escapes (walkable)
- For every brick module that draws fire escape parts (`_base`, `_floor`, `_floor_b`, `_floor_c`, `_crown`, all sizes): add **colliders** for:
  - each landing slab
  - each zig-zag flight as ONE ramp (the slope through the step noses; it must be under 46 degrees, so report the angle)
  - the railings as thin walls (so you can't walk off)
- **Add ladder empties:**
  - **Base:** `ladder_bottom_fe` (street) and `ladder_top_fe` (first landing), along the drawn drop ladder.
  - **Crown:** `ladder_bottom_fer` (the top landing) and `ladder_top_fer` (onto the roof deck, 1 m inside the parapet). Draw a short gooseneck ladder over the parapet if none exists.
- Check that the flights of stacked modules meet with no gap (preview a 3-floor stack + base + crown from the side).
- The meshes stay EXACTLY the same: compare tri counts before/after and report both.

## Previews
- Each rooftop prop.
- The 3-storey brick stack with the fire escape colliders shown as a wireframe overlay.

## Done when
Models + colliders + previews. Fill `## Worker notes

### Files changed / added
- `tools/blender/towers/rooftop_kit.py` (NEW): exports `build_all()`, builds `roof_plank_4`, `roof_plank_8`, `roof_plank_12`, `roof_step_crate`, `radio_console` into `assets/models/props/*.glb`.
- `tools/blender/towers/brick_parts.py`: added `add_fire_escape_colliders()` called by `add_fire_escape_floor()` (landing slab, outer/end guard rail walls, and one zig-zag ramp collider). Visual meshes unchanged.
- `tools/blender/towers/brick.py`: added `ladder_bottom_fe`/`ladder_top_fe` empties on `build_base`, and top landing colliders + gooseneck ladder over the parapet + `ladder_bottom_fer`/`ladder_top_fer` empties on `build_crown`.
- Rebuilt `assets/models/buildings/tw_brick_{16,24}_*.glb` (via `brick.build_all()` and `brick.build_r40_all()`).

### Builder functions for `build_r43_models.py`
- `rooftop_kit.build_all()`
- `brick.build_all()` and `brick.build_r40_all()` (note: `build_r40_all()` builds `tw_brick_{16,24}_floor_c` and `crown`; `build_all()` builds `floor`, `floor_b`, `base`, `crown` + `_lod1`).

### Fire-escape face/side, `fe_u`, ramp angles, and empties
- **Primary fire-escape face (full climb from base drop ladder to crown gooseneck):**
  - Blender `+Y` face (game local `-Z` front), `axis=1, sign=1`, `zig_dir=1`.
  - `size == 16` (`h = 8.0`): `fe_u = -2.0` (X = -2.0 in Blender, X = -2.0 in game local).
  - `size == 24` (`h = 12.0`): `fe_u = -4.0` (X = -4.0 in Blender, X = -4.0 in game local).
  - Note: `tw_brick_{16,24}_floor_b` has `has_landing=False` on the `+Y` face (only `floor` and `floor_c` have the `+Y` landing/stair).
- **Secondary fire-escape face:** Blender `+X` face (`axis=0, sign=1`, `u_center = 0.0`, `zig_dir = -1`) on `floor` (not on `base`, `crown`, or `floor_c`).
- **Ramp angle:** `atan2(3.80, 4.10) = 42.82 deg` (under the 46 deg player slope limit). Run `4.10 m` (`u_center +- 2.05 * zig_dir`), rise `3.80 m` from `z0 + 0.07` to `z1 + 0.12` so stacked modules meet with zero gap.
- **Empty names:**
  - `tw_brick_{16,24}_base`: `ladder_bottom_fe` at `(fe_u - 2.075, h + 1.25, 0.0)` (`rot_z = pi`), `ladder_top_fe` at `(fe_u - 2.075, h + 0.45, 3.92)`.
  - `tw_brick_{16,24}_crown`: `ladder_bottom_fer` at `(fe_u, h + 0.95, 0.12)` (`rot_z = pi`), `ladder_top_fer` at `(fe_u, h - 1.32, 2.40)`.

### Tri counts (before -> after)
- `tw_brick_16_base.glb`: 3924 -> 3924 (10 -> 15 cols)
- `tw_brick_16_floor.glb`: 1908 -> 1908 (0 -> 10 cols)
- `tw_brick_16_floor_b.glb`: 1824 -> 1824 (0 -> 5 cols)
- `tw_brick_16_floor_c.glb`: 2208 -> 2208 (0 -> 5 cols)
- `tw_brick_16_crown.glb`: 2402 -> 2606 (+204 tris for gooseneck ladder; 0 -> 4 cols)
- `tw_brick_24_base.glb`: 5484 -> 5484 (10 -> 15 cols)
- `tw_brick_24_floor.glb`: 2688 -> 2688 (0 -> 10 cols)
- `tw_brick_24_floor_b.glb`: 2652 -> 2652 (0 -> 5 cols)
- `tw_brick_24_floor_c.glb`: 3024 -> 3024 (0 -> 5 cols)
- `tw_brick_24_crown.glb`: 3936 -> 4140 (+204 tris for gooseneck ladder; 0 -> 4 cols)
- All `tw_brick_*_lod1.glb`: unchanged.
- Rooftop props: `roof_plank_4` (408/500), `roof_plank_8` (636/700), `roof_plank_12` (900/900), `roof_step_crate` (228/400), `radio_console` (592/600).

### Previews (inspected)
- `assets/models/_previews/roof_plank_4.png`
- `assets/models/_previews/roof_plank_8.png`
- `assets/models/_previews/roof_plank_12.png`
- `assets/models/_previews/roof_step_crate.png`
- `assets/models/_previews/radio_console.png`
- `assets/models/_previews/brick_16_fire_escape_stack.png``:
- the fire-escape face/side and `fe_u` per size, so R43-02 can place the ladders
- the ramp angles
- the empty names

## Worker notes

### Files changed / added
- `tools/blender/towers/rooftop_kit.py` (NEW): exports `build_all()`, builds `roof_plank_4`, `roof_plank_8`, `roof_plank_12`, `roof_step_crate`, `radio_console` into `assets/models/props/*.glb`.
- `tools/blender/towers/brick_parts.py`: added `add_fire_escape_colliders()` called by `add_fire_escape_floor()` (landing slab, outer/end guard rail walls, and one zig-zag ramp collider). Visual meshes unchanged.
- `tools/blender/towers/brick.py`: added `ladder_bottom_fe`/`ladder_top_fe` empties on `build_base`, and top landing colliders + gooseneck ladder over the parapet + `ladder_bottom_fer`/`ladder_top_fer` empties on `build_crown`.
- Rebuilt `assets/models/buildings/tw_brick_{16,24}_*.glb` (via `brick.build_all()` and `brick.build_r40_all()`).

### Builder functions for `build_r43_models.py`
- `rooftop_kit.build_all()`
- `brick.build_all()` and `brick.build_r40_all()` (note: `build_r40_all()` builds `tw_brick_{16,24}_floor_c` and `crown`; `build_all()` builds `floor`, `floor_b`, `base`, `crown` + `_lod1`).

### Fire-escape face/side, `fe_u`, ramp angles, and empties
- **Primary fire-escape face (full climb from base drop ladder to crown gooseneck):**
  - Blender `+Y` face (game local `-Z` front), `axis=1, sign=1`, `zig_dir=1`.
  - `size == 16` (`h = 8.0`): `fe_u = -2.0` (X = -2.0 in Blender, X = -2.0 in game local).
  - `size == 24` (`h = 12.0`): `fe_u = -4.0` (X = -4.0 in Blender, X = -4.0 in game local).
  - Note: `tw_brick_{16,24}_floor_b` has `has_landing=False` on the `+Y` face (only `floor` and `floor_c` have the `+Y` landing/stair).
- **Secondary fire-escape face:** Blender `+X` face (`axis=0, sign=1`, `u_center = 0.0`, `zig_dir = -1`) on `floor` (not on `base`, `crown`, or `floor_c`).
- **Ramp angle:** `atan2(3.80, 4.10) = 42.82 deg` (under the 46 deg player slope limit). Run `4.10 m` (`u_center +- 2.05 * zig_dir`), rise `3.80 m` from `z0 + 0.07` to `z1 + 0.12` so stacked modules meet with zero gap.
- **Empty names:**
  - `tw_brick_{16,24}_base`: `ladder_bottom_fe` at `(fe_u - 2.075, h + 1.25, 0.0)` (`rot_z = pi`), `ladder_top_fe` at `(fe_u - 2.075, h + 0.45, 3.92)`.
  - `tw_brick_{16,24}_crown`: `ladder_bottom_fer` at `(fe_u, h + 0.95, 0.12)` (`rot_z = pi`), `ladder_top_fer` at `(fe_u, h - 1.32, 2.40)`.

### Tri counts (before -> after)
- `tw_brick_16_base.glb`: 3924 -> 3924 (10 -> 15 cols)
- `tw_brick_16_floor.glb`: 1908 -> 1908 (0 -> 10 cols)
- `tw_brick_16_floor_b.glb`: 1824 -> 1824 (0 -> 5 cols)
- `tw_brick_16_floor_c.glb`: 2208 -> 2208 (0 -> 5 cols)
- `tw_brick_16_crown.glb`: 2402 -> 2606 (+204 tris for gooseneck ladder; 0 -> 4 cols)
- `tw_brick_24_base.glb`: 5484 -> 5484 (10 -> 15 cols)
- `tw_brick_24_floor.glb`: 2688 -> 2688 (0 -> 10 cols)
- `tw_brick_24_floor_b.glb`: 2652 -> 2652 (0 -> 5 cols)
- `tw_brick_24_floor_c.glb`: 3024 -> 3024 (0 -> 5 cols)
- `tw_brick_24_crown.glb`: 3936 -> 4140 (+204 tris for gooseneck ladder; 0 -> 4 cols)
- All `tw_brick_*_lod1.glb`: unchanged.
- Rooftop props: `roof_plank_4` (408/500), `roof_plank_8` (636/700), `roof_plank_12` (900/900), `roof_step_crate` (228/400), `radio_console` (592/600).

### Previews (inspected)
- `assets/models/_previews/roof_plank_4.png`
- `assets/models/_previews/roof_plank_8.png`
- `assets/models/_previews/roof_plank_12.png`
- `assets/models/_previews/roof_step_crate.png`
- `assets/models/_previews/radio_console.png`
- `assets/models/_previews/brick_16_fire_escape_stack.png`
