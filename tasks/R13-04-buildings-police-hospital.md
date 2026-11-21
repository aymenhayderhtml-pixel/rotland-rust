# R13-04: city buildings police + hospital

Read first: `tasks/R13-common.md`, `docs/CITY_SPEC.md` section 4, `tools/blender/buildings/house_small.py`, `tools/blender/buildings/gas_station.py`, `tools/blender/buildings/kit.py`.
Files: tools/blender/buildings/police.py, tools/blender/buildings/police_lod1.py, tools/blender/buildings/hospital.py, tools/blender/buildings/hospital_lod1.py, assets/models/buildings/police*, assets/models/buildings/hospital*, assets/models/_previews/police*, assets/models/_previews/hospital*, tasks/R13-04-*.md

Low-poly, flat colors, ruined and dusty but readable from far away. The lot size is the outer footprint (`lot_size` empty); keep the building 1 m inside it on every side. Ground at z=0, use `kit.FLOOR_TOP` like the other buildings.

## police
Lot 24 x 18 m. Budget: `police` 8000 triangles, `police_lod1` 1000 triangles.
2-floor police station (upper floor closed): blue-gray walls, a blue stripe, steps up to double doors, a flagpole without a flag, barred windows, a fenced parking corner with a barrier. Inside: front desk, a hallway, one cell with bars, a locker room. 5 loot, 4 spawns.

## hospital
Lot 30 x 22 m. Budget: `hospital` 10000 triangles, `hospital_lod1` 1200 triangles.
3-floor hospital (only the ground floor enterable): white walls with a red stripe, a red cross sign (boxes), a covered ambulance bay (roof on posts) at one end, wide entrance doors. Inside: reception, a corridor, 2 rooms with beds (simple boxes). 5 loot, 5 spawns.

## Worker notes
- Created `police.py` and `police_lod1.py`:
  - `police`: 2-floor police station on 24 x 18 m lot (footprint 15 x 15 m building, 7 x 15 m fenced parking corner). Blue-gray (`gown_blue`) walls with blue (`shirt_blue`) accent stripes, entrance steps with handrails, double doors, flagpole without flag, barred ground-floor windows, fenced parking lot with perimeter fence, striped barrier arm, and dumpster. Inside: front desk lobby with computer and file cabinets, central hallway, jail cell with iron bars, bunk bed and toilet, locker room with lockers and bench, armory with shelves and gun racks, barricaded stairs to closed second floor.
    - Triangle count: 5164 tris (budget 8000) -> PASS
    - Markers: `lot_size` (scale 24 x 18 x 1), `door_1`, `door_2`, `loot_1`..`loot_5`, `spawn_1`..`spawn_4`
  - `police_lod1`: Low-detail model matching outer footprint, volumes, roofline, parking corner, barrier, flagpole, and main colors, with interior and fine details removed.
    - Triangle count: 336 tris (budget 1000) -> PASS
    - Markers: `lot_size` (scale 24 x 18 x 1)
- Created `hospital.py` and `hospital_lod1.py`:
  - `hospital`: 3-floor hospital on 30 x 22 m lot (footprint 21 x 18 m building, 7 x 17 m ambulance bay). White walls with red (`red_paint`) stripes between floors and on roof fascia, 3D red cross sign made of boxes on the front facade, covered ambulance bay on posts with emergency parking markings, wide entrance doors with awning. Inside: reception waiting room with desk, chairs, water cooler, central corridor with blue line and crash cart/gurney, Room 1 (triage) with 2 simple box hospital beds, monitor carts, IV stand, Room 2 (ward) with 2 simple box hospital beds, privacy screens, oxygen tank, pharmacy storage with stocked medicine racks, boarded stairwell to closed upper floors.
    - Triangle count: 4872 tris (budget 10000) -> PASS
    - Markers: `lot_size` (scale 30 x 22 x 1), `door_1`, `door_2`, `loot_1`..`loot_5`, `spawn_1`..`spawn_5`
  - `hospital_lod1`: Low-detail model matching 3-floor silhouette, ambulance bay on posts, entrance awning, red cross sign, red stripes, elevator penthouse, and rooftop AC unit.
    - Triangle count: 468 tris (budget 1200) -> PASS
    - Markers: `lot_size` (scale 30 x 22 x 1)
- Palette: All colors used exist in `tools/blender/palette.py` (`gown_blue`, `shirt_blue`, `rock_dark`, `rock_mid`, `rock_light`, `white`, `red_paint`, `metal`, `gun_black`, `brass`, `wood_crate`, `water_blue`, `bark_dark`, `pants_dark`).
- Previews visually verified in `assets/models/_previews/`:
  - `police.png`, `police_top.png`, `police_lod1.png`
  - `hospital.png`, `hospital_top.png`, `hospital_lod1.png`
- Wiring notes for Claude:
  - Add `police` and `hospital` to `tools/blender/buildings/build_buildings.py`
  - Add `police_lod1` and `hospital_lod1` to `tools/blender/buildings/build_lods.py` (or equivalent build lists)
- Known issues: None.
