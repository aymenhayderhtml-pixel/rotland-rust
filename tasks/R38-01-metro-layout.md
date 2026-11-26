# R38-01: New Halden site, block grid, streets and flat ground

Owns: world/src/metro/mod.rs, world/src/metro/config.rs, world/src/metro/place.rs, world/src/metro/layout.rs (all new; tests live in `#[cfg(test)] mod tests` inside each file); data/metro.json (new: every top-level key, with placeholder values for `towers`, `highway`, `jam`, `skyline`). R38-02..06 replace only their own sub-object's values. Shared files (`world/src/lib.rs`, `world/src/generator.rs`, `world/src/generator_world.rs`, `world/src/generator_contents.rs`, `rotland/*`, `Cargo.toml`) belong to the lead: report the exact lines.
Read: tasks/R38-common.md ("Placement", "Layout", "Data file"), world/src/city/config.rs (`CityConfigData`, `District`, `load_from_file`, `from_json`, `validate`), world/src/city/layout.rs (`CityLayout::generate`, `CityBlock`, `contains`/`district_at`/`street_at`), world/src/city/streets.rs (`StreetSegment`, `generate_streets`, `contains_point`, `closest_point`), world/src/city/ground.rs (`CityPlane::fit`, `CityGround::flatten_city`), world/src/city/lots.rs (usable block rect = half street widths + lot gap), world/src/towns/place.rs (`LayoutFrame`, `place_towns_in_frame`, `sector_center_dir`, `rotate_xz_ccw_from_above`, `derive_town_seed`), world/src/biome/layout.rs (`WorldLayoutData`, `SectorConfig`, `WorldLayout::reference_dir`/`direction_at_deg`), world/src/generator_world.rs (`attach_city_with_ground`, `attach_outer_world`, `RoadHeightField::build`/`sample`), world/src/generator.rs (`WorldGenerator` fields, `sample`, `apply_outer_height`, `raw_height`), world/src/generator_contents.rs (`keep_scattered_prop`), world/src/random.rs (`DotnetRandom`), data/world_layout.json, data/towns.json, data/rivers.json, data/city.json, data/city_ground.json, world/tests/golden.rs.

- `world/src/metro/` is pure logic (no GPU/winit); keep each file under ~300 lines. Tests live in `#[cfg(test)] mod tests` inside each file, so R38-02/03 can add their own tests in their own files.
- `mod.rs` declares only this task's modules (`config`, `place`, `layout`) and re-exports their public types. Do not declare `towers`/`highway` yet: R38-02/03 report those `mod`/`pub use` lines for the lead so this task's gate passes alone.

## Config (`config.rs`)
- `MetroConfig` parses `data/metro.json` with `#[serde(rename_all = "camelCase")]`, `load_from_file`, `from_json`, `validate`.
- Fields: `key`, `name`, `center_distance`, `angle_jitter_deg`, `size_blocks`, `block_size`, `street_width`, `avenue_width`, `avenue_every`, `corner_drop_chance`, `skirt_width`, `districts`, `reserved`, `towers`, `highway`, `jam`, `skyline`.
- Runtime-only `#[serde(skip, default = "default_reference_dir")] pub reference_dir: Vec2` (default `Vec2::X`) is set by the lead from `WorldLayout::reference_dir()` after `attach_city_with_ground`. It is the camp→Ashford direction the sector angles rotate from.
- Every number gets `// data: metro.json <key>`; structural constants get `// new: DECISIONS 2026-11-26`.
- Define these sub-structs now so later tasks never edit `config.rs`:
  - `DistrictsConfig { edge_rings, midtown_rings, core_blocks }`.
  - `ReservedArea { kind: Reserved, ix, iz, w, h }`.
  - `TowersConfig { edge/midtown/core: DistrictTowers { floors: [u32;2], footprint: [f32;2] }, floor_height, landmark_count: [usize;2], landmark_floors: [u32;2], lot_gap, max_lots_per_block }`.
  - `HighwayConfig { highway_width, median_width, lanes: u32, shoulder_width, height_smooth_m, flatten_edge_m, town_setback_m }`.
  - `JamConfig { jam_length, car_spacing_m: [f32;2], car_models: Vec<WeightedModel { id, weight }>, bus_count: [usize;2], army_truck_count: [usize;2], body_chance, luggage_chance, burnt_chance, inbound_chance, avenue_jam_chance, shoulder_body_chance, #[serde(default)] seat_offsets: HashMap<String, Vec<[f32;3]>> }`.
  - `SkylineConfig { skyline_distance, tris_per_tower: usize }`.
