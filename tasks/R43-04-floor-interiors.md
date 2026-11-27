# R43-04: Stop-off floor interiors (office, apartments, last-stand camp)

Owns:
- `tools/blender/towers/interiors.py` (NEW, `build_all()`; R43-03's build script calls it), `tools/blender/towers/interior_props.py` (NEW, if the furniture needs its own file to stay under ~300 lines)
- `assets/models/buildings/tw_interior_{office,flat,camp}_{24,32}.glb` (+ `_lod1` only if the POI path needs one; check `rotland/src/props.rs`) + previews
- `data/climb_floors.json` (NEW: crates per interior model)

Not owned: stair modules (R43-03), placement code (R43-01).
Read: tasks/R43-common.md (the shaft frame, hatch spot, front = -Z: follow EXACTLY), `tools/blender/common.py`, `tools/blender/towers/lobby_props*.py` + `shell.py` (style), `tools/blender/places/*.py` or the R42 building scripts (walk-in rooms, colliders with door gaps, `spawn_*` empties), `data/places/hospital.json` (crate entry format).

## Every interior (one storey, 3.8 m; origin = tower centre on the floor surface y = 0)
- **Fills the square footprint** (24 or 32 m) minus:
  - the **shaft rect**: centre `(0, 0.25*size)`, 4.4 x 6.4 m. Leave it completely empty, with no floor there.
  - the **hatch spot**: a 2 x 2 m clear area at `(3.4, 0.25*size)` on every model, because any stop floor may be the top one.
- **Corridor:** from the shaft door (shaft -Z face, x -0.7..+0.7) into the floor. Doorways at least 1.2 m wide everywhere; the player and zombies must walk through.
- **Window wall:** the inside skin of the outer walls (0.15 m in from the footprint edge) with window openings every 3-4 m, sills, broken panes, curtains. You see the city through them. (The exterior module is back-face culled from inside, so its windows don't need to match.)
- **Surfaces:** a floor (carpet tiles / lino / wood by kind, stained) and a ceiling (tiles, some fallen, hanging wires, a light fixture or two).
- **Overgrowth:** some vines in from the windows, dust, leaves near broken windows, a water stain.
- **Colliders** (`add_collider`):
  - the inner walls (door gaps open)
  - the big furniture (desks, counters, beds, sofas, barricades)
  - NOT the outer walls, floor or ceiling: the code adds those (R43-01)
  - NOTHING inside the shaft rect or the hatch spot
- **Zombie spawns:** `spawn_1..spawn_N` (N 4-7) spread over the floor, 0.1 m above it, never inside furniture colliders.
- **Budgets:** up to ~45k tris (32 m) / 30k (24 m), 6 materials from the palette. Detail first: clutter, but no 2-tri decals flush with surfaces (2 cm out: the z-fight rule).

## The three kinds
- **`office`:**
  - An open-plan office: rows of desks with monitors and chairs (some tipped), cubicle partitions, a meeting room with a glass wall, a kitchenette, a copier, filing cabinets.
  - Papers everywhere; a desk barricade near the shaft door.
- **`flat`:** apartments.
  - A central corridor with 4-6 apartment doors (some kicked in, some boarded).
  - Inside: beds, sofas, kitchens, a bathroom, family clutter, a crib or toys in one.
- **`camp`:** a survivors' last stand.
  - Furniture barricades at the shaft door (with a gap you can walk through) and at the corridor ends.
  - Mattresses and sleeping bags, a stove, water jugs, a supply pile, a radio table with a map on the wall, "HELP" painted on the windows (reversed letters are fine), bodies under sheets.
  - It reads as people who held out here and lost.

## Crates (`data/climb_floors.json`)
```json
{
  "_why": "R43-04: loot boxes per stop-off floor interior. Interior-local game metres (x, y, z = Blender x, z, -y), front = -Z.",
  "tw_interior_office_24": [ { "x": 0, "y": 0, "z": 0, "yaw": 0, "model": "metro_locker", "table": "metro_tower_office" } ]
}
```
- 3-5 crates per model, using the existing containers (`metro_locker`, `med_cabinet`, `army_crate`, `camp_stash`). Not in the shaft or hatch spot, not inside colliders.
- Tables: `metro_tower_office`, `metro_tower_flat`, `metro_tower_camp` (R43-06 makes them).

## Previews
For each model: a top-down cut-away (ceiling hidden) and one eye-level shot from the shaft door. Each one must read at a glance as an office / apartments / a last stand. Fix and re-render until it does.

## Done when
6 models + data + previews. Fill `## Worker notes`: tris per model, `spawn_` count, `lod1` decision.

## Worker notes
- **Files added:**
  - `tools/blender/towers/interiors.py` (exports `build_all()`, returns `[(name, ok), ...]`)
  - `tools/blender/towers/interior_props.py` (furniture, barricade, and clutter builders)
  - `data/climb_floors.json` (4 crates per 24m model, 5 crates per 32m model)
  - `assets/models/buildings/tw_interior_{office,flat,camp}_{24,32}.glb`
- **Tri counts (all within 30k / 45k budgets):**
  - `tw_interior_office_24`: 27,300 tris (budget 30,000)
  - `tw_interior_office_32`: 42,936 tris (budget 45,000)
  - `tw_interior_flat_24`: 12,588 tris (budget 30,000)
  - `tw_interior_flat_32`: 14,724 tris (budget 45,000)
  - `tw_interior_camp_24`: 10,504 tris (budget 30,000)
  - `tw_interior_camp_32`: 12,640 tris (budget 45,000)
- **`spawn_` counts:** 5 (`spawn_1..spawn_5`) on all 24 m models; 6 (`spawn_1..spawn_6`) on all 32 m models.
- **`_lod1` decision:** Not needed. Checked `rotland/src/props.rs` (`load_model` and `load_tower_model`): `_lod1` is optional, and interiors are only visible from inside or through nearby windows.
- **Shaft & hatch:** Shaft rect `(0, 0.25*size)` 4.4 x 6.4 m is completely empty (no floor, ceiling, colliders, furniture, crates, or spawns). Hatch spot `(3.4, 0.25*size)` 2 x 2 m is kept clear of furniture, colliders, crates, and spawns.
- **Previews inspected:** `assets/models/_previews/tw_interior_{office,flat,camp}_{24,32}{_top,_eye}.png` (all 12 verified via `read_file`).
