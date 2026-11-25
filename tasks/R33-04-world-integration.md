# R33-04: roads, rivers and towns in terrain, paint and chunk contents; bridges; split generator.rs

Owns: world/src/generator.rs, world/src/generator_world.rs (new), world/src/generator_contents.rs (new), world/src/terrain_shape.rs, world/src/terrain_paint.rs, world/src/lib.rs, world/src/poi.rs, world/tests/integration_r33_tests.rs (new)
Files: world/src/generator.rs, world/src/generator_world.rs, world/src/generator_contents.rs, world/src/terrain_shape.rs, world/src/terrain_paint.rs, world/src/lib.rs, world/src/poi.rs, world/tests/integration_r33_tests.rs
Read: tasks/R33-round.md, tasks/LEAD.md, docs/WORLD.md, world/src/generator.rs, world/src/terrain_shape.rs, world/src/terrain_paint.rs, world/src/poi.rs, world/src/lib.rs, world/tests/golden.rs, world/src/lake/place.rs, world/src/city/ground.rs
API: world/src/biome/layout.rs, world/src/lake/mod.rs, world/src/city/mod.rs, world/src/ground_paint.rs, world/src/scatter.rs
Data: data/world_layout.json, data/city_ground.json, data/poi.json
Overlap: none. R33-01/02/03 do not touch `world/src/lib.rs`; they report their `mod` and `pub use` lines to you, and you write them. `world/tests/golden.rs` must pass unedited.
Rules: `tasks/LEAD.md` subagent rules. Use the "R33 shared names" in `tasks/R33-round.md` EXACTLY. R33-01/02/03 write the roads, rivers and towns modules at the same time: code against the shared names. You own `world/src/lib.rs`, so add their `pub mod roads; pub mod rivers; pub mod towns;` lines and the `pub use` lines yourself.

1. **Split `generator.rs` (560 lines)** to under ~300 per file. Construction/attach code → `generator_world.rs`, `chunk_contents`/crates → `generator_contents.rs`. Keep every public method's signature the same.
2. **Build order** (in the constructor or attach step, AFTER camp/city/lake and the biome layout exist):
   1. `place_towns(...)`, with `flat_check` using the base height (max relief in the radius < 6 m);
   2. `RiverNetwork::build(...)`, with `base_height` = today's height function (biomes, no roads/rivers);
   3. `RoadNetwork::build(...)`.
   Store all three in the generator. **Skip all of this work for points in the core:** the core path must stay bit-for-bit the same (`world/tests/golden.rs` unedited must pass).
3. **Height** (outer only), in this order:
   1. Town flattening: inside `radius`, blend to the town's average height, with a smooth 30 m edge.
   2. River carve: `lerp(h, bed, blend)`.
   3. Road flattening: within `width/2 + 3 m`, blend toward the road's smoothed height (the average along the road ±40 m; precompute per road point).
   - Rivers win over roads except at bridge crossings: there the road keeps its height, and the bridge spans the river.
4. **Paint** (outer only):
   - highways: asphalt (copy the colours in `data/city_ground.json`);
   - trails: packed dirt;
   - riverbanks: wet mud/sand by biome (darker version of `ground.base`);
   - town streets: like highways; town lots: dirt.
5. **Chunk contents** (outer only):
   - Town buildings: each `TownLot` that falls in the chunk becomes a POI-like building entry. Find how lake village lots become POIs (`attach_lake` / `lake/place.rs`) and do the same, with `loot_table`.
   - Bridges: for each `crossings()` entry in the chunk, a `bridge_wood` (trail) or `bridge_concrete` (highway) prop at the road height, rotated to the road heading, scaled to the span (span / 12 or / 16).
   - Signposts: a `signpost` at each trailhead, facing outward.
   - Remove scattered props (trees/rocks) on roads, in rivers, and inside town lots. Use `distance_to_road`, `water_at` and the lot footprints in the scatter filter step that already removes props for the city/lake.
6. **`town_at` and `indoors`:** from the shared names. `indoors` checks town lot footprints and the existing POI footprints (read `poi.rs`; add a small public footprint check there if needed).
7. **Tests** (`integration_r33_tests.rs`):
   - the core heights are unchanged (sample 200 points within the core vs a generator built without R33 data, if possible; otherwise rely on golden and say so);
   - a town centre is flat (relief < 1 m within 40 m);
   - the height on a river line < the bank height;
   - a bridge prop exists at each crossing;
   - no tree within 3 m of a road point;
   - `indoors` is true at a lot centre and false in the street.

## Worker notes
