# R14-01: put the city's buildings and street props into the world chunks

Read first: `tasks/R13-common.md` ("Rust/data lanes" part), `docs/CITY_SPEC.md` sections 2-5, `world/src/generator.rs` (`chunk_contents`, `attach_city`, `city()`), `world/src/city/mod.rs` and the files it lists, `world/src/poi.rs`, `world/src/scatter.rs` (`PropInstance`), `data/poi.json`.
Files: world/src/city/place.rs, world/src/city/place_tests.rs, world/src/city/mod.rs, world/src/generator.rs, data/poi.json, data/city.json, tasks/R14-01-city-chunk-contents.md
Test: city

`WorldGenerator::from_data_dir` already builds `CityLayout` (`generator.city()`): `lots`, `streets` (with `blocked`, `avenue`, `width`), `blocks` (district, park), `road` (camp to `entry`). Nothing in the city shows up in the world yet. Make `chunk_contents(coord)` return the city's buildings and props. Only `chunk_contents` changes in `generator.rs` (another worker is editing `sample` and `emit_triangle`; don't touch those).

1. **Buildings:** each `CityLot` becomes one `PoiInstance` in the chunk that contains its center (id `city_<index>`, `building_id`, position = (center.x, `self.height(x,z)`, center.y), yaw = lot yaw). Add the 8 city buildings to `data/poi.json` `buildings` (lot from `data/city.json` `buildings`, lootTable = the same name as the id (tables already exist in `data/loot_tables.json`), zombies [1,3], police and hospital [3,5], `chancePerChunk` 0 so they are never placed in the wild, biome `{}` or whatever makes the wild placer skip them, check `PoiPlacer`). Reused ids (house_small, gas_station, shed) already exist.
2. **Clear the wild stuff inside the city:** inside `city.contains(p)`, drop the randomly placed POIs and all scatter props, except trees and ground cover in Park blocks. Drop scatter props that sit on any street or within a lot (use `PoiGeometry::remove_cleared` for lots, the same way the code does today).
3. **Street props** (new `world/src/city/place.rs`, pure, deterministic from seed + chunk coord, each prop placed only in the chunk that contains it). Numbers go into a new `"props"` object in `data/city.json`:
   - `street_lamp` along sidewalks, every `lampSpacing` (24) m, on alternating sides, at the sidewalk edge (1 m inside the street edge), facing the street.
   - On every `blocked` segment: one blocker across the middle of the road, picked by weight from `blockers` (barricade 3, car_wreck_sedan 2, rubble_pile 2, jersey_barrier 2), plus 1-2 more blocker pieces beside it so the full street width is closed for a car.
   - Clutter on streets/sidewalks, per segment a count from `clutterPerSegment` [1, 3], picked by weight from `clutter` (car_wreck 2, car_wreck_sedan 2, car_wreck_van 1, trash_bags 3, dumpster 2, mailbox 1, sign_fallen 1, tire_pile 1, barrel_rusty 1); wrecks only on the road part, the rest on the sidewalk part. Never closer than 3 m to another prop.
   - `bus_stop`: on avenues, every `busStopSpacing` (128) m, on the sidewalk, facing the street.
   - `fence_broken`: 0-2 per Outskirts block, on the lot side of the sidewalk.
   - Along `road` (camp to city): a car wreck every `roadWreckSpacing` (150) m, 2 m off the road center, random yaw; none within the camp's clear radius.
   Prop y = `generator.height(x, z)`. Scale 1.0.
4. Tests in `world/src/city/place_tests.rs` (as `#[cfg(test)] mod place_tests;` in `mod.rs`; names start with `city_`): with `WorldGenerator::from_data_dir(42, <repo>/data)`: every lot shows up in exactly one chunk's POIs; no wild POI or non-park scatter inside the city; every blocked segment has at least one blocker prop within its width; no prop within 3 m of another; a chunk far from the city (e.g. the camp chunk) gives the same POIs and props as before this change except road wrecks; the same seed gives the same props.

## Worker notes
- **Files changed**:
  - `data/poi.json`: Added the 8 city buildings (`apartment`, `shop`, `office`, `pharmacy`, `police`, `hospital`, `warehouse`, `ruin`) with lot dimensions matching `data/city.json`, corresponding loot tables from `data/loot_tables.json`, zombie counts ([1, 3] or [3, 5]), and `chancePerChunk: 0.0`.
  - `data/city.json`: Added `"props"` configuration object containing `lampSpacing`, `lampSidewalkInset`, `busStopSpacing`, `roadWreckSpacing`, `roadWreckOffset`, `minPropDistance`, `sidewalkWidth`, `blockers`, `clutterPerSegment`, `clutter`, and `fenceBrokenPerBlock`.
  - `world/src/city/place.rs`: Pure, deterministic generation of street lamps, road blockers, bus stops, street/sidewalk clutter, outskirts broken fences, and road wrecks per chunk.
  - `world/src/city/mod.rs`: Exported `place` module and `place_street_props`, registered `place_tests` module.
  - `world/src/generator.rs`: Updated `chunk_contents(coord)` to drop wild POIs and non-park scatter inside the city, add `PoiInstance` for each `CityLot` containing its center, clear props inside lots using `PoiGeometry::remove_cleared`, clear props within settlement radius, and append city street props and road wrecks with >= 3.0 m spacing.
  - `world/src/city/place_tests.rs`: Added 6 unit tests covering:
    - `city_lots_show_up_in_exactly_one_chunk`: every lot appears in exactly one chunk's POIs.
    - `city_no_wild_poi_or_non_park_scatter_inside_city`: no wild POI or non-park scatter inside the city.
    - `city_blocked_segments_have_blockers`: every blocked segment contains at least one blocker prop within its width.
    - `city_no_prop_within_3m`: no street prop is within 3 m of any other prop.
    - `city_camp_chunk_matches_without_city`: camp chunk gives identical POIs and props to the base generator.
    - `city_deterministic_same_seed`: identical seeds yield identical chunk contents across all sampled coords.
- **Wiring notes for Claude**:
  - `generator.chunk_contents(coord)` now yields both city buildings (`city_{index}`) and street/road props (`street_lamp`, `bus_stop`, `fence_broken`, `barricade`, `rubble_pile`, `jersey_barrier`, car wrecks, etc.).
  - Outside of city/road chunks, `chunk_contents(coord)` remains identical to the previous behavior.
- **Known issues**:
  - None. All 16 city tests pass, clippy is clean with 0 warnings, scope and number source checks pass.

## Gate report
- date: 2026-11-21
- base: main
- files changed (7): data/city.json, data/poi.json, tasks/R14-01-city-chunk-contents.md, world/src/city/mod.rs, world/src/city/place.rs, world/src/city/place_tests.rs, world/src/generator.rs
- tests passed: 16
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
