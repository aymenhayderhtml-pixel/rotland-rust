# R13-03: city buildings shop + pharmacy

Read first: `tasks/R13-common.md`, `docs/CITY_SPEC.md` section 4, `tools/blender/buildings/house_small.py`, `tools/blender/buildings/gas_station.py`, `tools/blender/buildings/kit.py`.
Files: tools/blender/buildings/shop.py, tools/blender/buildings/shop_lod1.py, tools/blender/buildings/pharmacy.py, tools/blender/buildings/pharmacy_lod1.py, assets/models/buildings/shop*, assets/models/buildings/pharmacy*, assets/models/_previews/shop*, assets/models/_previews/pharmacy*, tasks/R13-03-*.md

Low-poly, flat colors, ruined and dusty but readable from far away. The lot size is the outer footprint (`lot_size` empty); keep the building 1 m inside it on every side. Ground at z=0, use `kit.FLOOR_TOP` like the other buildings.

## shop
Lot 18 x 10 m. Budget: `shop` 4000 triangles, `shop_lod1` 500 triangles.
a row of 2 shop fronts under one flat roof: big front windows (some broken, one boarded), a colored awning over each, a sign board above each (blank colored panels, no text). Inside: shelves and a counter, a wall between the two shops with one doorway. 4 loot (2 per shop), 2 spawns.

## pharmacy
Lot 12 x 10 m. Budget: `pharmacy` 3000 triangles, `pharmacy_lod1` 400 triangles.
small one-floor pharmacy: white/green walls, a green cross sign (made of boxes) above the door, front window, inside a counter and 3 shelf rows. 3 loot, 2 spawns.

## Worker notes
