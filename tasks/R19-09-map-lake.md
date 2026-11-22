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
