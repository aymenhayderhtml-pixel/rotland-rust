# R40-08: Wiring, colliders, shadow rules, flags (fallen, leaning, burnt, carwall, camp), landmark fix, README

Runs AFTER R40-01, 03, 04, 05, 06, 07 have merged (the lead gives it the finished names). Do not start it before.

Owns: world/src/generator_contents.rs (ONLY the new calls in `chunk_contents` for overgrowth, damage and traces props), rotland/src/metro_render/parts.rs (ONLY the replaced-tower skip and the `DamageMode` choice in `chunk_tower_parts`), rotland/src/metro_render/hulls.rs (ONLY: replaced towers give no hull and no base), rotland/src/props.rs (ONLY `casts_shadow` and the no-collider list for tiny props), rotland/src/app/metro_flags.rs (new spots, poses, landmark fix and its test), rotland/src/app/test_flags.rs (ONLY the `--metro=` doc line at ~line 55), docs/WORLD.md (New Halden section lines), tools/README_TOOLS.md (ONLY if it lists the `--metro=` flags). Shared files belong to the lead; report the exact lines: `rotland/src/streaming.rs`, `rotland/src/main.rs`, `rotland/src/app/state.rs`, `rotland/src/app/mod.rs`, `world/src/lib.rs`.

Read:
- tasks/R40-common.md (the "Wiring + flags" entry and the lead answers: `tower_is_replaced` and the burnt block's `DamageMode::All` are wired HERE at the tower loop; the leaning tower is ONE baked model).
- tasks/R40-01-overgrowth-place.md, R40-03-damage-place.md, R40-06-traces-place.md (the function names and the `(i32, i32)` chunk signature; `DamageSites::tower_is_replaced`, `tower_is_burnt`, `MetroWorld.damage`, `camp_sites`).
- tasks/R40-07-tower-variety.md (the `tower_parts_with_damage` and `DamageMode` names, read the exact enum variants).
- world/src/generator_contents.rs: `chunk_contents` (line ~30; the metro jam call at ~121 is the pattern to copy: `crate::metro::jam_props_in_chunk(&metro.jam, coord, &|x, z| self.height(x, z))`). Note `chunk_contents` takes `IVec2`; the new fns take `(i32, i32)`.
- rotland/src/streaming.rs: the chunk load path (~line 198 `chunk_contents`, ~line 234 `shapes_of`, ~line 317 `add_static_compound_deferred` for `static_props`). Non-tree props get a compound collider from their model shapes automatically; tree_ props take the `TreeSlot` route. Read both before deciding anything about colliders.
- rotland/src/metro_render/parts.rs (`chunk_tower_parts`, `tallest_landmark_index`), rotland/src/metro_render/hulls.rs (`tower_hulls`, `tower_bases`), rotland/src/metro_render/near.rs (`place_chunk_towers`).
- rotland/src/props.rs: `is_ground`, `range_for`, `casts_shadow`, `test_ground_and_shadow_rules` (~line 644), `is_small_prop` (set in `load_model`), `SMALL_PROP_SHADOW_DIST`.
- rotland/src/app/metro_flags.rs: `MetroSpot` enum and `parse` (~line 48), `metro_pose` (~line 209), `landmark_pose` (~line 158) with `LANDMARK_VIEW_DIST_M` (40) and `LANDMARK_PITCH_DEG` (35), `nearest_tower_of_style`, `style_pose`, the landmark test (~line 408, asserts 40 m: update it), `lobby_pose`, `top_pose`.
- physics/src/mover.rs: `max_step_height` (0.35). Rubble must stay climbable under it.
- docs/WORLD.md (where New Halden lines live), tools/README_TOOLS.md (the flags section, if any).

## What to build
1. **Colliders:** car walls, rubble fields, the fallen section and the leaning tower are non-tree static props, so the streaming path already gives them compound colliders from their model shapes. Confirm that in the code. Do not add a second collider. Add ONE exception list in props.rs or streaming (whichever holds the existing rule for "no collider"): tiny props (`weeds_`, `moss_`, `leaves_`, `graffiti_`, `poster_`, `vine_`, `grass_tall_patch`, `roof_bush`, `roof_tree`) get NO collider. `tree_asphalt` takes the tree route if its id starts with `tree_` (check the streaming condition); `sapling_city` is a plain prop.
2. **Walkable rubble:** the rubble and the fallen section must let the mover climb (`max_step_height` 0.35). Check the R40-04 notes for the step height of the rubble; if none, note it as a doubt. No code change needed for slopes.
3. **Replaced towers:** in `chunk_tower_parts` (parts.rs), skip towers with `metro.damage.tower_is_replaced(i)`. The index is the one from `towers` (not the chunk list). In `tower_hulls` and `tower_bases` (hulls.rs), skip them too. The leaning tower and the fallen section's models come from the damage props (R40-03), not from tower parts.
4. **Burnt block:** towers with `metro.damage.tower_is_burnt(tower.block)` call R40-07's `tower_parts_with_damage(.., DamageMode::All)`; all others keep the normal call. No other change.
5. **Chunk props:** in `chunk_contents` (generator_contents.rs), after the jam line, add `overgrowth_props_in_chunk`, `damage_props_in_chunk`, `traces_props_in_chunk` (each through `metro_world()`), with `coord` converted to `(coord.x, coord.y)`. The `keep_scattered_prop` rule does not apply to these (they are not scatter).
6. **Shadows:** `casts_shadow` in props.rs returns false for the tiny prefixes in item 1 plus `graffiti_` and `poster_` (decals), and the flat-ground list `is_ground` already covers the grass. Extend `test_ground_and_shadow_rules`.
7. **Flags** (`--metro=fallen|leaning|burnt|carwall|camp`), new `MetroSpot` variants and `parse` lines in metro_flags.rs:
   - `fallen`: stand on the street of the fallen section, 30 m from its centre, facing the middle, eye height.
   - `leaning`: 40 m from the leaning tower on the nearest avenue, looking up at its top (same rule as `landmark`).
   - `burnt`: 35 m from the burnt block centre on the nearest street, facing it.
   - `carwall`: 12 m before the first car wall on its street, facing it, eye height (the gap must be walkable: check it is not blocked).
   - `camp`: on the first `camp_sites` roof (R40-06), 6 m off the camp centre, eye height on the roof, looking across it.
8. **Landmark fix (R39 leftover 4):** `metro_pose` picks the NEAREST landmark to the centre, not the tallest. Use `tallest_landmark_index` (parts.rs) or the `towers` max height. `LANDMARK_VIEW_DIST_M` becomes 60. The pitch is computed so the top of the crown (`TowerSpot.height`) sits in the upper part of the view (aim at 85% of its height from the eye). Update the landmark test to 60 m and assert the crown is in view.
9. **README lines:** docs/WORLD.md New Halden section: the R40 layers (overgrowth, damage set pieces, survivor traces, tower variety), the new flags, and the FPS rule. tools/README_TOOLS.md only if it lists `--metro=` flags.

## Tests
- props.rs: tiny prefixes and decals cast no shadow; `grass_`, `tree_` and `car_wall_stack` rules unchanged.
- metro_flags.rs: the five new spot names parse (any case) and an unknown name is rejected; the landmark pose is 60 m from the tallest landmark and the crown top is inside the view (vertical angle check).
- parts.rs: `chunk_tower_parts` gives nothing for a replaced tower (use a `DamageSites` built in the test).
- Every flag pose lies on the ground inside the city square (`MetroLayout::contains`).
- `world/tests/golden.rs` passes UNEDITED.

## Perf rule (the lead measures, not the worker)
- The lead runs `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland` before and after the round, and nothing else (no `--set all`).
- Pass = AVERAGE FPS at least 20. The 1% low is written for information only.
- If the average is under 20, the lead lowers the overgrowth density in data first. Never strip close detail.
- The worker only writes the README line for this rule; it runs no measurement.

## Done when
- The streaming path, the no-collider rule, the replaced skip and the burnt call are wired; the five flags work in code and in the README.
- Shared-file lines (streaming.rs, lib.rs, main.rs if any) are in Worker notes.
- Doubts (rubble step height, collider duplicates, decal facing) are in Worker notes.

## Worker notes
