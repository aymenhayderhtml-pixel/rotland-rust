# R39-08: Test flags (lobby, landmark, tower style, map), README lines, bus underside

Owns: rotland/src/app/metro_flags.rs (the new spots and poses; existing functions keep their behaviour), rotland/src/app/test_flags.rs (ONLY the `--metro` value list, the new `--tower-style` and `--map` flags, and their apply calls and parse tests), tools/README_TOOLS.md (the flag lines only), tools/blender/metro/jam_bus.py (the bus_city_side underside and axles only; upright bus unchanged). Shared files (`rotland/src/main.rs`, `rotland/src/app/mod.rs`, `rotland/src/app/state.rs`, `rotland/src/app/startup.rs`, `Cargo.toml`) belong to the lead: report the exact lines. NOT owned: tools/perf_scenes.sh (the `metro` perf scene stays exactly as it is: `--metro=core --autowalk --hour=12`).
Runs AFTER R39-01..07 (it reads `TowerStyle`, `TowerSpot.style`, `MetroWorld.towers`, the lobby door direction from R39-01, and `MetroWorld.highway`).
Read: tasks/R39-common.md ("Test flags", "Perf"), rotland/src/app/metro_flags.rs (`MetroSpot`, `MetroSpot::parse`, `MetroPose`, `TopView`, `nearest_avenue`, `edge_pose`, `core_pose`, `highway_pose`, `town_view_pose`, `top_pose`, `metro_pose`, its tests), rotland/src/app/test_flags.rs (`TestFlags` fields, `parse` with `args.value("--metro")` at ~line 117, `apply_startup` at ~line 340, `apply_metro_tp` ~line 541, the `--gunsmith` handling ~line 382 and `apply_gunsmith_town`, the parse tests ~line 945-961), rotland_ui/src/hub_ui (`HubUiState::new`, `HubScreen::Tab`), rotland/src/app/frame_input.rs (line ~165: how `HubTab::Map` opens the hub; `ui.hub`), rotland/src/app/frame_ui.rs (line ~477: the `map: Some(MapView {...})` argument), rotland/src/app/state.rs (`UiState.hub`, `cursor_captured`), world/src/metro/towers.rs and tower_parts.rs and styles.rs (R39-01), tools/README_TOOLS.md, tools/blender/metro/jam_bus.py, tasks/archive/R38-05-jam-models.md (Worker notes: "bus_city_side is the same bus rotated 90 degrees on its length axis, so the windowed side faces up, the underside faces sideways and the wheels stick out into the air; origin on the ground").

