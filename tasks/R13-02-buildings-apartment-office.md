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
