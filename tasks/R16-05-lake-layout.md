# R16-05: lake & coast layout (pure, world crate)

Read first: `tasks/R16-common.md`, `docs/DECISIONS.md` rows dated 2026-11-21, `world/src/city/mod.rs` + `world/src/city/layout.rs` + `world/src/city/ground.rs` (copy this pattern: a pure layout attached to the generator), `world/src/generator.rs` (`attach_city`, `sample`, `emit_triangle`, `chunk_contents`).
Files: world/src/lake/*, world/src/lib.rs, data/lake.json, tasks/R16-05-lake-layout.md
Test: lake

The lake & coast region (owner: opposite the city, about 1.5 km from the camp). This task is the pure layout only; Claude attaches it to the generator on day 2 (so do NOT edit `generator.rs`).
- `LakeConfig` from `data/lake.json`: `distanceFromCamp` 1500, `radius` [260, 200] (ellipse half axes, m), `shoreNoise` amplitude/frequency (wobbly coastline), `beachWidth` 14, `shallowWidth` 12 (wading), `maxDepth` 8, `waterLevelAboveShore` 0.3, `village` lots (count 8, sizes, along one shore), `docks` 3 (piers 4 x 20 m out into the water), `angleFromCity` 180 (degrees; opposite).
- `LakeLayout::generate(seed, camp: Vec2, city_center: Option<Vec2>, cfg, raw_height: impl Fn(Vec2)->f32) -> LakeLayout`: center at `distanceFromCamp + radius` from the camp, in the direction opposite the city (or seeded when no city). Fields: `center`, `radius`, `water_level` (shore raw height + `waterLevelAboveShore`), `shore` polyline (sampled), `docks` (start, end, yaw), `village_lots` (center, size, yaw facing the water), `fishing_spots` (on docks and beaches).
- Queries (fast, called per vertex): `shore_distance(p) -> f32` (signed: negative in water), `depth_at(p) -> f32` (0 at the shore line, growing to `maxDepth` toward the middle, smooth), `zone_at(p) -> LakeZone { Land, Beach, Shallow, Deep }`, `ground_height(p, raw) -> f32` (carves the lake bed: terrain blends down to `water_level - depth` inside, and to a flat beach just above the water on the beach band; unchanged outside the region), `contains_region(p)` (bounding box + margin, for early-outs).
Tests (in `world/src/lake/tests.rs` as `#[cfg(test)] mod tests;`, names start `lake_`): deterministic; opposite side of the camp from a given city; distance from camp ~1500 at the nearest shore; depth 0 at the shore and max near the middle; zones in the right order walking from land to the middle; docks start on land and end over water; village lots on land, not in water, not overlapping; far points unchanged by `ground_height`.

## Worker notes

