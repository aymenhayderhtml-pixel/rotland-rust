# R40-03: Damage set pieces (fallen tower, leaning tower, burnt block) placement

Owns: world/src/metro/damage.rs (new: `DamageConfig`, `DamageSites`, `Site`, `damage_sites`, `damage_props_in_chunk`), world/src/metro/damage_tests.rs (new; split a sibling file if it passes ~300 lines), world/src/metro/state.rs (ONLY: the new `damage: DamageSites` field on `MetroWorld`, and the skip of replaced towers in `in_tower_footprint`), data/metro.json (ONLY the new top-level key `damage`). Shared files belong to the lead, report the exact lines: `world/src/metro/mod.rs`, `world/src/generator_metro.rs` (the `MetroWorld { .. }` literal at line ~44 needs `damage: damage_sites(&layout, &towers, &cfg)`). Do NOT edit `generator_contents.rs` or anything in `rotland/` (R40-08 wires it).

Read:
- tasks/R40-common.md ("Damage set pieces": the model ids, the rules, the `Site` struct, `tower_is_replaced`).
- tasks/archive/R39-01-tower-plan.md (task format).
- world/src/metro/towers.rs: `TowerSpot` (pos, half, yaw, height, floors, kind, landmark, block, style, size), `plan_block` (how a block gets its lots), `pick_landmarks`, `front_yaw`.
- world/src/metro/layout.rs: `MetroBlock` (ix/iz via the grid, rect, district, reserved), `MetroStreet` (a, b, width, avenue), `street_at`, `block_at`, `district_at`, `MetroLayout::streets`, `Reserved`.
- world/src/metro/jam.rs: `pick_avenues` and `AVENUE_COUNT` (private: copy the rule for "entrance avenue" into damage.rs; the entrance is `Highway.points.last()` from world/src/metro/highway.rs). Do not edit jam.rs.
- world/src/metro/state.rs: `MetroWorld`, `in_tower_footprint`, `towers_by_block`.
- world/src/metro/tower_parts.rs: `tower_parts`, `PartKind::Base`, `Floor` (the stack tops).
- world/src/metro/jam_chunks.rs and world/src/metro/jam_tests.rs (`union_over_all_chunks_is_the_whole_plan_once`): the per-chunk pattern and the test to copy.
- world/src/metro/place.rs: `MetroSite::to_world`, `to_local`, `contains_square`.
- world/src/metro/config.rs: `MetroConfig.reserved` and `districts`.
- world/src/scatter.rs: `PropInstance::new`.

## What to build
- `DamageConfig` (own serde struct in damage.rs, `#[serde(default)]`), from the top-level `damage` object: `fallenLengthM: [60, 80]`, `fallenWidthM: 14`, `fallenDistrict: "midtown"`, `leaningDistrict: "core"`, `leaningMinSize: 24`, `burntDistrict: "midtown"`, `burntMinTowers: 2`, `burntCarCount: [3, 5]`, `ashPatchesPerTower: 1`, `rubbleFieldsPerSite: 3`. Load and cache like `load_styles` / `default_styles` in world/src/metro/styles.rs.
- `DamageSites { fallen: Site, leaning: Site, burnt_block: (i32, i32), replaced_towers: Vec<usize> }` with `pub fn tower_is_replaced(&self, tower_index: usize) -> bool` (a method, because it needs the stored list; the brief's free fn name is kept). `Site { block: (i32, i32), pos: Vec2, yaw: f32 }`.
- `pub fn damage_sites(layout: &MetroLayout, towers: &[TowerSpot], cfg: &DamageConfig) -> DamageSites`. (Deviation from the brief: it takes layout + towers, not `&MetroWorld`, because `MetroWorld` stores the result, so it cannot be a parameter.) Deterministic per seed (site key). Rules:
  - **Fallen:** pick a block in `fallenDistrict` with a tower, not reserved, next to a plain (not avenue, not entrance) street. The section lies ALONG that street, centred on it, `fallenLengthM` long and `fallenWidthM` wide (street 12 m plus a bit on each sidewalk): it blocks the street end to end. Its own block's first tower is removed (listed in `replaced_towers`). No other tower footprint may overlap the section.
  - **Leaning:** the leaning piece is ONE baked model (an 18-floor Deco tower, about 80 m, leaning 8 degrees; lead decision 2026-11-26). Pick a block in `leaningDistrict` (core) with a tower of size `>= leaningMinSize` (24; prefer 32), whose 4-neighbour block also has a tower. That tower is replaced (listed). `Site.yaw` points the lean toward the neighbour tower (the lean axis).
  - Fallen and leaning blocks differ; neither is on a reserved block or the entrance avenue.
  - **Burnt block:** a block in `burntDistrict` with at least `burntMinTowers` towers, not the fallen or leaning block. `burnt_block` is its (ix, iz).
- `DamageSites::tower_is_burnt(&self, block: (i32, i32)) -> bool` (true for the burnt block). R40-08 uses it to pass `DamageMode::All` (R40-07's `tower_parts_with_damage`) for those towers; this task does not touch tower_parts.rs.
- `MetroWorld` gets `pub damage: DamageSites`. `in_tower_footprint` ignores towers in `replaced_towers` (they are gone), so people and props do not get blocked by a removed tower.
- `pub fn damage_props_in_chunk(metro: &MetroWorld, chunk: (i32, i32), height: &dyn Fn(f32, f32) -> f32) -> Vec<PropInstance>`. Per site, keyed to the chunk of its centre (the same floor rule as jam_chunks.rs):
  - `tw_fallen_section` at the fallen centre, yaw = street axis (its long axis is along the street).
  - `rubble_field_l` at both ends of the section, `rubble_field_m` x `rubbleFieldsPerSite` along it.
  - `tw_leaning_tower` at the leaning tower's pos, yaw = `Site.yaw`.
  - In the burnt block: `burnt_floor_overlay` at each tower base (yaw = tower yaw), `car_burnt_pile` x `burntCarCount` on the street edges (the cars stand off the asphalt centre), `ash_patch` on the block ground.
- Chunk-keyed PropInstance `scale` is 1.0 everywhere (the models are built at size).

## Tests (damage_tests.rs)
- Same site gives the same sites; a different seed gives a different block somewhere.
- Nothing on a reserved block or the entrance avenue (sites, sections, car piles).
- `tower_is_replaced` is true only for the fallen block's tower and the leaning tower (the other towers are not skipped).
- The fallen section covers its street from end to end and no remaining tower footprint overlaps it.
- The leaning direction points at the neighbour tower (within 30 degrees).
- Every tower in the burnt block is in the burnt set and the car count is in `burntCarCount`.
- `in_tower_footprint` is false for a replaced tower's old footprint (state.rs change).
- `damage_props_in_chunk`: over all chunks, each prop appears exactly once; heights come from `height(x, z)`.
- Config: `damage` parses; `fallenLengthM` with min > max is rejected.
- `world/tests/golden.rs` passes UNEDITED.

## Done when
- damage.rs and tests exist, each under ~300 lines; numbers come from `damage` in data (`// data: metro.json damage` tags).
- Shared-file lines (mod.rs, generator_metro.rs `damage:` field) are in Worker notes.
- Models listed here exist only after R40-04 (Blender); this task only uses their ids.

## Worker notes
