# R42-03: Hospital (main building, props, layout data)

Owns:
- `tools/blender/places/hospital.py` + `tools/blender/places/hospital_props.py` (NEW; `build_all()` in `hospital.py`)
- `tools/blender/places/place_kit.py` (NEW, shared helpers for R42-03/04/05: walls with door gaps + colliders, floor slabs, windows, signs. You create it FIRST, in your first 15 minutes, and keep its function names stable. R42-04/05 import it; if they need a helper you don't have, they add it in their own file.)
- the new .glb files below + previews
- `data/places/hospital.json` (NEW)

Not owned: `tools/blender/build_r42_models.py` (R42-06 owns it; it calls `places.hospital.build_all`), every other script.
Read: tasks/R42-common.md (whole: the coordinate rule and the JSON shape), `tools/blender/buildings/kit.py`, `tools/blender/common.py` (`add_collider`, `add_empty`, budgets/LOD), `tools/blender/buildings/hospital.py` (the small town one: do NOT change it), `tools/blender/towers/shell.py` + `lobby_props.py` (walk-in lobby style), `tools/blender/traces/lobby_dressing.py`, the palette in `tools/blender/palette.py`.

## Rules for every R42 building (R42-03/04/05)
- Origin at the bottom centre of the footprint, front = Blender +Y, footprint at most **40 x 40 m**.
- **Ground floor is walk-in:**
  - floor slab at z = 0.15
  - real wall thickness 0.3 m
  - doors at least 1.6 m wide, 2.4 m high
  - a collider per wall piece, a collider for the floor, none in the door gaps
  - ceiling height about 4 m
- Upper floors are a closed facade with windows (dark glass, some broken, some boarded), with grime and vines like R40 (2-5 cm off the wall).
- Inside, rooms read at a glance, with plenty of small props baked in (budget below). Bigger loose props are separate models placed from the JSON.
- 8-14 `spawn_N` empties: most inside the rooms, a few at the doors.
- Budgets:
  - main building 90k LOD0 / 6k `_lod1` (lod1 = the outside only)
  - props 4k each (vehicles 10k, `_lod1` at 1/10)
  - tents/piles 6k
- Signs use painted letters like `tools/blender/traces/graffiti_font.py` / `jam_font.py`: generic words only.
- Run Blender ONLY like this: `flock /tmp/rotland-blender.lock blender -b --python-expr "import sys; sys.path.insert(0,'tools/blender'); from places import hospital; hospital.build_all()"` (adapt the path setup to how `build_r41_models.py` does it). There must be no `Traceback` or `FAILED`.
- Look at EVERY preview: the outside front, the outside back, and a top-down or cut-away of the ground floor (add a cut-away preview: render with the upper floors and roof hidden). Fix until it reads.

## The hospital
- **`metro_hospital`**: 38 x 30 m, 5 floors (ground 4.2 m + 4 x 3.8 m).
  - Outside:
    - a big ER canopy on columns over the front door, with an "EMERGENCY" sign and a red cross
    - a second, smaller side door
    - a ramp with rails
    - roof: AC units and a helipad H on the roof
  - Ground floor:
    - a reception hall with a long desk
    - a waiting area (rows of chairs)
    - a corridor
    - a ward of 6 beds with curtain rails
    - a pharmacy room behind a mesh cage (cage door open)
    - an office
    - toilets (just the room)
    - debris, an overturned bed, papers, dark stains, a fallen ceiling panel
  - Overgrowth: vines on the front.
- **Props:**
  - `gurney`, `hospital_bed`, `ward_curtain` (a torn curtain on a rail, 2 m)
  - `wheelchair`, `iv_stand`, `reception_desk_h`, `pharmacy_cage`, `waiting_chairs` (a row of 4)
  - `body_bag` (a single black bag)
  - `body_bag_pile` (15-20 bags heaped, 5 x 4 m, flies not needed)
  - `triage_tent` (an open-sided white tent, 6 x 4 m, red cross on the roof)
  - `cot` (a folding army cot)
  - `ambulance_wreck` (a boxy ambulance: doors open, a flat tyre, smashed windscreen, faded stripes, "AMBULANCE" in mirror letters on the hood)
- **`data/places/hospital.json`** (shape in the common file):
  - `crates`: 4 `med_cabinet` in the ward/pharmacy (`metro_hospital`), 1 `metro_locker` in the office (`metro_medical`).
  - `props`: gurneys/wheelchairs at the front, 1 `ambulance_wreck` under the canopy.
  - `yards.a` = the ambulance bay: 3 `ambulance_wreck` askew, gurneys, `body_bag`s.
  - `yards.b` = triage: 4 `triage_tent`s, `cot`s, `iv_stand`s, the `body_bag_pile` + 2 single bags.
  - `yards.c` = parking: wrecked cars (use the existing jam car models, e.g. `car_sedan_jam`; check their names in `assets/models/vehicles/`).
  - `yardCrates.b`: 2 `med_cabinet` in the triage tents.
  - All positions inside a 52 x 52 m lot (inside the sidewalks), on the ground, nothing overlapping.

## Done when
All models exist within budget with good previews (including the cut-away), and the JSON is complete and every model name in it exists. Fill `## Worker notes` (the `place_kit.py` functions, the cut-away preview paths).

## Worker notes
