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