## 1. `--metro` spots (`metro_flags.rs`, `test_flags.rs`)
- Add two spots to `MetroSpot`: `Lobby` and `Landmark`. Keep `Edge`, `Core`, `Highway`, `TownView`, `Top` unchanged. `parse` accepts `lobby` and `landmark` (any case, like the others). Update the warning text in `apply_metro_tp` to list all seven.
- **`Lobby`**: the nearest tower with a lobby (nearest to the player's spawn: use the Core centre as the reference, not the camp). Pose: stand at the base centre of that tower (`TowerSpot.pos`, ground height from `site.ground_y`) and face its door (the tower's front, from `TowerSpot.yaw`: the front direction is the same one R39-01 uses for the lobby; face it). Inside the walls, so the door is visible. Eye height from the camera rig as for the others.
- **`Landmark`**: the nearest landmark tower (`TowerSpot.landmark`, so pick the tallest by `height` if none is nearby). Pose: on the nearest avenue (`nearest_avenue`), about 40 m from its base, facing the landmark, looking up with a small upward pitch (use `TopView`-like pitch: a new `pitch` only if the rig needs it; reuse `top_pose`'s pitch type).
- `--tower-style=<key>` (new flag, value `brick|stone|glass|slab|deco`, any case; unknown = log warning and ignore): only with `--metro=core`. The pose is the nearest tower of that `TowerStyle` to the Core centre, standing on its front street facing the tower. If the style is missing in the city, log a warning and fall back to `--metro=core`. Add `TestFlags.tower_style: Option<TowerStyle>` (parse with `TowerStyle::parse`, owned by R39-01 — if R39-01 did not add a parser, write the 5-line match in `metro_flags.rs`, not in `styles.rs`).
- Pure pose logic goes in `metro_flags.rs` (testable): `lobby_pose(tower, site, front_dir) -> MetroPose`, `landmark_pose(tower, avenue_point, ...) -> MetroPose`, `style_pose(tower, ...)`. The `apply_metro_tp` path calls them.

## 2. `--map` (screenshot of the big map)
- New flag `--map` (bool). In `apply_startup`, after the metro/gunsmith flags, set `ui.hub = Some(hub_ui::HubUiState::new(HubScreen::Tab(HubTab::Map)))` and `ui.cursor_captured = false`, the same way the other test screens open. It must not open when a `--gunsmith` or `--metro` screen already took the UI (check `ui.gunsmith.is_none()` and `ui.hub.is_none()` first).
- The `MapView` (`frame_ui.rs`) already draws New Halden's outline and streets (R38-06, `rotland_ui/src/map_ui/draw_city.rs`). Do not change the map UI; check in the screenshot that the outline and streets show (lead).

## 3. Bus underside (`tools/blender/metro/jam_bus.py`)
- `bus_city_side` (the bus lying on its side, `_build(True)`) must show its underside and axles clearly: keep the chassis rails, axles and fuel tank visible from the outside. Add: a lower skirt cut-out so the axle and rail stubs show from the road side; wheels sit on the ground in the lying pose (no wheel floating in the air: lower the whole lying bus so the lowest wheel touches z=0, or set the wheels to the ground line).
- Only the lying variant changes; `bus_city` (upright) must stay the same. Triangle budget for the bus stays 12k-20k (R38 used 13,116 for the side bus).
- The lead runs the Blender build once (`blender -b --python tools/blender/metro/jam_bus.py` through the usual path) and checks the log has no `Traceback`; the worker never runs Blender.

## 4. README lines (`tools/README_TOOLS.md`)
- Add one line each for: `--metro=lobby`, `--metro=landmark`, `--tower-style=<brick|stone|glass|slab|deco>` (with `--metro=core`), `--map`. Keep the existing format of the file's flag lines. Do not rewrite other lines.

## Tests (in `metro_flags.rs` and `test_flags.rs` parse tests)
- `MetroSpot::parse("lobby")`, `("LANDMARK")` and unknown spots (`"roof"` stays None).
- `--tower-style=deco` parses to `Deco`; `--tower-style=tall` gives None (warning path).
- `--map` parses to true; `--map` alone opens no gunsmith or metro flag.
- `lobby_pose`: the pose faces the door (the yaw points from the pose to the door within 5 degrees) and the pose is inside the tower's footprint.
- `landmark_pose`: the pose is 40 m from the landmark base and the landmark is in front (positive dot product).
- `style_pose`: the picked tower has the requested style.
- Existing metro flag tests still pass (their values do not change).

## Done when
- `--metro=lobby` puts the player inside the lobby with the door in view; `--metro=landmark` puts the player on the street looking up at the landmark (both checked by the lead in `tasks/R39-round.md` Checks).
- `--tower-style=<key>` with `--metro=core` shows the nearest tower of that style; `--map` opens the big map.
- The `metro` perf scene in `tools/perf_scenes.sh` is unchanged (diff shows no edit there).
- Bus side model has its underside and axles visible; upright bus unchanged.
- `tools/gate.sh --task tasks/R39-08-flags-wiring.md` ends with `GATE: PASS` (workers only).
- Shared-file lines for the lead (`rotland/src/main.rs` or `app/mod.rs` if `metro_flags` needs a new `mod` line, and the `apply_startup` call for `--map`) are in Worker notes.

## Worker notes
- **Files changed:**
  - `rotland/src/app/metro_flags.rs`: added `MetroSpot::Lobby` and `MetroSpot::Landmark` (parsed any-case), `tower_front_dir`, `lobby_pose`, `landmark_pose` (reusing `TopView` with `+35 deg` upward pitch and `4.0 m` spring), `nearest_tower_of_style`, `style_pose`, updated `metro_pose`, and added unit tests (`lobby_pose_is_inside_footprint_and_faces_the_door`, `landmark_pose_is_40m_from_base_and_looks_up_at_landmark`, `style_pose_picks_requested_style_and_faces_tower`, plus `lobby`/`LANDMARK` parse checks).
  - `rotland/src/app/test_flags.rs`: added `TestFlags.tower_style: Option<TowerStyle>` and `TestFlags.map: bool`, parsed `--tower-style` (via `TowerStyle::parse`, warning on unknown) and `--map`, updated `apply_metro_tp` warning and `--tower-style` override when `--metro=core` (falling back to `--metro=core` with a warning if missing), opened `HubTab::Map` in `apply_startup` for `--map` when `ui.gunsmith.is_none() && ui.hub.is_none()`, and updated `parses_all_test_flags_and_defaults` tests.
  - `tools/README_TOOLS.md`: added flag lines for `--metro=lobby`, `--metro=landmark`, `--tower-style`, and `--map`.
  - `tools/blender/metro/jam_bus.py`: updated `bus_city_side` (`lying=True`) to cut out the lower skirt panels around the axles/mid-chassis, include the ground-side wheels (`side_only=False`), and omit the ground-side mirror/rear marker so the lowest wheels sit on `z=0`; `bus_city` (`lying=False`) is unchanged.
- **Shared-file lines for the lead:** none needed (`rotland/src/app/mod.rs` already has `pub mod metro_flags;` and `rotland/src/app/startup.rs` already calls `super::test_flags::apply_startup(...)` at line 231).
- **Blender build command for lead:** run `blender -b --python tools/blender/metro/jam_bus.py` (or the metro jam batch builder) and re-bake `assets/models.pack`, then verify `bus_city_side` underside/axles and wheels on `z=0`.
- **Checks:** did not run `cargo`, `tools/gate.sh`, `blender`, or the game per subagent rules.
