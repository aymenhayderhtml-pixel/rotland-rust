# R38-08: wire New Halden into the game (start, terrain, props, colliders, haze, map, banner, test flags)

Owns: rotland/src/metro_play.rs (new: `MetroState` built once at world start, holds `MetroSite`, `MetroLayout`, `Vec<TowerSpot>`, `Highway`, `HighwayHeightField`, `Vec<JamProp>`, and the per-frame helpers: `metro_name_at(p)`, the haze call, the skyline draw list), rotland/src/app/metro_flags.rs (new: `--metro=<spot>` placement helpers, pure pose math with tests), and the wiring lines listed below. Also: tools/perf_scenes.sh (the `metro` scene), tools/README_TOOLS.md (the new flag lines), and `data/metro.json` only if a value must change for wiring (report first).
Shared world lines (copy them EXACTLY from the Worker notes of R38-01, R38-02, R38-03 and R38-04 once those are merged, and from R38-06 for the terrain paint line): `world/src/lib.rs` (`pub mod metro;` and the `pub use` line), `world/src/metro/mod.rs` (`mod towers;`, `mod highway;`, `mod jam;`, `mod jam_chunks;`, and their `pub use` lines), `world/src/generator.rs` (the `metro` field, `metro()`, `attach_metro`, the flatten calls in `sample`/`apply_outer_height`), `world/src/generator_world.rs` (`attach_metro` call after `attach_outer_world` at line ~198, the highway crossings append, the `HighwayHeightField` build), `world/src/generator_contents.rs` (`keep_scattered_prop` park rule near the metro, and the `jam_props_in_chunk` call after the city props at line ~89), `world/src/terrain_paint.rs` (`metro` field in `TerrainPaintCtx` and the `metro_ground_color` call after `city_ground.paint`, from R38-06's Worker notes). Do not change `raw_height`, `with_configs`, `terrain_shape.rs` or any other world line: `world/tests/golden.rs` must pass unedited.
Read: tasks/R38-common.md ("Test flags", "Data file"), tasks/R38-01 .. R38-07 (Worker notes), rotland/src/app/test_flags.rs (`TestFlags` fields and `from_args` around lines 43-130; `apply_startup` line ~333; `apply_town_tp` line 506 and `town_tp_pose` line 453 for the pose math; `WORLD_TP_DISTANCE_M`), rotland/src/app/anim_shot.rs (the `--cam-yaw` override, to find how a camera height or pitch could be forced for `top`), rotland/src/app/start_world.rs and world/src/generator_world.rs (`attach_outer_world`, line ~198: the world start order), rotland/src/app/render_frame.rs (around lines 228-240: weather lighting, then `sky.apply_biome_fog`), rotland/src/app/frame_ui.rs (line ~144: `play.region_banner.update(`), rotland/src/app/minimap.rs (line ~161: `draw_streets_and_road(`), rotland/src/app/minimap_draw.rs, rotland_ui/src/map_ui/draw.rs (line ~140: `draw_city_and_road`, the draw order), rotland/src/streaming.rs (`LoadedChunk` line 33, the build with colliders around lines 250-332), save/src/file.rs (serde default pattern at lines ~47-55), tools/perf_scenes.sh (case lines 71-79 and scene args lines ~138-146), tools/README_TOOLS.md (test flag section near line 60).

## 1. World start (rotland)
- Build `MetroState` once where the world is started (`rotland/src/app/start_world.rs`), after the `WorldGenerator` is built: `place_metro(seed, camp, cfg)`, then `attach_metro`, `plan_towers`, `plan_highway` (with the nearest town), `HighwayHeightField::build`, `plan_jam`. Keep `MetroState` behind an `Option` so a world without `data/metro.json` still starts (report the error as a warning, never a panic).
- Terrain height: the lead's `attach_metro` sets `ground_y` from `raw_height` before `MetroLayout::generate`; check that Ashford's golden heights are unchanged (`cargo test` golden must pass).
- Chunk props: the jam props and the towers reach the game through `jam_props_in_chunk` (chunk_contents) and the near tower mesh/hulls (R38-06) built in the streaming chunk build. Add `towers: Option<GpuMesh>` to `LoadedChunk`, and call `tower_hulls` next to the prop colliders (`colliders.push`), following the same pattern as the static props.
- Haze: call `rotland_ui::metro_ui::haze` `blend_fog`/`skyline_fog` through `rotland/src/metro_haze.rs` (R38-07) right after `sky.apply_biome_fog` in `render_frame.rs`. Skyline: draw `SkylineBatch::draw_items` in the MAIN pass only (not shadows). Night: no new lights.
- Map: call `draw_metro_map(painter, state, explore, site, layout, rect, view_size)` (R38-07) at the end of `draw_city_and_road` in `rotland_ui/src/map_ui/draw.rs` (line ~140), so it draws over the Ashford streets.
- Minimap: in `rotland/src/app/minimap.rs` next to `draw_streets_and_road(` (line ~161) draw `minimap_streets(...)` (R38-07) with the existing street colour; do not edit minimap_draw.rs beyond adding a sibling fn if needed.
- Banner: `rotland/src/app/frame_ui.rs` line ~144: pass the fifth argument `|| metro_state.name_at(player_pos)` (returns `Some("New Halden")` when the player is inside `MetroSite::contains_square`, else None).

## 2. Test flag `--metro=<spot>` (rotland/src/app/test_flags.rs)
- Add `pub metro: Option<String>` to `TestFlags`, parsed with `args.value("--metro")` like `--town`. Spots: `edge`, `core`, `highway`, `town_view`, `top`. Unknown spot: `log::warn!` and no teleport (like `--town`).
- Poses (in `rotland/src/app/metro_flags.rs`, pure, tested): yaw uses the AGENTS rule, forward = (-sin yaw, -cos yaw), so `yaw = atan2(-dx, -dz)` for a direction `(dx, dz)`.
  - `edge`: 10 m inside the first avenue entrance (the avenue nearest the highway end), looking toward the site centre.
  - `core`: the site centre, looking along the nearest avenue.
  - `highway`: the jam prop nearest the highway's midpoint, looking toward `site.center` (the skyline is visible).
  - `town_view`: 25 m from the nearer town centre (`WORLD_TP_DISTANCE_M`) on the side facing the city, looking at the city centre.
  - `top`: above the Core centre, 120 m above `ground_y`, looking down (pitch). Check `anim_shot.rs` for the camera override; if there is no pitch or height override, add the smallest one in `test_flags.rs` and report it.
- The teleport uses `teleport_player_and_stream` like `apply_town_tp`. Startup order: after `apply_town_tp`, before `apply_gunsmith_town`.

## 3. Perf scene and docs lines
- `tools/perf_scenes.sh`: add `metro) scene_args=(--metro=edge --autowalk --hour=12) ;;` and add `metro` to the `all` set (not the default set).
- `tools/README_TOOLS.md`: one line per flag (`--metro=<spot>` with the five spots), next to the `--town` line (line ~60). Do not touch other docs.

## 4. Save compatibility
- The metro world is not saved; nothing is added to the save unless R38-07 needs a saved flag. If it does, use `#[serde(default)]` (as in save/src/file.rs) and a test that an old save JSON still loads (save/tests/saves.rs). Default: add nothing; discovery already lives in the saved map explore data.

## Tests
- `metro_flags.rs`: each spot's pose: edge looks inward (forward dot toward centre > 0.9), core is at the centre, town_view is 25 m from the town centre, top is 120 m above ground, unknown spot gives None.
- `metro_play.rs`: `name_at` gives "New Halden" inside and None outside; `MetroState` is None when `metro.json` is missing.
- `cargo test` green; `world/tests/golden.rs` unedited and passing; `old saves load`.

## Done when
- A new world starts with New Halden, its towers, highway, jam, haze, map, minimap streets and banner, and Ashford is unchanged (golden passes).
- `--metro=edge|core|highway|town_view|top` each teleport and face the right way; `metro` perf scene exists and is in `--set all`.
- Every shared line used is listed under Worker notes with the file and line.

## Worker notes
(fill in: lines applied from each task's Worker notes, any spot pose changed, the `top` camera override added (if any), any save flag added, questions)

### R38-08a (world side)

Done by the world coder. `cargo test` (whole workspace) passes, `cargo test golden` passes unedited (`world/tests/golden.rs` untouched), `cargo test --lib metro` 57 pass, clippy clean in `world/`. Nothing in `rotland/` was touched.

**Loading.** `WorldGenerator::from_data_dir` now builds New Halden after `attach_outer_world` when `data/metro.json` exists (no file = no metro, `metro_world()` is None; a broken file returns Err like the other data files). So `start_world.rs` needs no call if it already uses `from_data_dir`. Otherwise call `attach_metro` yourself (below).

**Re-exports (`world/src/lib.rs`):** `pub use metro::{Highway, HighwayHeightField, JamProp, MetroConfig, MetroLayout, MetroLook, MetroSite, MetroWorld, TowerKind, TowerSpot, highway_height_at, jam_props_in_chunk, metro_ground_color, place_metro, plan_highway, plan_jam, plan_towers};`. Module `pub mod metro;` and `mod generator_metro;` are there too.

**Public API (`impl WorldGenerator`, file `world/src/generator_metro.rs`):**
- `pub fn attach_metro(&mut self, cfg: MetroConfig, look: MetroLook, roads_data: &RoadsData)`
- `pub fn metro_world(&self) -> Option<&MetroWorld>`
- `pub fn metro_layout(&self) -> Option<&MetroLayout>`
- `pub fn metro_towers(&self) -> &[TowerSpot]` (empty slice when no metro)
- `pub fn metro_highway(&self) -> Option<&Highway>`
- `pub fn metro_highway_field(&self) -> Option<&HighwayHeightField>`
- `pub fn metro_jam(&self) -> &[JamProp]` (empty slice when no metro)
- `pub fn metro_name_at(&self, x: f32, z: f32) -> Option<&str>` (Some("New Halden") inside the square)

**`MetroWorld` (`world/src/metro/state.rs`)**, pub fields: `config: MetroConfig`, `layout: MetroLayout` (`layout.site` is the `MetroSite`), `towers: Vec<TowerSpot>`, `highway: Highway`, `highway_field: HighwayHeightField`, `jam: Vec<JamProp>`, `look: MetroLook` (loaded from data/metro.json, use it for `near_tower_mesh` / skyline; no need to load it again). Methods: `in_tower_footprint(&self, p: Vec2) -> bool`, `paint_ground(&self, color: Color, wx: f32, wz: f32) -> Color`.

**`HighwayHeightField`** gained `pub fn surface_weight(&self, p: Vec2) -> f32` (1 on the asphalt, easing to 0 at the edge, 0 away).

**What the world now does:**
- Terrain: `apply_outer_height` flattens the highway, then the city (`apply_roads_and_water` is the old body, split out unchanged for Ashford). Only runs where `sample.outer > 0` (true beyond the 2.6 km core; the site is at ~6 km).
- Highway river crossings appended to the bridge list (bridge props too).
- Ground paint: `ChunkPaintContext` has `metro: Option<&MetroWorld>`; city streets/sidewalks/lots and highway asphalt are painted after the biome colour, so grass outside stays green.
- Chunk props: `chunk_contents` adds `jam_props_in_chunk` (with the terrain height, plus seat height for bodies). Scatter is dropped on metro streets, in tower footprints and on the highway; lots and parks keep it.

**Test change (only one):** `metro/jam_tests.rs` `no_seats_means_no_bodies` asserted that real data had no seat offsets, but R38-05 filled them in `data/metro.json`. The test now clears `cfg.jam.seat_offsets` first. Its meaning is unchanged.

**Not done / for the game side:** no tower meshes, skyline, haze or map (those are rotland / rotland_ui). Towers are not in chunk props, so the game must build them from `metro_towers()`. The highway is not painted with lanes, only asphalt.

### R38-08b (game side)

Done by the game coder. `cargo test` (whole workspace) passes, `cargo clippy --all-targets` is clean (no warnings anywhere, no dead code in metro_render, metro_haze, metro_ui). `cargo build` passes. One screenshot taken (see below). Nothing in `world/` or `engine/` was touched.

**Files:** new `rotland/src/metro_play.rs` (per-world `MetroPlay`: skyline batch, haze config loaded once from `data/sky.json`, `skyline_fog_scale`), new `rotland/src/app/metro_flags.rs` (pure pose maths for the five spots, `MetroSpot::parse`, tests), new `rotland/src/app/minimap_metro.rs` (metro street strokes). Edited: `rotland/src/main.rs` (`mod metro_play;`), `rotland/src/app/mod.rs` (`mod metro_flags;`, `mod minimap_metro;`), `rotland/src/app/state.rs` (`PlaySide.metro: Option<MetroPlay>`, default None), `rotland/src/app/start_world.rs` (`play.metro = MetroPlay::start(&world_gen)` after the Arc), `rotland/src/streaming.rs` (`LoadedChunk.towers`, near mesh and hull colliders in `upload`), `rotland/src/app/render_frame.rs` (tower items in the ground list, `update_skyline` before the draw list, `metro_haze::apply` after `sky.apply_biome_fog`, skyline draw items appended with `fog_scale`), `rotland/src/app/frame_ui.rs` (banner: `wg.metro_name_at(...)`), `rotland/src/app/test_flags.rs` (`--metro`, `apply_metro_tp`, unit parse test), `rotland/src/app/minimap.rs` (`MinimapShapes` struct carries city, lake and metro; metro streets drawn after Ashford's), `rotland_ui/src/map_ui/state.rs` (`metro_cfg` cached once), `rotland_ui/src/map_ui/draw.rs` (`draw_metro_map` after `draw_city_and_road`), `rotland/src/metro_render/boxes.rs` and `tests.rs` (`tri_count` moved to tests, `is_multiple_of` lint fixes), `tools/perf_scenes.sh` (`metro` scene, in `--set all`), `tools/README_TOOLS.md` (`--metro` line).

**How each item is wired:**
- Towers: `upload` builds `near_tower_mesh` and `tower_hulls` from `world_gen.metro_world()` (ground_y, look, towers). Hulls go in one `add_static_compound_deferred`, pushed to the chunk's colliders (removed on unload like the rest). The mesh is drawn with `fog_scale` 1.0 (so it casts shadows).
- Skyline: `MetroPlay::update_skyline` runs once per frame, fills a reused `HashSet` of loaded chunks, and calls `rebuild_if_needed` (rebuilds only on a loaded-set or 512 m cell change). `draw_items(fog_scale)` with `fog_scale = skyline_fog_end / lighting.fog_end` (min 1), computed after the haze. Items get `fog_scale > 1`, so the engine keeps them out of the shadow pass.
- Haze: `metro_haze::apply(&mut lighting, player, Some(&site), &haze)` right after `sky.apply_biome_fog`.
- Map: `draw_metro_map(painter, state, &view.map_state.explore, layout, &state.metro_cfg, rect, view_size)` after `draw_city_and_road`, only when the world has a metro layout.
- Minimap: `minimap_streets(layout, player, radius_m, radius_px, streets_cfg.street_min_px)` drawn with `street_color` and `width_px * scale`, only when `layout.near(player)`.
- Banner: `|| wg.metro_name_at(p.position.x, p.position.z).map(str::to_string)`.
- Save: nothing added; the metro is not saved.

**Test flag `--metro=<spot>`** (`apply_metro_tp`, after `--town`, before `--gunsmith-town`). Unknown spot: warning, no teleport. Poses (`metro_flags.rs`, pure):
- `edge`: 10 m inside the avenue endpoint nearest the highway end, looking at the site centre.
- `core`: site centre, looking along the avenue nearest the centre (a to b).
- `highway`: 3 m to the right of the jam prop nearest the highway's midpoint, looking at the centre.
- `town_view`: 25 m from the town on the highway (`highway.from_town`) toward the city, looking at the centre.
- `top`: site centre, looking from the highway side into the city, pitch -60 degrees. The camera height comes from the rig's spring (`stored_zoom` and `current_spring_length` set to about 137 m), so the camera is about 120 m above the feet. It sits about 68 m off the Core toward the highway, not straight above it, because the rig has no height-only override. This is the only override added; if the lead wants a straight-down camera, `render.camera` needs a direct position set each frame.
- Facing: yaw = atan2(-dx, -dz), the AGENTS rule.

**Tests:** `metro_flags.rs` (spot names, yaw rule, edge inset and facing, core, highway side offset and facing, town_view 25 m, top 120 m and -60 degrees, polyline midpoint, nearest avenue); `metro_play.rs` (skyline fog scale); `test_flags.rs` (`--metro=Top` parses to "top"). Not added: a test of `metro_pose` on a real world (no cheap way to build a `MetroWorld` in a unit test); `name_at` is the world's `metro_name_at`, tested on the world side.

**Screenshot** (`--metro=town_view`, one run): `/tmp/r38/try_town_view.png`. The teleport worked: the HUD says "CAMP (behind) 3803 m", which matches the Frostmere town-view spot (the log says Frostmere used its fallback centre). The frame shows snow ground, dense snowy pines, a dirt road on the right, the HUD and the hint "Towns have good loot but many zombies". **No New Halden skyline is visible in this frame**, and the city is not in it. I could not tell whether the skyline batch is drawing, or whether the city is out of view from this spot. Fog at 3.8 km and nearer is zero (frame fog 120 to 190 m, divided by the skyline scale), so towers within about 3.8 km should show clearly if they are drawn and in view. Next check for the lead: one `--metro=highway` or `--metro=top` shot, and the count of towers in the skyline batch (`tower_count()`), which is not logged yet.

**Known gaps / for the lead:**
- The skyline batch takes the frame's fog colour, not the grey-green haze colour. The engine has one fog colour per frame, so far from the city the towers fade to the biome fog colour. Fixing that needs a per-item colour in `DrawItem` (engine, not touched).
- `top` has the camera offset discussed above.
- `highway` and `edge` look at the centre, so they are not checked against real street directions in a screenshot (not run).
- `metro_pose` takes the town distance and eye height as arguments (no globals), so the caller passes `WORLD_TP_DISTANCE_M` and the rig's eye height.
- `streaming.rs` is now about 445 lines (over the 300-line guideline, already over before this change).
- `draw_minimap_overlay` used to take 8 arguments; it now takes 7 via `MinimapShapes`.
