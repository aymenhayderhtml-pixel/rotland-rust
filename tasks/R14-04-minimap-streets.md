# R14-04: city streets and the camp road on the minimap

Read first: `tasks/R13-common.md` ("Rust/data lanes" part), `rotland/src/app/minimap.rs`, `rotland/src/app/minimap_cache.rs`, `rotland/src/app/minimap_draw.rs`, `data/minimap.json`. The city: `world::CityLayout` from `world_gen.city()`: `streets` (`a`, `b`, `width`, `blocked`), `road` (polyline of world x/z points), `contains`, `origin`, `size`.
Files: rotland/src/app/minimap.rs, rotland/src/app/minimap_cache.rs, rotland/src/app/minimap_draw.rs, data/minimap.json, tasks/R14-04-minimap-streets.md
Test: minimap

Draw the city streets and the road from camp on the minimap, under the player arrow and zombie dots: dark gray lines, width scaled from the street width (at least 2 px), avenues a bit wider; blocked segments drawn with a small red cross mark in the middle. Colors and min line width go in `data/minimap.json` (`street_color`, `road_color`, `blocked_color`, `street_min_px`, matching that file's snake_case style). Only draw segments that are within the minimap radius (cheap: skip by distance before drawing). If the minimap uses a cached ground image, draw the streets in the same cached layer if that's simple; otherwise draw them as egui line shapes each frame (at most a few hundred segments, skip anything outside the radius). Add a unit test for the world-to-minimap transform of a street end point if there is a pure function for it (reuse it, don't invent a new one).

## Worker notes
- Files changed:
  - `data/minimap.json`: Added `street_color` (#45454a), `road_color` (#5a5348), `blocked_color` (#d9352b), `street_min_px` (2.0).
  - `rotland/src/app/minimap_cache.rs`: Added `MinimapStreetsConfig` to deserialize the new minimap styling options from `data/minimap.json` (supporting both hex strings and numeric arrays) with clean fallbacks; stored in `MinimapState`.
  - `rotland/src/app/minimap_draw.rs`: Implemented `draw_streets_and_road` which culls distant segments, clips lines to the minimap disk via `game::clip_segment_to_circle`, scales stroke width by street width (ensuring avenues are wider and minimum width is respected), draws red cross marks at the midpoint of blocked segments, and added unit test `minimap_street_endpoint_world_to_minimap_transform`.
  - `rotland/src/app/minimap.rs`: Passed `world.world_gen.city()` to `draw_minimap_overlay`, drawing streets and the camp road on top of terrain and camp outlines but under player arrow, zombie dots, and map markers.
- How to test:
  - Run `tools/gate.sh --task tasks/R14-04-minimap-streets.md` or `cargo test minimap`.
- Wiring notes for Claude:
  - City streets and road are drawn dynamically from `world.world_gen.as_ref().and_then(|g| g.city())` without requiring extra scratch allocations.
  - Early-out spatial checks skip street evaluation when the player is not near the city bounds.
- Known issues:
  - None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (5): data/minimap.json, rotland/src/app/minimap_cache.rs, rotland/src/app/minimap_draw.rs, rotland/src/app/minimap.rs, tasks/R14-04-minimap-streets.md
- tests passed: 11
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
