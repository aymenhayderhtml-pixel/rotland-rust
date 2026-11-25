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

## Neon notes

* **What was written:**
* `data/roads.json`: road network configuration (`trailhead_margin: 60.0`, `trail_width: 4.0`, `highway_width: 8.0`, `point_spacing: 20.0`, `wiggle_m: 40.0`, `wiggle_frequency: 0.0012`, `ring_radius_from_towns: 1.0`).
* `world/src/roads/mod.rs`: `RoadKind`, `Road`, `RoadsData` (`load`, `load_from_file`, `from_json`), and `RoadNetwork` (`build`, `roads`, `distance_to_road`, `trailheads`).
* `world/src/roads/build.rs`: sector trailhead placement (`core_radius + blend_band + trailhead_margin` along each sector's `center_deg` via `WorldLayout::direction_at_deg`), sector-matched highways (`"Route to <Town name>"`), polar-arc ring trails joining towns in CCW angle order (`"<A> - <B> Trail"`) outside `core_radius + blend_band`, arc-length resampling (`~point_spacing`), and low-frequency simplex noise lateral wiggle (`seed.wrapping_add(307)`) tapered to `0` at both endpoints.
* `world/src/roads/index.rs`: 64 m spatial grid index (CSR dense grid with sparse fallback for huge coordinate spans) querying the 3×3 neighborhood around `p` with a 30 m cutoff (`distance_to_road`).
* `world/tests/roads_tests.rs`: integration tests covering trailhead radii/angles, road endpoints/names, radial clearance outside `core_radius + blend_band`, `distance_to_road` accuracy/cutoff, seed determinism vs. variation, and 100k `distance_to_road` queries.


* **Required lines for `world/src/lib.rs` (owned by R33-04):**
```rust
pub mod roads;
pub use roads::{Road, RoadKind, RoadNetwork, RoadsData};

```


* **Caller & integration notes:**
* `world/src/roads/build.rs` imports `crate::towns::TownSite` (from R33-03) and uses `biome`, `name`, and `center`.
* Ring trails join towns sorted counter-clockwise around `origin` relative to `layout.reference_dir()`, matching `desert (0°) -> swamp (90°) -> snow (180°) -> mountains (270°) -> desert (0°)` in `data/world_layout.json`.
