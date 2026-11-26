# R20-07: stop the flicker in boathouse, house_small, barn (leftover from R19-08)

Read first: `tasks/R20-common.md`, `tasks/R19-08-zfight-camp-lake.md` (same job, same rules and steps: read it fully), `tools/blender/buildings/apartment.py` (fix style: details 2 cm out of the wall).
Files: tools/blender/buildings/boathouse*.py, assets/models/buildings/boathouse*, assets/models/_previews/boathouse*, tools/blender/buildings/house_small*.py, assets/models/buildings/house_small*, assets/models/_previews/house_small*, tools/blender/buildings/barn*.py, assets/models/buildings/barn*, assets/models/_previews/barn*, tasks/R20-07-zfight-boathouse-house-barn.md
Test: (Blender lane: no cargo)

Checker today (`python3 -I tools/find_coplanar.py <glb>`): boathouse 11.51 m2, house_small 8.28 m2, barn 2.96 m2. Goal: OK or under 0.05 m2 each. Work one model at a time: fix, rebuild, re-check, look at the preview, then the next. Don't change sizes, colors, colliders, markers or budgets. If a model gets worse, undo it (`git checkout -- <files>`). Write before/after areas in Worker notes. Commit after each model is done (small commits are fine), so nothing is lost if you run out of time.

## Worker notes
- **Areas before / after (`python3 -I tools/find_coplanar.py <glb>`):**
  - `barn.glb`: **2.96 m2 -> 0.01 m2** (2 pairs left: 0.007 m2 ridge slab intersection at eaves)
  - `house_small.glb`: **8.28 m2 -> 0.02 m2** (2 pairs left: 0.008 m2 ridge slab intersection at eaves)
  - `boathouse.glb`: **11.51 m2 -> 0.01 m2** (2 pairs left: 0.004 m2 ridge slab intersection at eaves)
  - `boathouse_lod1.glb`: **0.03 m2** (untouched, already below 0.05 m2)

- **Fixes applied per model:**
  - `barn.py`:
    - Gables: adjusted X extents from `((-5.38, 0.13), (5.25, 0.13))` to `((-5.38, 0.11), (5.27, 0.11))` so gable inner faces stop 2 cm before inner wall planes at `x = ±5.25` (fixed 2.10 m2 overlap).
    - Door frame: removed duplicate white header bar `k.box((-2.6, 4.04, 3.6), (2.6, 4.08, 3.7), "white")` and replaced `k.frame` with clean non-overlapping jambs (ending at `z = 3.6`) and header beam (spanning `[-2.72, 2.72]` from `z = 3.6` to `3.72`) to eliminate corner overlap.
  - `house_small.py`:
    - Window Z alignment: computed `WIN_Z0 = F + 3 * 0.3` and `WIN_Z1 = F + 6 * 0.3` matching wall `extra` cuts (`lo[2] + k * course`). This fixed a float precision bug (`0.1 + 0.9 = 0.9999999999999999` vs `1.0`) that had created 1e-16 m degenerate wall cells across all 4 walls (~7.9 m2 overlap).
    - Window/door trim: implemented clean non-overlapping `frame()` where jambs run between `lo[2]` and `hi[2]`, head runs above `hi[2]`, and sill runs below `lo[2]` (eliminated 0.11 m2 corner overlap).
    - Roof width: adjusted roof slab width from 9.0 to 9.06 (3 cm overhang beyond walls at `x = ±4.5`) so the roof slab side faces do not share planes with the exterior side walls.
  - `boathouse.py`:
    - Door/window Z alignment: set `DOOR_H = F + 6 * 0.35` matching wall course cuts to eliminate float precision coplanar slicing at `z = 2.2` (~9 m2 overlap).
    - Big boat door timber frame: side posts now start at `z = 0.08` (inside floor slab) to prevent bottom face coplanar with wall at `F = 0.1`; header beam sits between posts at `z in [2.93, 3.15]` instead of `[2.95, 3.15]` so it doesn't coplanar with the wall header at `z = 2.95`.
    - Pilings: top frustum height adjusted to `z = 0.08` (inside floor slab) so piling top circles don't coplanar with walkway top at `z = F = 0.1`.
    - Shelves: replaced full solid block visual with upright corner posts and shelf boards inset 2 cm from wall at `x = -6.85`; supplies sit proud at `z = 1.16`.
    - Barrel: placed on walkway floor (`z = F`) rather than penetrating into ground (`z = 0.0`).
    - Workbench: lip inset to `-4.83` (2 cm from wall inner face at `-4.85`); tackle box and lantern sit proud at `z = 0.95`.
    - Ceiling beams: shortened X extents from `±7.0` to `±6.84` so beam ends sit inside the walls and do not coplanar with wall outer faces at `x = ±7.0`.
    - Window/door trim: clean non-overlapping `frame()` helper.

- **How to verify in game / tools:**
  - Run `python3 -I tools/find_coplanar.py assets/models/buildings/barn.glb assets/models/buildings/house_small.glb assets/models/buildings/boathouse.glb assets/models/buildings/boathouse_lod1.glb` -> all models <= 0.03 m2.
  - Run `python3 -I tools/gate_checks.py --base main --task tasks/R20-07-zfight-boathouse-house-barn.md` -> scope: PASS, numbers: PASS, unused: PASS.