- If a later task needs a new field, it reports the exact line; it does not edit this file.

## Data (`data/metro.json`, new)
- Top level: `key` "new_halden", `name` "New Halden", `centerDistance` 6000, `angleJitterDeg` 4, `sizeBlocks` 32, `blockSize` 64, `streetWidth` 12, `avenueWidth` 20, `avenueEvery` 4, `cornerDropChance` 0.35, `skirtWidth` 40.
- `districts`: `{ "edgeRings": 5, "midtownRings": 6, "coreBlocks": 10 }` (10 + 2*6 + 2*5 = 32; validate this sum).
- `reserved`: one rectangle per kind, all 8 present, inside the grid and not overlapping: Park 3x3 at (6,6), Stadium 2x2 at (13,4), Hospital 2x2 at (18,13), Police (12,17), FireStation (20,12), ArmyLine 4x1 at (14,8), SafeHouseTower (15,15), Subway (16,16).
- `towers` placeholder: `{ "edge": { "floors": [4,8], "footprint": [14,22] }, "midtown": { "floors": [10,20], "footprint": [18,30] }, "core": { "floors": [20,30], "footprint": [24,40] }, "floorHeight": 3.8, "landmarkCount": [3,5], "landmarkFloors": [38,42], "lotGap": 3.0, "maxLotsPerBlock": 2 }`.
- `highway` placeholder: `{ "highwayWidth": 22, "medianWidth": 2, "lanes": 4, "shoulderWidth": 2, "heightSmoothM": 40, "flattenEdgeM": 8, "townSetbackM": 8 }`.
- `jam` placeholder: `{ "jamLength": 900, "carSpacingM": [6,9], "carModels": [sedan/hatch/suv/pickup/van with weights 4/3/2/2/2], "busCount": [1,2], "armyTruckCount": [1,2], "bodyChance": 0.25, "luggageChance": 0.2, "burntChance": 0.08, "inboundChance": 0.15, "avenueJamChance": 0.5, "shoulderBodyChance": 0.05, "seatOffsets": {} } (the three extra jam fields are for R38-04).`.
- `skyline` placeholder: `{ "skylineDistance": 6000, "trisPerTower": 60 }`.
- R38-02..06 own their sub-object and replace only its values (R38-05 adds only `jam.seatOffsets`); do not let any placeholder fail to parse.

## Placement (`place.rs`)
- `MetroSite` exactly as R38-common, with `to_local(p)`, `to_world(local)`, `signed_distance(p)`, `corners()`, `contains_square(p)`.
- `fn place_metro(seed: u64, camp: Vec2, cfg: &MetroConfig) -> MetroSite`. Load `data/world_layout.json` (`WorldLayoutData::load_from_file`) and read the swamp and snow `center_deg`; never type 90/180.
- `fn shortest_arc_midpoint_deg(a, b)` handles wrap (90/180 → 135; 350/10 → 0). Bearing = midpoint + `DotnetRandom::new(seed as i32).next_double()` jitter in `[-angleJitterDeg, +angleJitterDeg]`; direction = `rotate_xz_ccw_from_above(cfg.reference_dir, bearing)` (reuse `world/src/towns/place.rs`).
- Center = camp + direction * `centerDistance`. `half_size = sizeBlocks * blockSize * 0.5`.
- `yaw = to_camp.x.atan2(to_camp.y)` so local +Z faces the camp. `to_world` uses `rotate_xz_ccw_from_above(local, yaw.to_degrees())` and `to_local` the negative angle (Godot +Y rotation; the same rotation R38-07 must use).
- `ground_y = 0.0` placeholder; the lead sets it from `raw_height(center)` before `MetroLayout::generate`.
- Obstacle avoidance: build a pure `MetroPlacementFrame { reference_dir, swamp_center_deg, snow_center_deg, towns: Vec<(Vec2, f32)>, river_chords: Vec<(Vec2, Vec2, f32)>, core_outer_radius, spiral_step_m, spiral_max_m }` and `place_metro_in_frame(seed, camp, cfg, &frame)` for tests.
- `place_metro` loads `data/towns.json` and `data/rivers.json` into it: town ideal center = camp + sector dir * `distance_m`, radius = `radius + placement.spiral_max_m` (towns may drift); river chord = camp + sector dir(start_angle) * start_radius → camp + sector dir(target_angle) * target_radius, half width = `width_end_m + bank_m`.
- A candidate is clear if the rotated square (conservative radius `half_size * sqrt(2)`) is outside every town circle, every river chord, and `core_radius + blend_band`. Step outward along the same bearing by `placement.spiral_step_m` up to `placement.spiral_max_m`; if still blocked use the last candidate and `eprintln!` like `place_towns`.

