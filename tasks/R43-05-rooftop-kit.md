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
Models + colliders + previews. Fill `## Worker notes`:
- the fire-escape face/side and `fe_u` per size, so R43-02 can place the ladders
- the ramp angles
- the empty names

## Worker notes
