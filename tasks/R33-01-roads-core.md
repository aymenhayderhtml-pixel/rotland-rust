# R33-01: road network (pure world logic + data)

Owns: world/src/roads/* (new: mod.rs, build.rs, index.rs), data/roads.json (new), world/tests/roads_tests.rs (new)
Files: world/src/roads/mod.rs, world/src/roads/build.rs, world/src/roads/index.rs, data/roads.json, world/tests/roads_tests.rs
Read: tasks/R33-round.md, tasks/LEAD.md, docs/WORLD.md, world/src/biome/layout.rs, world/src/noise.rs
API: world/src/lib.rs, world/src/biome/mod.rs
Data: data/world_layout.json
Overlap: `world/src/lib.rs` belongs to R33-04 (it writes the `pub mod roads;` and `pub use` lines). Do not write it: report your lines to the lead. `TownSite` comes from R33-03: code against the shared names.
Rules: `tasks/LEAD.md` subagent rules. Use the "R33 shared names" in `tasks/R33-round.md` EXACTLY. R33-03 makes `TownSite` at the same time; code against the shared names. Don't edit `world/src/lib.rs`: report your `pub mod roads;` / `pub use` lines to the lead.

1. **Trailheads:** one per sector, at angle = the sector `center_deg` (from `WorldLayout`; read `world/src/biome/layout.rs` for how angles and `reference_dir` work), at distance `core_radius + blend_band + trailhead_margin` (60 m) from `origin`.
2. **Roads** (`data/roads.json`: `trailhead_margin`, `trail_width` 4, `highway_width` 8, `point_spacing` 20, `wiggle_m` 40, `wiggle_frequency`, `ring_radius_from_towns`):
   - **Highways:** from each trailhead to the town of that sector (match by biome); the town center is the end.
   - **Ring:** trails joining the 4 towns in angle order (desert → swamp → snow → mountains → desert). Each trail segment must stay outside `core_radius + blend_band` the whole way: arc around the core, never cut through it. Test it.
   - **Shape:** a straight base path, resampled every `point_spacing` m, pushed sideways by low-frequency noise of up to `wiggle_m` (zero at both ends, so it still meets the trailhead and the town). Use `crate::noise` with a new, unused seed offset (`wrapping_add`).
   - `name`: highways "Route to <Town name>", ring trails "<A> - <B> Trail".
3. **Spatial index** (`index.rs`): a grid of 64 m cells storing the segment ids that touch each cell. `distance_to_road` checks the 3×3 cells around the point, so it stays fast (it is called ~1,100 times per chunk).
4. **Tests:**
   - 4 trailheads at the right radius;
   - every road starts and ends where it should;
   - no road point lies inside `core_radius + blend_band`;
   - `distance_to_road` gives ≈0 on a road point, ≈ half the offset beside it, and None 100 m away;
   - same seed gives the same network, a different seed changes the wiggles;
   - a speed test: 100k `distance_to_road` queries finish fast (no timing assert, just make sure it runs; no `#[ignore]`).

## Worker notes
