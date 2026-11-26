# R16-05: lake & coast layout (pure, world crate)

Read first: `tasks/R16-common.md`, `docs/DECISIONS.md` rows dated 2026-11-21, `world/src/city/mod.rs` + `world/src/city/layout.rs` + `world/src/city/ground.rs` (copy this pattern: a pure layout attached to the generator), `world/src/generator.rs` (`attach_city`, `sample`, `emit_triangle`, `chunk_contents`).
Files: world/src/lake/*, world/src/lib.rs, data/lake.json, tasks/R16-05-lake-layout.md
Test: lake

The lake & coast region (owner: opposite the city, about 1.5 km from the camp). This task is the pure layout only; Claude attaches it to the generator on day 2 (so do NOT edit `generator.rs`).
- `LakeConfig` from `data/lake.json`: `distanceFromCamp` 1500, `radius` [260, 200] (ellipse half axes, m), `shoreNoise` amplitude/frequency (wobbly coastline), `beachWidth` 14, `shallowWidth` 12 (wading), `maxDepth` 8, `waterLevelAboveShore` 0.3, `village` lots (count 8, sizes, along one shore), `docks` 3 (piers 4 x 20 m out into the water), `angleFromCity` 180 (degrees; opposite).
- `LakeLayout::generate(seed, camp: Vec2, city_center: Option<Vec2>, cfg, raw_height: impl Fn(Vec2)->f32) -> LakeLayout`: center at `distanceFromCamp + radius` from the camp, in the direction opposite the city (or seeded when no city). Fields: `center`, `radius`, `water_level` (shore raw height + `waterLevelAboveShore`), `shore` polyline (sampled), `docks` (start, end, yaw), `village_lots` (center, size, yaw facing the water), `fishing_spots` (on docks and beaches).
- Queries (fast, called per vertex): `shore_distance(p) -> f32` (signed: negative in water), `depth_at(p) -> f32` (0 at the shore line, growing to `maxDepth` toward the middle, smooth), `zone_at(p) -> LakeZone { Land, Beach, Shallow, Deep }`, `ground_height(p, raw) -> f32` (carves the lake bed: terrain blends down to `water_level - depth` inside, and to a flat beach just above the water on the beach band; unchanged outside the region), `contains_region(p)` (bounding box + margin, for early-outs).
Tests (in `world/src/lake/tests.rs` as `#[cfg(test)] mod tests;`, names start `lake_`): deterministic; opposite side of the camp from a given city; distance from camp ~1500 at the nearest shore; depth 0 at the shore and max near the middle; zones in the right order walking from land to the middle; docks start on land and end over water; village lots on land, not in water, not overlapping; far points unchanged by `ground_height`.

## Worker notes
- Files changed:
  - `data/lake.json`: Configuration defining lake distance (1500m), ellipse radius ([260m, 200m]), shore noise amplitude/frequency, beach/shallow widths, max depth (8m), water level offset, village lots (8 count + dimensions), docks (3 count, 4x20m piers), and opposite city angle (180°).
  - `world/src/lake/config.rs`: `LakeConfigData` (`LakeConfig`), `ShoreNoiseConfig`, `VillageConfig`, `DocksConfig`, with serde deserialization, validation, and file loader.
  - `world/src/lake/layout.rs`: `LakeLayout`, `LakeZone`, `Dock`, `VillageLot`, `FishingSpot`, `FishingSpotKind`, `generate()`, and fast vertex queries (`shore_distance`, `depth_at`, `zone_at`, `ground_height`, `contains_region`).
  - `world/src/lake/mod.rs`: Module definitions and public re-exports.
  - `world/src/lake/tests.rs`: 11 unit tests covering determinism, camp/city opposite placement, ~1500m nearest shore distance, 0-to-max depth gradient, zone progression (Land -> Beach -> Shallow -> Deep), dock placement (land start, water end), village lots (on land, facing water, non-overlapping), fishing spots, and terrain carving.
  - `world/src/lib.rs`: Exposes `lake` module and re-exports lake types.

- How to test:
  - `tools/gate.sh --task tasks/R16-05-lake-layout.md`
  - Or `cargo test lake`

- Wiring notes for Claude:
  - Attach to `WorldGenerator`:
    ```rust
    pub fn attach_lake(&mut self, cfg: &LakeConfigData) {
        let camp = self.settlements.first().map_or(Vec2::ZERO, |s| Vec2::new(s.center.x, s.center.z));
        let city_center = self.city.as_ref().map(|c| c.origin + Vec2::splat(c.size * 0.5));
        let layout = LakeLayout::generate(self.seed, camp, city_center, cfg, |p| self.raw_height(p.x, p.y));
        self.lake = Some(layout);
    }
    ```
  - In `sample` / chunk terrain generation:
    ```rust
    let h = if let Some(lake) = &self.lake {
        lake.ground_height(Vec2::new(x, z), raw_h)
    } else {
        raw_h
    };
    ```
  - In player / water physics:
    `lake.zone_at(pos.xz())` gives `LakeZone::{Land, Beach, Shallow, Deep}`. Use `Shallow` for wading slowdown and `Deep` for swimming/stamina drain.
  - Docks and village lots:
    `lake.docks` (pier segments `start`, `end`, `yaw`), `lake.village_lots` (`center`, `size`, `yaw`), and `lake.fishing_spots` (`position`, `kind`) provide placements for models and interactables.

- Known issues:
  - None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (7): data/lake.json, tasks/R16-05-lake-layout.md, world/src/lake/config.rs, world/src/lake/layout.rs, world/src/lake/mod.rs, world/src/lake/tests.rs, world/src/lib.rs
- tests passed: 11
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
