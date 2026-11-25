# R33-02: rivers (paths, carving, water levels, water mesh, crossings), pure logic

Owns: world/src/rivers/* (new: mod.rs, path.rs, carve.rs, mesh.rs), data/rivers.json (new), world/tests/rivers_tests.rs (new)
Files: world/src/rivers/mod.rs, world/src/rivers/path.rs, world/src/rivers/carve.rs, world/src/rivers/mesh.rs, data/rivers.json, world/tests/rivers_tests.rs
Read: tasks/R33-round.md, tasks/LEAD.md, docs/WORLD.md, world/src/lake/mesh.rs, world/src/lake/height.rs, world/src/noise.rs, world/src/biome/layout.rs
API: world/src/lib.rs, world/src/lake/mod.rs
Data: data/world_layout.json
Overlap: `world/src/lib.rs` belongs to R33-04: do not write it, report your `pub mod rivers;` / `pub use` lines to the lead. The `crossings()` input `RoadNetwork` comes from R33-01 (shared names only).
Rules: `tasks/LEAD.md` subagent rules. Use the "R33 shared names" in `tasks/R33-round.md` EXACTLY. Read `world/src/lake/mesh.rs` + `world/src/lake/height.rs` and copy their style. The water mesh must be the same triangle-list format the lake uses. Don't edit `world/src/lib.rs`: report your lines to the lead.

1. **Paths** (`data/rivers.json`): 3 rivers. Each starts in the mountains or the snow sector (high ground) and flows down toward the swamp or the desert edge, outside the core.
   - Recipe: start point at a sector angle (from data) at a radius of 3500-5000 m, then walk in steps of `step_m` (25 m). At each step, sample `base_height` (the terrain without rivers) at 5 directions within ±50° of the current heading, and take the lowest, with a bit of noise wiggle.
   - Stop after `max_length_m` (6000), or when it would enter `core_radius + blend_band + 150 m`: then turn along the core edge (keep the radius) instead of entering.
   - Width grows from `width_start` 6 m to `width_end` 16 m.
2. **Water level:** along the path, `level = min(previous level, bank height - depth)`, so it never flows uphill. Depth comes from data, 1.6 m.
3. **Carve:** `carve(x, z)`: within `width/2 + bank_m` (6 m) of the river line, the bed height is `level - depth*0.5` in the middle, rising smoothly to the bank. `blend` = 1 in the water, falling to 0 at the outer edge of the bank, so the generator can `lerp(terrain, bed, blend)`. Use a grid index like the roads (64 m cells of segment ids) so it stays fast.
4. **`water_at`:** the level where the point is over water, else None. Used for swimming/drinking and for the indoors/warmth "wet" checks later.
5. **`water_mesh_near(center, radius)`:** a strip of quads along each river segment within the radius (left/right edges at the current width + 1 m into the banks), y = water level at each point. Triangle list. Keep the vertex count under ~20k for radius 300.
6. **`crossings(roads)`:** every place a road segment crosses a river line: the position, the road heading and the span (river width + 2 × bank, rounded up to 12 or 16 m).
7. **Tests:**
   - the rivers never enter the core;
   - water levels never rise downstream;
   - the carve blend is 1 on the line and 0 beyond the bank;
   - `water_at` is Some on the line and None 30 m away;
   - the mesh is non-empty near a river and empty far away;
   - a crossing is found for a hand-made road across a hand-made river (build a small `RiverNetwork` from data in the test).

## Worker notes
