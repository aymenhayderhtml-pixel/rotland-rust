# R19-08: stop the flicker (z-fighting) in the camp, lake and countryside buildings

Read first: `tasks/R19-common.md`, `tasks/R13-common.md` (Blender lane rules, the "No coplanar faces" rule), `tools/blender/buildings/kit.py` (read only), `tools/blender/buildings/apartment.py` (window details were moved 2 cm out of the wall on 2026-11-21: copy that fix style).
Files: tools/blender/buildings/camp_doctor*.py, assets/models/buildings/camp_doctor*, assets/models/_previews/camp_doctor*, tools/blender/buildings/camp_gate_concrete*.py, assets/models/buildings/camp_gate_concrete*, assets/models/_previews/camp_gate_concrete*, tools/blender/buildings/camp_gate_metal*.py, assets/models/buildings/camp_gate_metal*, assets/models/_previews/camp_gate_metal*, tools/blender/buildings/camp_gate_wood*.py, assets/models/buildings/camp_gate_wood*, assets/models/_previews/camp_gate_wood*, tools/blender/buildings/camp_mechanic*.py, assets/models/buildings/camp_mechanic*, assets/models/_previews/camp_mechanic*, tools/blender/buildings/camp_tower_metal*.py, assets/models/buildings/camp_tower_metal*, assets/models/_previews/camp_tower_metal*, tools/blender/buildings/camp_tower_wood*.py, assets/models/buildings/camp_tower_wood*, assets/models/_previews/camp_tower_wood*, tools/blender/buildings/camp_wall_concrete*.py, assets/models/buildings/camp_wall_concrete*, assets/models/_previews/camp_wall_concrete*, tools/blender/buildings/camp_wall_metal*.py, assets/models/buildings/camp_wall_metal*, assets/models/_previews/camp_wall_metal*, tools/blender/buildings/camp_wall_wood*.py, assets/models/buildings/camp_wall_wood*, assets/models/_previews/camp_wall_wood*, tools/blender/buildings/camp_workshop_1*.py, assets/models/buildings/camp_workshop_1*, assets/models/_previews/camp_workshop_1*, tools/blender/buildings/camp_workshop_2*.py, assets/models/buildings/camp_workshop_2*, assets/models/_previews/camp_workshop_2*, tools/blender/buildings/camp_workshop_3*.py, assets/models/buildings/camp_workshop_3*, assets/models/_previews/camp_workshop_3*, tools/blender/buildings/boathouse*.py, assets/models/buildings/boathouse*, assets/models/_previews/boathouse*, tools/blender/buildings/fish_shack*.py, assets/models/buildings/fish_shack*, assets/models/_previews/fish_shack*, tools/blender/buildings/barn*.py, assets/models/buildings/barn*, assets/models/_previews/barn*, tools/blender/buildings/cabin*.py, assets/models/buildings/cabin*, assets/models/_previews/cabin*, tools/blender/buildings/gas_station*.py, assets/models/buildings/gas_station*, assets/models/_previews/gas_station*, tools/blender/buildings/house_small*.py, assets/models/buildings/house_small*, assets/models/_previews/house_small*, tools/blender/buildings/shed*.py, assets/models/buildings/shed*, assets/models/_previews/shed*, tools/blender/buildings/watchtower*.py, assets/models/buildings/watchtower*, assets/models/_previews/watchtower*, tasks/R19-08-zfight-camp-lake.md
Test: (Blender lane: no cargo)

