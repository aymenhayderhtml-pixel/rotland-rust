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
