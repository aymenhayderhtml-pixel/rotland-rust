# R33-05: rivers drawn and swimmable, colliders, town banner, dev teleports

Owns: rotland/src/water_view.rs, rotland/src/river_view.rs (new), rotland/src/region_banner.rs, rotland/src/app/test_flags.rs, rotland/src/streaming.rs (only bridge/boardwalk colliders, if they need special handling), tools/README_TOOLS.md, tools/perf_scenes.sh
Files: rotland/src/water_view.rs, rotland/src/river_view.rs, rotland/src/region_banner.rs, rotland/src/app/test_flags.rs, rotland/src/streaming.rs, tools/README_TOOLS.md, tools/perf_scenes.sh
Read: tasks/R33-round.md, tasks/LEAD.md, docs/WORLD.md, rotland/src/water_view.rs, rotland/src/region_banner.rs, rotland/src/streaming.rs, rotland/src/app/test_flags.rs, tools/README_TOOLS.md, tools/perf_scenes.sh, engine/src/render/water.rs, rotland/src/app/frame_water.rs, game_player/src/swimming.rs
API: rotland/src/app/state.rs, world/src/generator.rs
Data: data/swimming.json
Overlap: wiring in files you do not own (`rotland/src/main.rs` `mod river_view;`, frame calls in `rotland/src/app/*`, the `water_at` hook in `game_player/src/swimming.rs`, and the `WaterRenderer` call) is NOT in your Files. Report those exact lines to the lead. `world/src/generator.rs` is R33-04's: read it, do not write it.
Rules: `tasks/LEAD.md` subagent rules. Use the "R33 shared names" in `tasks/R33-round.md` EXACTLY. Report any shared-file lines (app state, mod lines, frame calls) to the lead.

1. **River water** (`river_view.rs`):
   - Every ~1 s, or when the player has moved 50 m, rebuild the river triangle list with `generator.rivers().water_mesh_near(player_xz, 300.0)`. Cache it in an `Arc<Vec<Vec3>>`, the same way `water_view.rs` caches the lake mesh.
   - Draw it with the existing `WaterRenderer` the way the lake is drawn (read `water_view::draw`; add a second draw call or merge the lists).
   - Rebuild the mesh off the main thread if it takes > 2 ms. Measure with `std::time::Instant` in a debug log and say what you saw.
2. **Swimming and drinking in rivers:** find how the lake is treated as water for the player (grep `water_level`, `in_water`, `swim`) and make `generator.rivers().water_at(x, z)` count the same way. Report the exact lines if the code lives in files you don't own.
3. **Bridges and boardwalks you can walk on:** check how prop colliders are made in `streaming.rs` (the compound collider). A bridge deck must be walkable (the box collider from the model bounds is fine if the model is a flat deck plus railings; otherwise add a thin deck box). Test by screenshot: the player stands on a bridge (`--bridge=0`).
4. **Town banner:** in `region_banner.rs` (R31), entering a town radius shows `Entering: Dusthollow` (with the Danger line as today). Leaving shows nothing.
5. **Dev flags** (`test_flags.rs`), each teleporting to a ground point 25 m away from the target, facing it:
   - `--town=<key>`: the town centre.
   - `--trailhead=<n>`: trailhead n, facing outward along the road.
   - `--bridge=<n>`: the n-th crossing, standing on the deck.
   - `--river=<n>`: a bank point of river n, facing the water.
   Add each to `tools/README_TOOLS.md`, and add `town_desert` / `town_snow` scenes to `tools/perf_scenes.sh` under `--set all` (`--town=dusthollow --autowalk`, `--town=frostmere --autowalk`).

## Worker notes
