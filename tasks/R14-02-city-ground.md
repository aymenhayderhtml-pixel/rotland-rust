# R14-02: flatten the city ground and paint streets, sidewalks and the camp road

Read first: `tasks/R13-common.md` ("Rust/data lanes" part), `docs/CITY_SPEC.md` sections 1-2, `world/src/generator.rs` (`sample`, `emit_triangle`, `attach_city`, `city()`), `world/src/ground_paint.rs`, `world/src/city/layout.rs`, `world/src/city/streets.rs`, `world/src/settlement.rs` (`SettlementGround::flatten`: copy its blend style).
Files: world/src/city/ground.rs, world/src/city/ground_tests.rs, world/src/city/mod.rs, world/src/generator.rs, data/city_ground.json, tasks/R14-02-city-ground.md
Test: city

Only edit `sample` and `emit_triangle` (and the struct/constructor if you must) in `generator.rs`; another worker edits `chunk_contents`. All numbers and colors go in a new `data/city_ground.json` (loaded in `attach_city`'s caller path or a new `WorldGenerator::from_data_dir` line; colors as [r, g, b] 0-255).

1. **Flat city:** inside the city square the height is a plane: the raw height at the city center, tilted so the total change across the city is at most `maxSlope` from `data/city.json` (fit the tilt to the raw heights at the 4 corners, then clamp). Blend from the plane back to normal terrain over `edgeBlend` (48) m outside the square with a smooth step. Cache the plane in the city/generator so `sample` stays fast (it's called for every ground vertex).
2. **Road from camp:** along `city.road`, flatten the ground across `roadWidth` (8) m, blend over `roadBlend` (6) m to each side (height = the terrain height at the nearest point on the road's center line, smoothed: use the average of raw heights at ±8 m along the road). Skip inside the camp's flat lot.
3. **Paint** (new `world/src/city/ground.rs`, used from `emit_triangle`): road surface of every street = `asphalt` (dark gray), the sidewalk part (the outer `sidewalkWidth` (1.5) m on each side of the street) = `sidewalk` (light gray), lots and the rest of non-park blocks = `lotGround` (gray-brown concrete/dirt mix, broken up by the existing patch noise like camp dirt), Park blocks keep the normal grass. The camp road = `asphalt` with ragged edges (cracked, blending into `dirt` with the patch noise). Blocked segments look the same as other streets. The existing jitter/hue stay applied.
4. **The ground must stay green:** outside the city square + `edgeBlend` and off the road, height and color are exactly the same as before (the existing golden tests must still pass unchanged; do not edit them).
5. Tests in `world/src/city/ground_tests.rs` (`#[cfg(test)] mod ground_tests;` in `mod.rs`, names start with `city_`), using `WorldGenerator::from_data_dir(42, <repo>/data)`: a street center is asphalt-colored; a park block center is green (g > r and g > b); the city's height range is at most `maxSlope` + 0.5; a point 300 m outside the city and off the road has the same height and color as a `WorldGenerator` built with no city (`with_configs` with the same data); the road midpoint is asphalt-colored.

## Worker notes

