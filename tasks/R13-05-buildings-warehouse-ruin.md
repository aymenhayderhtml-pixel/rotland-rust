# R13-05: city buildings warehouse + ruin

Read first: `tasks/R13-common.md`, `docs/CITY_SPEC.md` section 4, `tools/blender/buildings/house_small.py`, `tools/blender/buildings/gas_station.py`, `tools/blender/buildings/kit.py`.
Files: tools/blender/buildings/warehouse.py, tools/blender/buildings/warehouse_lod1.py, tools/blender/buildings/ruin.py, tools/blender/buildings/ruin_lod1.py, assets/models/buildings/warehouse*, assets/models/buildings/ruin*, assets/models/_previews/warehouse*, assets/models/_previews/ruin*, tasks/R13-05-*.md

Low-poly, flat colors, ruined and dusty but readable from far away. The lot size is the outer footprint (`lot_size` empty); keep the building 1 m inside it on every side. Ground at z=0, use `kit.FLOOR_TOP` like the other buildings.

## warehouse
Lot 26 x 16 m. Budget: `warehouse` 4000 triangles, `warehouse_lod1` 500 triangles.
one big metal warehouse: corrugated-look walls (alternate two grays in vertical bands), a shallow gable roof, one big roller door (half open, so you can walk under) and one small side door, a loading dock platform. Inside: one big room with pallet stacks and crates. 4 loot, 3 spawns.

## ruin
Lot 16 x 12 m. Budget: `ruin` 4000 triangles, `ruin_lod1` 500 triangles.
a 2-floor brick building that has collapsed: walls broken at different heights (stepped, jagged tops), no roof, part of the upper floor slab hanging at an angle, rubble piles inside and outside, open to walk in through gaps. 3 loot, 3 spawns.

## Worker notes
- Implemented `tools/blender/buildings/warehouse.py` and `tools/blender/buildings/warehouse_lod1.py`:
  - 23 x 11 m footprint on a 26 x 16 m lot (all geometry >= 1 m inside lot bounds).
  - Corrugated metal walls with alternating vertical bands (`metal` and `rock_dark`).
  - Shallow gable roof with ridge cap, fascia eave trim, and roof rotary turbine ventilators.
  - Front half-open roller door (4.0 m wide, 2.2 m walk-under clearance with yellow/black hazard edge).
  - Side pedestrian door (1.2 m wide with metal frame and canopy on right wall).
  - Front-right raised loading dock platform (Z = 0.7 m, yellow safety stripes, rubber bumpers, steps, canopy awning, and closed loading bay doors).
  - Interior: Concrete floor at `kit.FLOOR_TOP` (0.1 m), steel portal frame trusses, multiple pallet stacks, wooden crates, oil barrels, a hand pallet jack, an industrial 3-tier storage rack, and a shipping desk.
  - Markers: `lot_size` (scale 26 x 16 x 1 m), `door_1`, `door_2`, 4 loot markers (`loot_1`..`loot_4`), 3 spawn markers (`spawn_1`..`spawn_3`).
  - Triangles: `warehouse` 3436 tris (budget 4000), `warehouse_lod1` 424 tris (budget 500).

- Implemented `tools/blender/buildings/ruin.py` and `tools/blender/buildings/ruin_lod1.py`:
  - 13 x 9 m footprint on a 16 x 12 m lot (all geometry >= 1 m inside lot bounds).
  - Collapsed 2-floor red brick building (`red_paint` with mortar courses).
  - Walls broken at various heights with stepped, jagged tops (back-left corner reaches 5.2 m into the 2nd floor, stepping down to broken sections).
  - No roof.
  - 2nd-floor concrete slab section anchored in back-left, with a suspended tilted slab section hanging down at an angle into the interior with exposed twisted rebar rods.
  - Natural low-poly rubble piles inside and outside the walls with scattered bricks and a leaning timber beam.
  - Open at floor level: 5.4 m wide main front breach and 2.7 m wide side breach for smooth player entry.
  - Markers: `lot_size` (scale 16 x 12 x 1 m), `door_1`, `door_2`, 3 loot markers (`loot_1`..`loot_3`), 3 spawn markers (`spawn_1`..`spawn_3`).
  - Triangles: `ruin` 2224 tris (budget 4000), `ruin_lod1` 264 tris (budget 500).

- Palette & Assets:
  - All colors used are from palette Sheet A (`metal`, `rock_dark`, `rock_mid`, `rock_light`, `wood_crate`, `bark_dark`, `bark_light`, `dirt`, `red_paint`, `white`, `gun_black`, `brass`).
  - Visuals verified via workbench preview renders (`warehouse.png`, `warehouse_top.png`, `warehouse_lod1.png`, `ruin.png`, `ruin_top.png`, `ruin_lod1.png`).
  - Scope check passed: `python3 -I tools/gate_checks.py --base main --task tasks/R13-05-buildings-warehouse-ruin.md` returns `scope: PASS`.
- Known issues: None.
