# R13-02: city buildings apartment + office

Read first: `tasks/R13-common.md`, `docs/CITY_SPEC.md` section 4, `tools/blender/buildings/house_small.py`, `tools/blender/buildings/gas_station.py`, `tools/blender/buildings/kit.py`.
Files: tools/blender/buildings/apartment.py, tools/blender/buildings/apartment_lod1.py, tools/blender/buildings/office.py, tools/blender/buildings/office_lod1.py, assets/models/buildings/apartment*, assets/models/buildings/office*, assets/models/_previews/apartment*, assets/models/_previews/office*, tasks/R13-02-*.md

Low-poly, flat colors, ruined and dusty but readable from far away. The lot size is the outer footprint (`lot_size` empty); keep the building 1 m inside it on every side. Ground at z=0, use `kit.FLOOR_TOP` like the other buildings.

## apartment
Lot 20 x 14 m. Budget: `apartment` 6000 triangles, `apartment_lod1` 800 triangles.
3-floor brick apartment block, flat roof with a low parapet and a water tank. Ground floor: entrance hall with a 1.2 m door on the front (-Y? use the same front side as house_small), a stair that ends at a boarded/collapsed landing (upper floors closed). Rows of windows on every floor, some boarded, some broken (dark). A rusty fire escape on one side. 3 loot, 3 spawns.

## office
Lot 18 x 18 m. Budget: `office` 6000 triangles, `office_lod1` 800 triangles.
4-floor concrete office, glass-band windows (dark glass color), one corner of the top floor collapsed (jagged broken edge, rubble pile at the foot). Ground floor lobby open with a reception desk; upper floors closed. 3 loot, 3 spawns.

## Worker notes
- **What was made:**
  - `apartment`: 3-floor red brick apartment block (18 x 12 m footprint within 20 x 14 m lot). Front entrance on +Y with 1.2 m wide door and portico. Open ground floor entrance hall with brass mailboxes, notice board, hallway radiator, and staircase ending at a collapsed/boarded landing barricading upper floors. 3 floors of windows with stone lintels/sills (dark panes, some boarded with planks, some broken). Rusty fire escape (`rust`) on the east side (+X) with platforms at floors 2 & 3, handrails, angled stairs, drop ladder, and roof access ladder over the parapet. Flat roof with stone coping, stilted wooden water tank with conical roof and steel hoops, roof access bulkhead, and chimney stack. Markers: `lot_size` (scale=(20, 14, 1)), `door_1`, `loot_1`..`loot_3`, `spawn_1`..`spawn_3`.
  - `apartment_lod1`: Low-poly distant LOD matching the exact 18 x 12 m footprint, 3 floors, red brick facade with stone belt courses, parapet, water tank on stilts, roof bulkhead, and fire escape silhouette. 1 convex collider.
  - `office`: 4-floor concrete office building (16 x 16 m footprint within 18 x 18 m lot). Commercial concrete facade with dark tinted glass-band windows (`glass_tint`) and precast spandrels. Top front-right corner (+X, +Y on Floor 4) collapsed with jagged fractured concrete slabs and exposed bent rebar (`rust`). Massive rubble pile at the foot with broken concrete slabs and rebar debris. Open ground floor lobby on +Y with storefront glass, entrance canopy, 1.2 m door, L-shaped reception desk with computer and swivel chair, waiting area with modern sofas, coffee table, and architectural planter. Back wall features 2 elevator doors boarded with heavy timber planks and stair door barricaded with filing cabinets (upper floors closed). Rooftop HVAC penthouse and chiller AC units. Markers: `lot_size` (scale=(18, 18, 1)), `door_1`, `loot_1`..`loot_3`, `spawn_1`..`spawn_3`.
  - `office_lod1`: Low-poly distant LOD matching the exact 16 x 16 m footprint, 4 floors, collapsed top front-right corner, rubble pile at the foot, horizontal glass bands, and roof penthouse. 3 convex colliders.
- **Triangle counts:**
  - `apartment`: 4470 tris (budget 6000) - OK
  - `apartment_lod1`: 642 tris (budget 800) - OK
  - `office`: 2936 tris (budget 6000) - OK
  - `office_lod1`: 576 tris (budget 800) - OK
- **Files created / modified:**
  - `tools/blender/buildings/apartment.py`
  - `tools/blender/buildings/apartment_lod1.py`
  - `tools/blender/buildings/office.py`
  - `tools/blender/buildings/office_lod1.py`
  - `assets/models/buildings/apartment.glb`
  - `assets/models/buildings/apartment_lod1.glb`
  - `assets/models/buildings/office.glb`
  - `assets/models/buildings/office_lod1.glb`
  - `tasks/R13-02-buildings-apartment-office.md`
- **How to test:**
  - `flock /tmp/rotland-blender.lock blender -b --python tools/blender/buildings/apartment.py`
  - `flock /tmp/rotland-blender.lock blender -b --python tools/blender/buildings/apartment_lod1.py`
  - `flock /tmp/rotland-blender.lock blender -b --python tools/blender/buildings/office.py`
  - `flock /tmp/rotland-blender.lock blender -b --python tools/blender/buildings/office_lod1.py`
  - `python3 -I tools/gate_checks.py --base main --task tasks/R13-02-buildings-apartment-office.md`
- **Wiring notes for Claude:**
  - Add `"apartment"`, `"office"` to building lists when ready.
  - Add `"apartment_lod1"`, `"office_lod1"` to LOD pipeline.
  - Markers follow standard convention: `lot_size`, `door_1`, `loot_1`..`loot_3`, `spawn_1`..`spawn_3`.
- **Known issues:**
  - None. All four models build cleanly, are within triangle budgets, match lot setbacks, and pass scope checks.