The owner sees buildings flicker in the game: two surfaces in the same plane facing the same way fight for the pixel. New checker: `python3 -I tools/find_coplanar.py <model.glb ...>` lists every overlapping same-plane pair with its area and position (x, y, z in the model's own space; y is up). Today's report for your models:
```
assets/models/buildings/camp_doctor.glb: 331 overlapping coplanar triangle pairs (4.26 m2)
assets/models/buildings/camp_gate_concrete.glb: 273 overlapping coplanar triangle pairs (2.09 m2)
assets/models/buildings/camp_gate_metal.glb: 548 overlapping coplanar triangle pairs (2.10 m2)
assets/models/buildings/camp_gate_wood.glb: 279 overlapping coplanar triangle pairs (3.04 m2)
assets/models/buildings/camp_mechanic.glb: 316 overlapping coplanar triangle pairs (7.13 m2)
assets/models/buildings/camp_tower_metal.glb: 323 overlapping coplanar triangle pairs (0.58 m2)
assets/models/buildings/camp_tower_wood.glb: 353 overlapping coplanar triangle pairs (1.85 m2)
assets/models/buildings/camp_wall_concrete.glb: 1 overlapping coplanar triangle pairs (0.00 m2)
assets/models/buildings/camp_wall_metal.glb: 73 overlapping coplanar triangle pairs (0.18 m2)
assets/models/buildings/camp_wall_wood.glb: 48 overlapping coplanar triangle pairs (0.11 m2)
assets/models/buildings/camp_workshop_1.glb: 70 overlapping coplanar triangle pairs (0.15 m2)
assets/models/buildings/camp_workshop_2.glb: 106 overlapping coplanar triangle pairs (0.20 m2)
assets/models/buildings/camp_workshop_3.glb: 105 overlapping coplanar triangle pairs (1.03 m2)
assets/models/buildings/boathouse.glb: 396 overlapping coplanar triangle pairs (11.68 m2)
assets/models/buildings/boathouse_lod1.glb: 4 overlapping coplanar triangle pairs (0.03 m2)
assets/models/buildings/fish_shack.glb: 200 overlapping coplanar triangle pairs (1.17 m2)
assets/models/buildings/fish_shack_lod1.glb: 2 overlapping coplanar triangle pairs (0.01 m2)
assets/models/buildings/barn.glb: 54 overlapping coplanar triangle pairs (5.78 m2)
assets/models/buildings/cabin.glb: 158 overlapping coplanar triangle pairs (5.66 m2)
assets/models/buildings/gas_station.glb: 207 overlapping coplanar triangle pairs (2.66 m2)
assets/models/buildings/house_small.glb: 254 overlapping coplanar triangle pairs (11.96 m2)
assets/models/buildings/shed.glb: 42 overlapping coplanar triangle pairs (1.70 m2)
assets/models/buildings/watchtower.glb: 344 overlapping coplanar triangle pairs (1.03 m2)
```
Models: camp_doctor, camp_gate_concrete, camp_gate_metal, camp_gate_wood, camp_mechanic, camp_tower_metal, camp_tower_wood, camp_wall_concrete, camp_wall_metal, camp_wall_wood, camp_workshop_1, camp_workshop_2, camp_workshop_3, boathouse, fish_shack, barn, cabin, gas_station, house_small, shed, watchtower (and their `_lod1` versions where a `<name>_lod1.py` exists).

For each model:
1. Run the checker on the .glb, find the parts at the listed positions in the script, and fix them: details on a wall (windows, frames, sills, signs, stripes, trims, doors, posters) stick out at least 2 cm; boxes that are only partly visible (wall ends inside other walls, floor slabs under walls, roof caps) are shortened or moved 1-2 cm so their faces don't share a plane with a neighbour facing the same way; duplicate boxes are removed.
2. Rebuild: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/buildings/<name>.py` (and `<name>_lod1.py`). Check the preview PNG still looks the same (nothing moved visibly, no holes).
3. Re-run the checker. Goal: `OK`, or under 0.05 m2 left. Write each model's before/after area in Worker notes, plus any spot you left and why.
Don't change sizes, colors, colliders, markers (`lot_size`, `loot_N`, `spawn_N`) or triangle budgets.
Check scope: `python3 -I tools/gate_checks.py --base main --task tasks/R19-08-zfight-camp-lake.md` must print `scope: PASS`. Commit once.

## Worker notes
- (Claude, 2026-11-22) The worker hit its Gemini quota before committing. Claude rebuilt every edited script and re-ran the checker. Kept: all 13 camp models (now OK or <= 0.05 m2), shed (1.70 -> 0.01), watchtower (OK), barn (5.78 -> 2.96), cabin (5.66 -> 2.08), house_small (11.96 -> 8.28), boathouse (11.68 -> 11.51). Reverted fish_shack (got worse: 1.17 -> 2.97). STILL TO DO next round: boathouse, house_small, barn, cabin, gas_station (2.66, untouched), fish_shack.
