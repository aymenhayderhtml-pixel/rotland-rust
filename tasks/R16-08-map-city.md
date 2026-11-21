# R16-08: big map screen shows Ashford, roads and labels

Read first: `tasks/R16-common.md`, `rotland/src/map_ui/` (all files: the M-key map screen), `game/src/map/` (`explore.rs`, `markers.rs`, `tiles.rs`, `buildings.rs`), `rotland/src/app/minimap_draw.rs` (`draw_streets_and_road`: the minimap already draws city streets; reuse its approach and colors from `data/minimap.json`). The city: `world_gen.city()` (`streets`, `road`, `origin`, `size`, `contains`, `district_at`).
Files: rotland/src/map_ui/*, game/src/map/*, game/tests/all/map_*.rs, game/tests/all/main.rs, data/minimap.json, tasks/R16-08-map-city.md
Test: map

On the big map (M): draw the city streets (avenues wider), the camp road, blocked street marks, and the label "Ashford" at the city center, plus small district labels ("Core", "Industrial") when zoomed in. Only explored parts show (use the existing explored-cell memory: a street segment is drawn where its cells are explored), so the city appears as the player discovers it. Keep it fast when panning/zooming (skip segments outside the view; at most a few hundred lines). The camp road can show fully once the player has been on any part of it. Pure helpers (world -> map transform reuse, "is segment explored") get tests in `game/tests/all/map_city_tests.rs` (add to `main.rs`, names start `map_city_`) if they live in `game/src/map/`.

## Worker notes
- Files changed:
  - `game/src/map/city.rs`: Pure helpers for city map: `is_segment_explored`, `is_segment_any_explored`, `explored_segment_spans`, `is_road_discovered`, `is_city_center_revealed`, `is_district_revealed`, `city_center`, `district_center`, `map_visible_world_bounds`, `segment_in_bounds`, plus colors and constants tagged from `data/minimap.json`.
  - `game/src/map/mod.rs`: Exported city map helpers and constants.
  - `game/tests/all/map_city_tests.rs`: Unit/integration tests covering all pure helpers (prefixed `map_city_`).
  - `game/tests/all/main.rs`: Added `mod map_city_tests;`.
  - `rotland/src/map_ui/draw_city.rs`: Full drawing implementation for city streets, avenues, camp road, blocked street crosses, Ashford label, and zoomed-in Core/Industrial district labels.
  - `rotland/src/map_ui/draw.rs`: Wired `draw_city_and_road` into `draw_map`.
  - `rotland/src/map_ui/state.rs`: Added `streets_cfg: MapStreetsConfig` to `MapUiState`.
  - `rotland/src/map_ui/mod.rs`: Exported `draw_city_and_road` and `MapStreetsConfig`.
  - `rotland/src/map_ui/tests.rs`: Added `map_ui_streets_config_loads` test.
- How to test:
  - Run `tools/gate.sh --task tasks/R16-08-map-city.md`.
  - In-game: open map ('M'), pan/zoom around city. Streets reveal incrementally as cells are explored, avenues draw wider, blocked streets show red crosses, "Ashford" displays at the city center once explored, and "Core" & "Industrial" labels display when zoomed in (scale >= 1.0). Camp road reveals fully once player visits any part of it.
- Wiring notes for Claude:
  - `draw_city_and_road` is already called in `rotland/src/map_ui/draw.rs`. `MapView` already receives `generator: world.world_gen.as_deref()`.
- Known issues:
  - None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (10): game/src/map/city.rs, game/src/map/mod.rs, game/tests/all/main.rs, game/tests/all/map_city_tests.rs, rotland/src/map_ui/draw_city.rs, rotland/src/map_ui/draw.rs, rotland/src/map_ui/mod.rs, rotland/src/map_ui/state.rs, rotland/src/map_ui/tests.rs, tasks/R16-08-map-city.md
- tests passed: 23
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
