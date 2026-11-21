# R16-08: big map screen shows Ashford, roads and labels

Read first: `tasks/R16-common.md`, `rotland/src/map_ui/` (all files: the M-key map screen), `game/src/map/` (`explore.rs`, `markers.rs`, `tiles.rs`, `buildings.rs`), `rotland/src/app/minimap_draw.rs` (`draw_streets_and_road`: the minimap already draws city streets; reuse its approach and colors from `data/minimap.json`). The city: `world_gen.city()` (`streets`, `road`, `origin`, `size`, `contains`, `district_at`).
Files: rotland/src/map_ui/*, game/src/map/*, game/tests/all/map_*.rs, game/tests/all/main.rs, data/minimap.json, tasks/R16-08-map-city.md
Test: map

On the big map (M): draw the city streets (avenues wider), the camp road, blocked street marks, and the label "Ashford" at the city center, plus small district labels ("Core", "Industrial") when zoomed in. Only explored parts show (use the existing explored-cell memory: a street segment is drawn where its cells are explored), so the city appears as the player discovers it. Keep it fast when panning/zooming (skip segments outside the view; at most a few hundred lines). The camp road can show fully once the player has been on any part of it. Pure helpers (world -> map transform reuse, "is segment explored") get tests in `game/tests/all/map_city_tests.rs` (add to `main.rs`, names start `map_city_`) if they live in `game/src/map/`.

## Worker notes

