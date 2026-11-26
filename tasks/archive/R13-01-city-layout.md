# R13-01: city layout (pure code, world crate)

Read first: `tasks/R13-common.md`, `docs/CITY_SPEC.md` sections 1-3, `world/src/settlement.rs` (style of a pure placer plus a JSON config), `world/src/lib.rs`.
Files: world/src/city/*, world/src/lib.rs, data/city.json, tasks/R13-01-city-layout.md
Test: city

Build the street grid and lots of the ruined city as pure, deterministic code. No rendering, no wiring into the game (Claude does that on day 5).

## data/city.json (all numbers live here)
Keys (camelCase, like the other data files):
- `distanceFromCamp` 1500 (metres from the camp center to the nearest city edge), `blocks` 10, `blockSize` 64, `streetWidth` 10, `avenueWidth` 14, `avenueEvery` 3, `blockedChance` 0.25, `cornerDropChance` 0.5, `maxSlope` 3, `directionTries` 8, `coreBlocks` 2, `outskirtsRings` 2, `industrialStrip` [2, 6], `parks` [1, 2].
- `districts`: for each of `outskirts`, `downtown`, `core`, `industrial`, a list of building ids with weights. Use: outskirts = house_small 4, gas_station 1, shop 2, shed 1; downtown = apartment 4, shop 3, office 2, pharmacy 1, ruin 1; core = police 1, hospital 1, ruin 3, office 2, apartment 2; industrial = warehouse 4, gas_station 1, ruin 1, shed 2.
- `buildings`: lot size [width, depth] in metres: apartment [20,14], shop [18,10], office [18,18], pharmacy [12,10], police [24,18], hospital [30,22], warehouse [26,16], ruin [16,12], house_small [9,7], gas_station [10,8], shed [3,3].
- `lotGap` 2 (minimum metres between lots and from the sidewalk).

## world/src/city/ (several files, each under ~300 lines)
- `CityConfig` (serde, loads `data/city.json`, `load_from_file` and `from_json` like `SettlementConfig`).
- `District` enum: Outskirts, Downtown, Core, Industrial, Park.
- `CityLayout::generate(seed: i32, camp: Vec2, cfg: &CityConfig, forest_at: impl Fn(Vec2) -> f32) -> CityLayout`:
  1. Direction: try `directionTries` evenly spaced directions (start angle from the seed). For each, the city square sits so its nearest edge is `distanceFromCamp` from the camp. Score = average `forest_at` over a 5x5 sample grid in the square (lower is better). Pick the best. The square's min corner snaps to the 64 m chunk grid (so blocks = chunks).
  2. Blocks: `blocks` x `blocks`. Each outer-ring CORNER block is dropped with `cornerDropChance` (ragged edge). District per block: the center `coreBlocks` x `coreBlocks` = Core; outer `outskirtsRings` rings = Outskirts; the rest Downtown. Then one Industrial strip of `industrialStrip` blocks along one side, inside the outskirts rings (seeded side), and `parks` random Downtown blocks become Park.
  3. Streets: a segment along every block edge that touches a present block. A segment's width is `avenueWidth` when its grid line index % `avenueEvery` == 0, else `streetWidth`. Streets lie centered on the block boundary lines. Each segment is `blocked` with `blockedChance`, BUT keep the city connected for cars: after marking, every present block must still be reachable from the street piece nearest the camp through unblocked segments (unblock segments until true). Avenues are never blocked.
  4. Lots: inside each non-park block, the usable square is the block minus half the street widths minus `lotGap`. Pick building ids by the district weights and place lots along the block's sides, facing the nearest street (yaw so the building front faces the street; Godot yaw convention from AGENTS.md rule 7), with no overlaps and fully inside the usable square. Stop when nothing more fits (try each pick a few times, then move on).
- `CityLayout` fields (pub): `origin: Vec2` (min corner), `size: f32`, `blocks: Vec<CityBlock>` (`coord: IVec2` chunk coord, `district`, `present: bool`), `streets: Vec<StreetSegment>` (`a: Vec2`, `b: Vec2`, `width`, `blocked`, `avenue`), `lots: Vec<CityLot>` (`building_id: String`, `center: Vec2`, `size: Vec2`, `yaw`, `district`), `entry: Vec2` (the street point nearest the camp).
- Queries: `contains(p: Vec2) -> bool`, `district_at(p) -> Option<District>`, `street_at(p) -> Option<&StreetSegment>` (p is on that street's surface).

## Tests (in `world/src/city/tests.rs` as a `#[cfg(test)] mod tests;`, test names start with `city_`; no new files under world/tests/)
Same seed gives the same layout; different seeds differ. The nearest city edge is `distanceFromCamp` (±64 m because of the snap) from the camp. No two lots overlap; every lot is inside its block and off the streets. Every present block is reachable through unblocked streets from `entry`. About `blockedChance` of the non-avenue segments are blocked over many seeds (within ±0.08). Direction avoids forest: with a `forest_at` that is 1.0 for x > 0, the city is placed at x < 0. Exactly one Core 2x2 area, one industrial strip.

## Worker notes
- Files created/changed:
  - `data/city.json`: city parameters, district tables, building lot dimensions, street widths and probabilities.
  - `world/src/city/config.rs`: `CityConfigData` (`CityConfig`), `District`, `CityDistrictsData`, `BuildingWeight`, JSON loading and validation.
  - `world/src/city/streets.rs`: `StreetSegment`, street grid placement along block boundaries, avenues, entry finding, and 0-1 BFS connectivity ensuring all present blocks are reachable.
  - `world/src/city/lots.rs`: `CityLot`, lot placement along block perimeters with Godot yaw convention, non-overlap, and sidewalk clearance.
  - `world/src/city/layout.rs`: `CityBlock`, `CityLayout::generate`, forest-avoidance direction scoring, 64 m chunk snapping, ragged corner drops, district distribution (Core, Outskirts, Downtown, Industrial, Park), and spatial queries (`contains`, `district_at`, `street_at`).
  - `world/src/city/mod.rs`: module re-exports.
  - `world/src/city/tests.rs`: 9 comprehensive unit tests verifying determinism, camp distance, lot separation/containment, street graph reachability, blocked street ratio, forest avoidance, districts, and queries.
  - `world/src/lib.rs`: exported `city` module and public types.
- How to test:
  `tools/gate.sh --task tasks/R13-01-city-layout.md`
- Wiring notes for Claude:
  - Call `CityConfig::load_from_file(paths::repo_path("data/city.json"))` to load configuration.
  - Generate the city using `CityLayout::generate(seed, camp_center.xz(), &city_cfg, |pos| generator.forest_weight(pos.x, pos.y))`.
  - Blocks align with chunk boundaries (`coord: IVec2` gives the chunk coordinate).
  - Lots provide `center: Vec2`, `size: Vec2` (width, depth), `yaw: f32` (facing street), `district: District`, and `building_id: String`.
  - Streets provide `a`, `b`, `width`, `blocked`, and `avenue`. `entry: Vec2` provides the road hookup point nearest camp.
- Known issues:
  None. Clippy, test suite, scope, numbers, and unused items checks all pass clean.

## Gate report
- date: 2026-11-21
- base: main
- files changed (9): data/city.json, tasks/R13-01-city-layout.md, world/src/city/config.rs, world/src/city/layout.rs, world/src/city/lots.rs, world/src/city/mod.rs, world/src/city/streets.rs, world/src/city/tests.rs, world/src/lib.rs
- tests passed: 9
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
