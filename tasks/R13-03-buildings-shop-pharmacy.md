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
- Implemented `shop` and `shop_lod1`:
  - `shop`: 3552 triangles (budget 4000). A row of two shop fronts with 16 x 8 m footprint on an 18 x 10 m lot (1 m setback on all sides). Features flat roof with parapet, rooftop AC units, two colored awnings (red and blue with white valances and metal brackets), blank colored sign boards, large front display windows (one boarded, one with broken pane details and mullions), 1.2 m doorways with metal frames, interior dividing wall with 1.2 m doorway, counters with cash registers, wall shelves, center gondola shelf, back wall shelf, crates, 4 loot empties (2 per shop) and 2 spawn empties.
  - `shop_lod1`: 240 triangles (budget 500). Matches footprint, outline, flat roof shape, AC units, awnings, sign boards, and front door/window panels with simplified geometry and no interior.
- Implemented `pharmacy` and `pharmacy_lod1`:
  - `pharmacy`: 1416 triangles (budget 3000). Small one-floor pharmacy with 10 x 8 m footprint on a 12 x 10 m lot (1 m setback on all sides). Features white and medical green walls with green roof parapet, rooftop HVAC and vent, 3D green cross box sign above door, corner projecting green cross blade sign, front display window with frame and boarded section, 1.2 m entrance door, L-shaped prescription counter with cash register, back wall medicine cabinets, 3 parallel shelf aisle rows with green endcaps, 3 loot empties and 2 spawn empties.
  - `pharmacy_lod1`: 216 triangles (budget 400). Matches footprint, white/green walls, green parapet, rooftop unit, green cross signs, and door/window facade panels with no interior.
- Previews verified:
  - `assets/models/_previews/shop.png`
  - `assets/models/_previews/shop_top.png`
  - `assets/models/_previews/shop_lod1.png`
  - `assets/models/_previews/pharmacy.png`
  - `assets/models/_previews/pharmacy_top.png`
  - `assets/models/_previews/pharmacy_lod1.png`
- Known issues: none.
- Scope check passed: `python3 -I tools/gate_checks.py --base main --task tasks/R13-03-buildings-shop-pharmacy.md` reports `scope: PASS`.
