# R19-07: stop the flicker (z-fighting) in the city buildings

Read first: `tasks/R19-common.md`, `tasks/R13-common.md` (Blender lane rules, the "No coplanar faces" rule), `tools/blender/buildings/kit.py` (read only), `tools/blender/buildings/apartment.py` (window details were moved 2 cm out of the wall on 2026-11-21: copy that fix style).
Files: tools/blender/buildings/apartment*.py, assets/models/buildings/apartment*, assets/models/_previews/apartment*, tools/blender/buildings/office*.py, assets/models/buildings/office*, assets/models/_previews/office*, tools/blender/buildings/police*.py, assets/models/buildings/police*, assets/models/_previews/police*, tools/blender/buildings/hospital*.py, assets/models/buildings/hospital*, assets/models/_previews/hospital*, tools/blender/buildings/shop*.py, assets/models/buildings/shop*, assets/models/_previews/shop*, tools/blender/buildings/pharmacy*.py, assets/models/buildings/pharmacy*, assets/models/_previews/pharmacy*, tools/blender/buildings/warehouse*.py, assets/models/buildings/warehouse*, assets/models/_previews/warehouse*, tools/blender/buildings/ruin*.py, assets/models/buildings/ruin*, assets/models/_previews/ruin*, tasks/R19-07-zfight-city.md
Test: (Blender lane: no cargo)

The owner sees buildings flicker in the game: two surfaces in the same plane facing the same way fight for the pixel. New checker: `python3 -I tools/find_coplanar.py <model.glb ...>` lists every overlapping same-plane pair with its area and position (x, y, z in the model's own space; y is up). Today's report for your models:
```
assets/models/buildings/apartment.glb: 1218 overlapping coplanar triangle pairs (79.27 m2)
assets/models/buildings/apartment_lod1.glb: 206 overlapping coplanar triangle pairs (5.63 m2)
assets/models/buildings/office.glb: 1466 overlapping coplanar triangle pairs (114.55 m2)
assets/models/buildings/office_lod1.glb: 148 overlapping coplanar triangle pairs (41.17 m2)
assets/models/buildings/police.glb: 1198 overlapping coplanar triangle pairs (15.25 m2)
assets/models/buildings/police_lod1.glb: 26 overlapping coplanar triangle pairs (222.51 m2)
assets/models/buildings/hospital.glb: 735 overlapping coplanar triangle pairs (29.29 m2)
assets/models/buildings/hospital_lod1.glb: 36 overlapping coplanar triangle pairs (629.68 m2)
assets/models/buildings/shop.glb: 390 overlapping coplanar triangle pairs (2.80 m2)
assets/models/buildings/shop_lod1.glb: 4 overlapping coplanar triangle pairs (0.00 m2)
assets/models/buildings/pharmacy.glb: 241 overlapping coplanar triangle pairs (2.49 m2)
assets/models/buildings/pharmacy_lod1.glb: 8 overlapping coplanar triangle pairs (0.12 m2)
assets/models/buildings/warehouse.glb: 276 overlapping coplanar triangle pairs (8.55 m2)
assets/models/buildings/warehouse_lod1.glb: 50 overlapping coplanar triangle pairs (4.82 m2)
assets/models/buildings/ruin.glb: 341 overlapping coplanar triangle pairs (69.95 m2)
assets/models/buildings/ruin_lod1.glb: OK
```
Models: apartment, office, police, hospital, shop, pharmacy, warehouse, ruin (and their `_lod1` versions where a `<name>_lod1.py` exists).

For each model:
1. Run the checker on the .glb, find the parts at the listed positions in the script, and fix them: details on a wall (windows, frames, sills, signs, stripes, trims, doors, posters) stick out at least 2 cm; boxes that are only partly visible (wall ends inside other walls, floor slabs under walls, roof caps) are shortened or moved 1-2 cm so their faces don't share a plane with a neighbour facing the same way; duplicate boxes are removed.
2. Rebuild: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/buildings/<name>.py` (and `<name>_lod1.py`). Check the preview PNG still looks the same (nothing moved visibly, no holes).
3. Re-run the checker. Goal: `OK`, or under 0.05 m2 left. Write each model's before/after area in Worker notes, plus any spot you left and why.
Don't change sizes, colors, colliders, markers (`lot_size`, `loot_N`, `spawn_N`) or triangle budgets.
Check scope: `python3 -I tools/gate_checks.py --base main --task tasks/R19-07-zfight-city.md` must print `scope: PASS`. Commit once.

## Worker notes
