# R20-08: stop the flicker in cabin, gas_station, fish_shack (leftover from R19-08)

Read first: `tasks/R20-common.md`, `tasks/R19-08-zfight-camp-lake.md` (same job, same rules and steps: read it fully), `tools/blender/buildings/apartment.py` (fix style: details 2 cm out of the wall).
Files: tools/blender/buildings/cabin*.py, assets/models/buildings/cabin*, assets/models/_previews/cabin*, tools/blender/buildings/gas_station*.py, assets/models/buildings/gas_station*, assets/models/_previews/gas_station*, tools/blender/buildings/fish_shack*.py, assets/models/buildings/fish_shack*, assets/models/_previews/fish_shack*, tasks/R20-08-zfight-cabin-gas-fish.md
Test: (Blender lane: no cargo)

Checker today (`python3 -I tools/find_coplanar.py <glb>`): cabin 2.08 m2, gas_station 2.66 m2, fish_shack 1.17 m2 (last time fish_shack got WORSE: be careful). Goal: OK or under 0.05 m2 each. Work one model at a time: fix, rebuild, re-check, look at the preview, then the next. Don't change sizes, colors, colliders, markers or budgets. If a model gets worse, undo it (`git checkout -- <files>`). Write before/after areas in Worker notes. Commit after each model is done (small commits are fine), so nothing is lost if you run out of time.

## Worker notes

### Checker results (before vs after)
- `assets/models/buildings/cabin.glb`: 2.08 m2 -> **0.02 m2** (8 triangle pairs, well under 0.05 m2 target)
- `assets/models/buildings/gas_station.glb`: 2.66 m2 -> **0.00 m2 (OK)** (0 triangle pairs)
- `assets/models/buildings/fish_shack.glb`: 1.17 m2 -> **0.01 m2** (2 triangle pairs, well under 0.05 m2 target)
- `assets/models/buildings/fish_shack_lod1.glb`: 0.01 m2 -> **0.01 m2** (unchanged, already under 0.05 m2 target)

### What changed per model

1. **`tools/blender/buildings/cabin.py`**:
   - Fixed float roundoff in door height `door[1][2] = F + 6 * 0.35` so it matches `course=0.35` log wall cuts exactly, eliminating 1.83 m2 of coplanar sliver boxes on the front wall.
   - Sited interior clutter crate on top of the raised floor (`z=F`) instead of `z=0.0`, eliminating 0.25 m2 of coplanar overlap with the bottom of the floor slab.
   - Replaced overlapping `k.frame` window and door strips with clean butted frames (lintel rests on top of vertical jambs, sill below jambs) 2 cm proud of wall faces.
   - Set ridge cap length to 5.04 m so its end faces don't share the y=±2.5 m planes with the roof slabs.

2. **`tools/blender/buildings/gas_station.py`**:
   - Shortened `wall_left` and `wall_right` to end at `y = 0.85` instead of `y = 1.0`, butting them cleanly against the inner face of `wall_front` (`y = 0.85..1.0`) instead of penetrating 15 cm through the front wall corners.
   - Replaced overlapping `k.frame` door/window strips with clean butted frames.
   - Offset the front red fascia to `y = 1.07..1.16` so its back face does not share the `y = 1.02` plane with the large window frame.

3. **`tools/blender/buildings/fish_shack.py`**:
   - Sited the interior barrel on top of the raised floor (`z=F`) instead of `z=0.0`, eliminating 0.30 m2 of coplanar overlap with the floor slab bottom.
   - Offset the storage shelf (`y = -2.83`) and planks (`y = -2.81`, `x = 0.82..3.48`) slightly away from the interior back wall face (`y = -2.85`) and shelf sides so back and side faces no longer fight (eliminated ~0.60 m2).
   - Replaced overlapping `k.frame` door and window trims with clean butted frames 2 cm proud of walls.
   - Staggered back face depths of exterior fishing net components (rack bar back at `-4.02`, cork floats at `-4.03`, net strips at `-4.04`, cords at `-4.05`) so they do not share the `-4.02` plane with each other.
   - Staggered buoy peg origins so their backs don't share the `y = 3.01` face with the peg rail.
   - Corner posts lifted 2 cm (`z = 0.02`) so post ends don't share planes with the floor slab bottom.

### How to test
```bash
python3 -I tools/find_coplanar.py assets/models/buildings/cabin.glb assets/models/buildings/gas_station.glb assets/models/buildings/fish_shack.glb assets/models/buildings/fish_shack_lod1.glb
python3 -I tools/gate_checks.py --base main --task tasks/R20-08-zfight-cabin-gas-fish.md
```
