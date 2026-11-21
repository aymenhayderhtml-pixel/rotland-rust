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