## Layout (`layout.rs`)
- `MetroDistrict { Edge, Midtown, Core }`; `Reserved { Park, Stadium, Hospital, Police, FireStation, ArmyLine, SafeHouseTower, Subway }` (`#[serde(rename_all = "snake_case")]`, plus `ALL` for validation).
- `MetroBlock { ix, iz, rect: (Vec2, Vec2), district, reserved }`; `MetroStreet { a, b, width, avenue }` with `contains_point`/`closest_point` copied from `StreetSegment`; `MetroLayout { site, blocks, streets }`.
- `rect`, `a`, `b` are site-local; `MetroLayout` methods transform world→local with `site.to_local`.
- `MetroLayout::generate(site, cfg)`: derive a local RNG from `site.key` + `site.center.to_bits()` (FNV-1a like `derive_town_seed`); tile 32x32 blocks of 64 m; district by ring (core square, then midtown rings, then edge).
- Ragged corners: in each corner L (corner block + its two edge neighbours) drop blocks with `cornerDropChance`, never a reserved block and never a whole side; force reserved rectangles present and set `reserved`.
- Streets on every boundary with at least one present block; `avenueEvery` lines are avenues (20 m), others 12 m.
- Queries: `contains(p)` = present block or street surface; `district_at(p)`; `street_at(p)`; `block_at(ix, iz)`; `block_at_point(p)` (for parks); `near(p)` = `site.signed_distance(p) <= skirtWidth` (fast gate for scatter).
- `height_at(p)` = `Some(site.ground_y)` inside the square, `None` outside. `flatten(raw, p)` = `ground_y` inside, `lerp(ground_y, raw, smooth_step(0, skirtWidth, signed_distance))` in the skirt, `raw` outside.
- Keep queries simple like `CityLayout::street_at`; the lead gates city checks with `near` so props far away do not pay for 2k street tests.

## Wiring for the lead (report exact lines, do not edit)
- `world/src/lib.rs`: add `pub mod metro;` and `pub use metro::{MetroBlock, MetroConfig, MetroDistrict, MetroLayout, MetroSite, MetroStreet, Reserved, place_metro};`.
- `WorldGenerator`: add `metro: Option<MetroLayout>` (initialize `None` in `with_all_configs`), `pub fn metro()`, and `pub fn attach_metro(&mut self, cfg: &MetroConfig)` called after `attach_outer_world`.
- In `attach_metro`: set `site.ground_y = self.raw_height(site.center.x, site.center.y)` **before** `MetroLayout::generate`, then store the layout.
- In `sample`/`apply_outer_height`, after the shape height: `if let Some(m) = &self.metro { h = m.flatten(h, x, z); }` before roads/rivers.
- In `keep_scattered_prop`, when `metro.near(p)`, keep only park ground cover/trees off streets (copy the Ashford park rule); grass outside the skirt stays green.
- Do not change `raw_height`, `with_configs`, or `terrain_shape.rs`, so `world/tests/golden.rs` passes unedited.

## Tests (in-module, no new test files)
- Config loads every top-level key and all 8 reserved kinds.
- Same seed/site → identical site/blocks/streets; site is 5.9–6.7 km from camp and within 10° of the swamp/snow midpoint.
- Synthetic town/river obstacles force a step outward.
- Blocks tile the square, districts match the ring counts, reserved blocks are present/non-overlapping.
- Streets stay inside the square; `street_at` finds a street point; `contains` is true at block centers/street midpoints and false far outside.
- `height_at` is flat inside; `flatten` blends monotonically to raw at `half+skirt`; yaw round-trip `to_world(to_local(p)) ≈ p`.
- Golden untouched.

## Done when
- `data/metro.json` exists with every top-level key and parses; R38-02..06 can fill their sub-objects without editing this task's files.
- `place_metro` is deterministic, about 6 km out on the swamp/snow seam, clear of towns/rivers/core, and its yaw faces the camp.
- The 32x32 grid, 12/20 m streets, ragged corners, Edge/Midtown/Core, all 8 reserved blocks, flat `height_at` and `flatten` skirt are implemented and tested.
- `world/tests/golden.rs` is unedited and still passes; clippy is clean; the exact shared wiring lines are in Worker notes.

## Worker notes
