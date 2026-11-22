# R19-09: big map and minimap show the lake, its docks and the fishing village

Read first: `tasks/R19-common.md`, `rotland/src/map_ui/*` (how Ashford is drawn: `draw_city.rs`, `MapStreetsConfig` in `state.rs`), `game/src/map/city.rs` (pure map data for the city), `game/src/map/explore.rs` (discovered areas), `world/src/lake/layout.rs` (`LakeLayout`: `shore`, `docks`, `village_lots`, `fishing_spots`, `water_bounds`), `rotland/src/app/minimap*.rs`.
Files: game/src/map/lake.rs, game/src/map/mod.rs, game/tests/all/map_lake_tests.rs, game/tests/all/main.rs, rotland/src/map_ui/draw_lake.rs, rotland/src/map_ui/draw.rs, rotland/src/map_ui/state.rs, rotland/src/map_ui/mod.rs, rotland/src/map_ui/tests.rs, rotland/src/app/minimap.rs, rotland/src/app/minimap_cache.rs, rotland/src/app/minimap_draw.rs, data/map.json, tasks/R19-09-map-lake.md
Test: map

1. `game/src/map/lake.rs`: pure map data built from a `LakeLayout`: the shore as a simplified polygon (at most ~64 points), dock lines, village lot rectangles, fishing spot points, the label position. Tests: polygon closed and simplified, points inside the lake bounds.
2. Big map: draw the lake as a filled water-blue polygon with a darker shore outline, docks as short brown lines, village lots as small gray squares, a "Lake" label (name from data), a fish icon (or small circle) at fishing spots. Only after the player has discovered any part of it (same rule as the city). Colors and sizes in `data/map.json`.
3. Minimap: water cells show blue (match the big map color) where `depth_at > 0`.
4. Test flag: use the existing big map test flag (`--hub=map` or similar) with `--at=528,-1478` to screenshot; write the exact command in Worker notes.
5. If the lake layout isn't reachable from the map state, add an `Option<LakeMapData>` field the map receives, and write in Wiring notes where Claude must pass `world_gen.lake()` (follow how the city map data is passed).

## Worker notes
- Files changed:
  - `data/map.json`: Visual configuration for lake styling (water, shore outline, docks, village lots, fishing spots, label, and max shore points).
  - `game/src/map/lake.rs`: Pure map data (`LakeMapData`, `LakeMapDock`, `LakeMapLot`, `LakeMapFishingSpot`), `simplify_shore` polygon sub-sampler (<= 64 points), and `is_lake_discovered` exploration check.
  - `game/src/map/mod.rs`: Exported lake map structs, helpers, and constants.
  - `game/tests/all/map_lake_tests.rs`: Tests verifying closed simplified shore polygon (<= 64 points), all points within lake bounds, shore simplification edge cases, and discovery rules.
  - `game/tests/all/main.rs`: Added `mod map_lake_tests;`.
  - `rotland/src/map_ui/draw_lake.rs`: Big map drawing implementation for filled water-blue polygon, darker shore outline, docks, village lots, fishing spots, and "Lake" label; `MapLakeConfig` loader.
  - `rotland/src/map_ui/draw.rs`: Wired `draw_lake` into `draw_map`.
  - `rotland/src/map_ui/state.rs`: Added `lake_cfg: MapLakeConfig` to `MapUiState`.
  - `rotland/src/map_ui/mod.rs`: Added `mod draw_lake;`.
  - `rotland/src/map_ui/tests.rs`: Added `map_ui_lake_config_loads` test.
  - `rotland/src/app/minimap_cache.rs`: Updated `ensure_terrain_texture` to shade explored water cells water-blue matching big map color where `depth_at > 0`; added lake styling to `MinimapStreetsConfig`.
  - `rotland/src/app/minimap_draw.rs`: Added `draw_lake_features` (docks, village lots, fishing spots on minimap HUD) and transform unit test.
  - `rotland/src/app/minimap.rs`: Passed `lake` from world generator into `draw_minimap_overlay` and invoked `draw_lake_features`.
- How to test:
  - Verification gate: `tools/gate.sh --task tasks/R19-09-map-lake.md`.
  - Big map screenshot command:
    `flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch_saves --at=528,-1478 --hub=map --shot=/tmp/x/lake_map.png`
  - In-game: Press 'M' to open map. Lake appears once any part (shore, dock, lot, fishing spot, center) is discovered. On the minimap HUD, water cells render water-blue where `depth_at > 0`, and docks/lots/fishing spots appear in the overlay.
- Wiring notes for Claude:
  - `MapView` already receives `generator: world.world_gen.as_deref()`, which provides `.lake()`. No external wiring in other crates or files is required.
- Known issues:
  - None.

## Gate report
- date: 2026-11-22
- base: main
- files changed (14): data/map.json, game/src/map/lake.rs, game/src/map/mod.rs, game/tests/all/main.rs, game/tests/all/map_lake_tests.rs, rotland/src/app/minimap_cache.rs, rotland/src/app/minimap_draw.rs, rotland/src/app/minimap.rs, rotland/src/map_ui/draw_lake.rs, rotland/src/map_ui/draw.rs, rotland/src/map_ui/mod.rs, rotland/src/map_ui/state.rs, rotland/src/map_ui/tests.rs, tasks/R19-09-map-lake.md
- tests passed: 36
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
