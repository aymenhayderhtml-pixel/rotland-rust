# R33-06: maps show roads, rivers and towns

Owns: game_camp/src/map/*, rotland_ui/src/map_ui/*, world/examples/worldmap.rs
Files: game_camp/src/map/tiles.rs, game_camp/src/map/markers.rs, rotland_ui/src/map_ui/draw_city.rs, rotland_ui/src/map_ui/draw_icons.rs, world/examples/worldmap.rs
Read: tasks/R33-round.md, tasks/LEAD.md, docs/WORLD.md, game_camp/src/map/tiles.rs, game_camp/src/map/markers.rs, rotland_ui/src/map_ui/draw_city.rs, rotland_ui/src/map_ui/draw_icons.rs, rotland_ui/src/map_ui/draw_lake.rs, world/examples/worldmap.rs
API: game_camp/src/map/mod.rs, world/src/lib.rs, rotland_ui/src/map_ui/mod.rs
Overlap: none with other R33 tasks. Any `rotland/src/app/*` wiring needed for the big-map town labels is not in your Files: report it to the lead.
Rules: `tasks/LEAD.md` subagent rules. Use the "R33 shared names" in `tasks/R33-round.md` EXACTLY. Read how the big map draws Ashford (`rotland_ui/src/map_ui/draw_city.rs`) and how the minimap tiles are coloured (`game_camp/src/map/tiles.rs`), and follow the same patterns.

1. **Minimap + big-map terrain tiles** (`tiles.rs`): river texels in the water colour the lake already uses on the map, highways dark grey, trails light brown. Use `distance_to_road` / `water_at` per texel, but only outside the core (cheap distance check first).
2. **Big map overlay:**
   - town names in the same style as the "Ashford" label, with a small square marker;
   - a signpost marker at each trailhead;
   - road names shown as a tooltip when hovering near a road, if the map already has hover; otherwise skip that.
3. **`world/examples/worldmap.rs`:** draw roads (highways 2 px dark grey, trails 1 px brown), rivers (blue, width by pixel scale, at least 1 px), towns (a black square + the name in 5×7 pixel letters; a tiny built-in bitmap font of only the needed capital letters is fine) and bridges (white dots).
4. **Tests:** a tile at a known road point has the road colour; a tile in the core is unchanged.

## Worker notes

- **Files changed:**
  - `game_camp/src/map/tiles.rs`: Added `HIGHWAY_MAP_COLOR` (`[72, 72, 76, 255]`), `TRAIL_MAP_COLOR` (`[166, 132, 92, 255]`), and cheap core-radius squared-distance guard in `shade_texel` so texels outside `core_radius` render river water (`LAKE_WATER_COLOR`), highways, and trails. Added unit tests `road_texel_outside_core_has_road_color` and `core_tile_is_unchanged`.
  - `rotland_ui/src/map_ui/draw_icons.rs`: Added `draw_town_icon` (8×8 square marker + Ashford-style 16 px label with 1 px drop shadow) and `draw_signpost_icon` (post + tan sign board).
  - `rotland_ui/src/map_ui/draw_city.rs`: Updated `draw_city_and_road` to draw outer towns (`world_gen.towns()`) and trailhead signposts (`world_gen.roads().trailheads()`) even when `world_gen.city()` is `None`. Skipped road hover tooltip because the map UI does not currently have hover tooltip handling.
  - `world/examples/worldmap.rs`: Added `draw_rivers` (blue, scaled by river width, min 1 px), `draw_roads` (2 px dark grey highways, 1 px brown trails), `draw_bridges` (2 px white dots at `rivers().crossings(roads)`), and `draw_towns` (5×5 black square + uppercase 5×7 bitmap font label).
- **Shared-file lines:** None needed (no changes to `rotland/src/app/*`, `*/src/lib.rs`, `Cargo.toml`, or `game/tests/all/main.rs`).
- **How to see it:** Run `cargo run --release -p world --example worldmap -- --seed=42 --size=12000 --px=1200 --out=/tmp/r33/worldmap.png` to inspect rivers, roads, bridges, and town labels; open the big map or minimap in-game outside the 2.2 km core to see river/road texels, town names, and trailhead signposts.
- **Unverified / notes for lead:** Per subagent rules, `cargo` and `tools/gate.sh` were not run; `WorldGenerator::towns()`, `roads()`, and `rivers()` are expected from R33-04.

