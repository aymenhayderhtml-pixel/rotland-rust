# R38-02: New Halden tower spots

Owns: world/src/metro/towers.rs (new; tests in `#[cfg(test)] mod tests`); data/metro.json "towers" sub-object only (R38-01 creates the file; replace only its values). Shared `world/src/metro/mod.rs` and `world/src/lib.rs` lines belong to the lead: report them.
Read: tasks/R38-common.md ("Towers", "Data file"), world/src/metro/mod.rs, config.rs, place.rs, layout.rs (R38-01: `MetroSite`, `MetroBlock`, `MetroLayout`, `MetroDistrict`, `Reserved`, `site.to_local`/`to_world`), world/src/city/lots.rs (how a lot is packed inside a block: usable rect = block minus half street/avenue widths minus `lotGap`), world/src/random.rs (`DotnetRandom`), data/metro.json ("towers", "districts", "reserved"), world/tests/golden.rs.

- `TowerKind { Lowrise, Midrise, Highrise, Landmark }` and `TowerSpot { pos: Vec2, half: Vec2, yaw: f32, height: f32, floors: u32, kind: TowerKind, landmark: bool, block: (i32, i32) }`. `pos`/`yaw` are world-space (`layout.site.to_world`), `half` is the local half extent, `block` is `(ix, iz)`.
- `fn plan_towers(layout: &MetroLayout, cfg: &MetroConfig) -> Vec<TowerSpot>`. Derive a per-block RNG from `layout.site.key`, `layout.site.center.to_bits()` and `(ix, iz)` with FNV-1a like `derive_town_seed`; the site is the seed, no wall clock/global random. Same site → identical vector.
- For each present block: compute the usable rect by shrinking each edge by half the street/avenue width on that edge (same `ix % avenueEvery`/`(ix+1) % avenueEvery` rule as `city/lots.rs`) plus `cfg.towers.lot_gap`. Skip reserved blocks except `SafeHouseTower` (exactly one tower there). Park/Stadium/Hospital/Police/FireStation/ArmyLine/Subway get no towers.
- Pack 1..=`max_lots_per_block` lots per block: split the usable rect along its longer local axis into 1 or 2 cells; pick a square side in the district's `footprint` range, clamp it to the cell minus gap; place the lot with a small deterministic offset; `yaw = site.yaw + 0 or 90°` (alternate per lot) so boxes face the grid. Skip a lot that cannot fit; never overlap another tower or a street.
- Heights from `data/metro.json` per district: Edge `floors` 4-8 (Lowrise), Midtown 10-20 (Midrise), Core 20-30 (Highrise); `height = floors as f32 * cfg.towers.floor_height` (3.8 m). Landmarks: pick `landmark_count` (3-5) distinct Core blocks deterministically, one per block, `floors` in `landmark_floors` (38-42, ~145-160 m), `kind: Landmark`, `landmark: true`; never on a reserved block, never two in a block. All other Core towers stay below `landmark_floors[0]`.
- Data: fill only `towers` in `data/metro.json` with the values above; add `// data: metro.json towers.<key>` tags. Do not edit `config.rs` or any other sub-object. If a field is missing, report the exact `config.rs` line for the lead.
- Tests: same site → identical `Vec<TowerSpot>`; every non-reserved block has 1..=`max_lots_per_block` towers; reserved blocks have none except `SafeHouseTower` (exactly one); floors/heights per district and landmark range; 3-5 landmarks, all in Core, distinct blocks, ~150 m; no tower rect (rotated, world) overlaps a `layout.street_at` point or leaves its block; total count is dense (e.g. > 700) but each block ≤ max; golden untouched.

## Done when
- `plan_towers` is deterministic, dense, street-safe, district-correct, has 3-5 Core landmarks ~150 m, and leaves reserved blocks free except SafeHouseTower.
- `data/metro.json` `towers` is filled; no other file is edited; the `mod towers; pub use ...` lines are reported for the lead.
- Tests above pass; clippy clean; `world/tests/golden.rs` unedited.

## Worker notes
