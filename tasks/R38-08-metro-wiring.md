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
