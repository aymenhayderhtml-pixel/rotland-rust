# R40-01: Overgrowth placement (weeds, street trees, lot scrub, vines, roof bushes)

Owns: world/src/metro/overgrowth.rs (new), world/src/metro/overgrowth_tests.rs (new; split into `overgrowth_roofs.rs` if a file passes ~300 lines), data/metro.json (ONLY the new top-level key `overgrowth`). Shared files belong to the lead, report the exact lines: `world/src/metro/mod.rs` (`mod overgrowth;`, the `pub use` line), `world/src/lib.rs`. Do NOT edit `generator_contents.rs` (R40-08 wires the call).

Read:
- tasks/R40-common.md ("R40 shared names": the `overgrowth_props_in_chunk` signature, the four placement rules, the model ids).
- tasks/archive/R39-01-tower-plan.md (task format; copy the style of its "Data" and "Tests" sections).
- world/src/metro/jam_chunks.rs: `jam_props_in_chunk`, `in_chunk`, `CHUNK_SIZE_M`, `PropInstance::new` (the per-chunk pattern to copy). `PropInstance` is in world/src/scatter.rs.
- world/src/metro/jam.rs: `pick_avenues` and `AVENUE_COUNT`. The entrance avenues are the ones nearest `Highway.points.last()` (world/src/metro/highway.rs). `pick_avenues` is private: copy its rule into overgrowth.rs, do not edit jam.rs.
- world/src/metro/layout.rs: `MetroBlock` (rect, district, reserved), `MetroStreet` (a, b, width, avenue), `MetroLayout::block_at_point`, `street_at`, `district_at`, `block_at`. Layout points are WORLD XZ; `MetroSite::to_local` / `to_world` in world/src/metro/place.rs.
- world/src/metro/curbs.rs: `build_curbs_in_chunk` and how it finds street edges and the curb height (`look.curbHeight`, `look.sidewalk_width` in world/src/metro/paint.rs).
- world/src/metro/state.rs: `MetroWorld` fields, `in_tower_footprint` (it rotates the offset by `-yaw`; use the same rotation for vine faces), `towers_by_block`.
- world/src/metro/towers.rs: `TowerSpot` (pos, half, yaw, height, floors, block, style, size); `front_yaw` (private: read it to learn which local axis is the lobby front).
- world/src/metro/tower_parts.rs: `tower_parts`, `PartKind::Floor`, `TowerPart.pos`. Floor tops and the roof height come from here, not from a guess.
- world/src/metro/towers_tests.rs: `front_faces_nearest_street` (the front axis convention, use the same one).
- world/src/random.rs: `DotnetRandom::new`, `next_double`, `next_range`. `mix_seed` in towers.rs is private: copy its FNV rule into overgrowth.rs (same key/centre/block bytes).
- world/src/metro/config.rs and `from_json` in world/src/metro/styles.rs (for the own-file config loader pattern: `load_overgrowth`, cached `default_overgrowth()` with OnceLock, no edit to config.rs).
- data/metro.json: `blockSize` (64, same as the chunk), `streetWidth` (12), `avenueWidth` (20), `districts`.

## What to build
- `OvergrowthConfig` (own serde struct in overgrowth.rs, `#[serde(default)]`), loaded from the top-level `overgrowth` object. Per district (`edge`, `midtown`, `core`) it gives counts and chances per 64 m block: `crackWeedsPerBlock`, `streetTreeChance`, `mossPerBlock`, `lotShrubsPerBlock`, `lotTreesPerBlock`, `lotGrassPerBlock`, `vineTowerChance`, `roofBushChance`. Plus `vineSpacingM`, `crackWeedLengthM` (3), `streetTreeSpacingM` (32). Every number is in data.
- `pub fn overgrowth_props_in_chunk(metro: &MetroWorld, chunk: (i32, i32), height: &dyn Fn(f32, f32) -> f32) -> Vec<PropInstance>`. Y = `height(x, z)` plus the small offset of the model. A chunk is 64 m, so the block of the chunk is the block at the chunk centre (blockSize = chunk size; a test proves it).
- Seeds: one RNG per block from the FNV rule (site key + centre + block ix/iz + the salt `"overgrowth"`). Output for a chunk never depends on load order.
- Street level (streets of the block, not the entrance avenues, not reserved blocks): `weeds_crack` lines 0.3 m off the curb on the asphalt side; `tree_asphalt` on the asphalt edge lane (1.5-3 m from the kerb), at most one per `streetTreeSpacingM` and only with `streetTreeChance`; `moss_patch` on the curb base.
- Lots (block rect inset by half a street width, minus every tower footprint plus 1.5 m clearance; test with `in_tower_footprint`-style rotation): `shrub_wild`, `sapling_city` (young trees), `grass_tall_patch`, counts from the district.
- Walls: with `vineTowerChance`, a tower gets `vine_wall_s` / `_m` / `_l` (4 / 8 / 14 m, picked by the floors up to the tower's height in floor steps) on one or two faces, never on the lobby front face. The model's back sits 2-5 cm off the facade (the model is 0.3 m deep, so its centre is 0.15 + 0.02..0.05 m from the face). Base at ground height.
- Roofs: with `roofBushChance`, a `roof_bush` or `roof_tree` on the roof: the roof height is the top of the last `PartKind::Floor` part from `tower_parts` (plus its height), not a guessed number. Keep it inside the roof rect (1.5 m in from the edge).
- Reserved blocks: none (Park keeps the existing scatter).
- No prop on a street that is an entrance avenue.

## Tests (overgrowth_tests.rs)
- Counts per district: summed over the whole layout, Edge > Midtown > Core for the weeds and the vines (same site, same seed).
- Nothing on the entrance avenue or on any reserved block.
- Vines are flat on a facade: back 2-5 cm off the face, base at ground height, never on the front (lobby) face.
- Roof props sit at the last floor top from `tower_parts` (within 0.05 m) and inside the roof rect.
- Lot props stay out of tower footprints and off street surfaces.
- Deterministic: same site gives the same list; the union over all chunks has no duplicates and no misses (like `union_over_all_chunks_is_the_whole_plan_once` in jam_chunks.rs).
- `load_overgrowth` parses the data; a negative chance is rejected.
- `world/tests/golden.rs` passes UNEDITED.

## Done when
- overgrowth.rs (and tests) exist, each file under ~300 lines; no `env!` path, numbers only from data (`// data: metro.json overgrowth` tags).
- The lead's shared-file lines (mod.rs `mod` and `pub use`) are in Worker notes.
- Density is visibly higher in Edge than in Core on the test numbers.

## Worker notes
