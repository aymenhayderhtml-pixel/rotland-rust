# R33-03: town placement + lot layout (pure)

Owns: world/src/towns/* (new: mod.rs, place.rs, lots.rs), data/towns.json (new), world/tests/towns_tests.rs (new)
Files: world/src/towns/mod.rs, world/src/towns/place.rs, world/src/towns/lots.rs, data/towns.json, world/tests/towns_tests.rs
Read: tasks/R33-round.md, tasks/LEAD.md, docs/WORLD.md, world/src/lake/layout.rs, world/src/lake/types.rs
API: world/src/lib.rs, world/src/biome/mod.rs, world/src/lake/mod.rs
Data: data/world_layout.json, data/lake.json
Overlap: `world/src/lib.rs` belongs to R33-04: do not write it, report your `pub mod towns;` / `pub use` lines to the lead.
Rules: `tasks/LEAD.md` subagent rules. Use the "R33 shared names" in `tasks/R33-round.md` EXACTLY. Read `world/src/lake/layout.rs` + `world/src/lake/types.rs` (`VillageLot`, overlap checks): the lake village is the closest example of a small town. Copy its approach. Don't edit `world/src/lib.rs`: report your lines to the lead.

1. **`data/towns.json`**, one entry per town:
   - Keys: `key`, `name`, `biome`, `distance_m` (from origin, along that sector's centre angle: desert 3600, swamp 3400, snow 3800, mountains 4200), `radius` (110-150 m), `street_length` (180-240 m), `lot_spacing` (26-34 m), `buildings` (a weighted list of model names, from the shared names, plus existing ones that fit: `gas_station` in the desert, `cabin`/`shed` in snow/swamp, `watchtower` in the mountains), `loot_tables` per building, `_why`.
   - Dusthollow: adobe_house ×many, motel ×1, water_tower ×1, desert_store ×1, gas_station ×1.
   - Frostmere: log_cabin_snow ×many, research_station ×1, shed.
   - Blackwater: stilt_shack ×many, bait_shop ×1, a boardwalk along the street (as lots with `boardwalk_section`).
   - Ridgeback Station: ranger_station ×1, radio_tower ×1, cabin ×3-4, watchtower ×1.
2. **Placement:**
   - Start at the sector-centre point at `distance_m`.
   - If `flat_check(center, radius)` fails, search a small spiral (step 80 m, up to 600 m) for the first place that passes. Use the result even if none passes, and note it in a log line (no panic).
   - `street_dir` = facing back toward the camp (so the highway arrives along the street).
3. **Lots:**
   - Along both sides of a straight main street through the centre: lots every `lot_spacing`, facing the street (yaw), with the footprint from a small per-model table in the JSON (half extents).
   - Pick the building per lot from the weighted list (seeded RNG); "×1" buildings exactly once, nearest the centre.
   - No overlaps (test). Each lot gets its building's `loot_table`.
4. **Tests:**
   - 4 towns, each in its biome's sector, outside `core_radius + blend_band`;
   - no lot overlaps;
   - each unique building appears once;
   - deterministic for a seed;
   - a failing `flat_check` still places a town (fallback) without a panic.

## Worker notes

## Neon notes

* **Shared-file lines for the Lead (`world/src/lib.rs`, owned by R33-04):**
Add alphabetically:
```rust
pub mod towns;

```


* **Missing API signature (`world/src/biome/layout.rs`):**
`world/src/biome/layout.rs` was not included in the task pack's `API signatures`, so the adapter `layout_frame(layout: &WorldLayout)` at the bottom of `world/src/towns/place.rs` assumes `layout.reference_dir()`, `layout.core_radius()`, `layout.blend_band()`, and `layout.sector_center_deg(biome)`. Verify those 4 calls against the real fields/methods on `WorldLayout` (all other logic in `place.rs`, `lots.rs`, and `towns_tests.rs` goes through `LayoutFrame` and needs no changes). You may also add one integration test in `world/tests/towns_tests.rs` calling `world::towns::place_towns` directly with `WorldLayout`'s real constructor.
* **Logging crate:**
`world/Cargo.toml` was not included in the task pack, so the fallback log line in `world/src/towns/place.rs` uses `eprintln!`. Swap to `log::warn!` or `tracing::warn!` if `world` depends on either crate.
* **Boardwalk loot & indoors convention (for R33-04 & R33-09):**
`boardwalk_section` lots in Blackwater have `loot_table: ""` and are excluded from `TownSite::indoors(p)` so boardwalks do not count as enclosed indoor shelters or spawn indoor loot crates.
* **Per-town lot counts & footprints:**
All building footprints in `data/towns.json` are half-extents `[half_width_along_street, half_depth_perpendicular_to_street]`. `ridgeback_station` caps `cabin` at `max_count: 4` (giving 1 ranger station, 1 radio tower, 1 watchtower, and 4 cabins).
