# R14-04: city streets and the camp road on the minimap

Read first: `tasks/R13-common.md` ("Rust/data lanes" part), `rotland/src/app/minimap.rs`, `rotland/src/app/minimap_cache.rs`, `rotland/src/app/minimap_draw.rs`, `data/minimap.json`. The city: `world::CityLayout` from `world_gen.city()`: `streets` (`a`, `b`, `width`, `blocked`), `road` (polyline of world x/z points), `contains`, `origin`, `size`.
Files: rotland/src/app/minimap.rs, rotland/src/app/minimap_cache.rs, rotland/src/app/minimap_draw.rs, data/minimap.json, tasks/R14-04-minimap-streets.md
Test: minimap

Draw the city streets and the road from camp on the minimap, under the player arrow and zombie dots: dark gray lines, width scaled from the street width (at least 2 px), avenues a bit wider; blocked segments drawn with a small red cross mark in the middle. Colors and min line width go in `data/minimap.json` (`street_color`, `road_color`, `blocked_color`, `street_min_px`, matching that file's snake_case style). Only draw segments that are within the minimap radius (cheap: skip by distance before drawing). If the minimap uses a cached ground image, draw the streets in the same cached layer if that's simple; otherwise draw them as egui line shapes each frame (at most a few hundred segments, skip anything outside the radius). Add a unit test for the world-to-minimap transform of a street end point if there is a pure function for it (reuse it, don't invent a new one).

## Worker notes

