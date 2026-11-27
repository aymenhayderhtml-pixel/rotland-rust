# R43-08: Wiring (contents, colliders, ladders, lobby, radio) — after 01-07

Owns:
- `world/src/generator_contents.rs` (the `climb_in_chunk` + `rooftops_in_chunk` calls: POIs, props, crates into the chunk; plus a way to pass `ladders` and `boxes` to rotland, e.g. a new `WorldGenerator::metro_climb_extras(chunk)` beside it)
- `rotland/src/metro_render/hulls.rs` + `parts.rs` (climb towers: `climb_hull_boxes` instead of the one box; drop `lobby_elevators`; fire-escape towers: load the brick part colliders)
- `rotland/src/streaming_metro.rs` / `rotland/src/streaming.rs` (ladders from `ClimbLadder` into `LoadedChunk.ladders`; fire-escape ladders from the brick base/crown `ladder_*_fe*` markers; `ClimbBox` colliders)
- `rotland/src/metro_radio.rs` hookup lines (from R43-07's Worker notes) in the frame/save code they name
- `world/src/metro/mod.rs` (the `mod`/`pub use` lines from R43-01/02)

Read: the Worker notes of R43-01..07, tasks/R43-common.md, `rotland/src/streaming_metro.rs`, `rotland/src/metro_render/{hulls,parts,near}.rs`, `rotland/src/streaming.rs` (`rebuild_ladders`), `world/src/generator_contents.rs`, `rotland/src/ground_detail.rs` + `generator_contents.rs` `keep_scattered_prop` (tower footprints already have no tufts; stop floors are inside towers, so nothing to add: check).

## Wiring
1. **Contents:** climb + rooftop POIs, props and crates go in like the R42 places (`chunk_crates_from` for crates). Stair modules, planks and crates get convex colliders through the normal prop path. Tiny props (digits, signs, ladder segments, the hatch's dark plane) cast no shadows and have no collider (extend `is_no_collider_prop` for `stair_digit_`, `roof_ladder_seg`, `stair_sign_`, `stair_arrow_`).
2. **Hulls:**
   - For a climb tower, the compound gets the `climb_hull_boxes` (8-point boxes, tower-local, like `box_points`) instead of the single upper box.
   - Every other tower keeps exactly today's hull. Add a unit test that proves it.
3. **Lobby:** climb towers skip the `lobby_elevators` part (the stair module stands there). Other lobby props stay.
4. **Ladders:**
   - `ClimbLadder` -> `game::LadderRail::new(foot, deck, id)` into the chunk's ladders.
   - Fire-escape towers: read the base/crown markers with `ladders_from_markers` and the tower matrix (the same as POI buildings).
   - Rails must reach `rebuild_ladders` on chunk load and unload.
5. **Fire escapes:** for `fire_escape_towers` only, add the brick parts' model colliders (`shapes_of(part.model)`) to the chunk compound. Count the colliders added per tower and write the number in Worker notes.
6. **Boxes:** `ClimbBox` -> a box collider (8 points, rotated by yaw).
7. **Radio:** the R43-07 lines; the console position comes from the climb plan.
8. **Stop floors don't drop zombies to the street:**
   - POI zombies spawn at their real height (already true). Check in a run that floor zombies stand on their floor.
   - Check that `metro_spawn_ground` still rejects tower footprints, so street zombies never spawn inside.

## Done when
Wired; the gate passes; fill `## Worker notes

### Files changed
- `world/src/generator_contents.rs`: merged `climb_in_chunk_with` (loading `data/climb_floors.json` via `OnceLock`) and `rooftops_in_chunk` into `chunk_contents` (`pois`, `props`) and `chunk_crates_from` (`crates`), and added `WorldGenerator::metro_climb_extras(coord) -> (Vec<ClimbLadder>, Vec<ClimbBox>)`.
- `rotland/src/props.rs`: extended `is_no_shadow_metro_prop` with `stair_digit_`, `stair_sign_`, `stair_arrow_`, `roof_ladder_seg`.
- `rotland/src/metro_render/parts.rs`: added `chunk_tower_parts_with_climb` (drops `lobby_elevators` on climbable towers) and `tower_parts_for_index` (shared `parts_for_tower` helper).
- `rotland/src/metro_render/near.rs`: updated `place_chunk_towers` to pass `climb_towers(metro)` indices to `chunk_tower_parts_with_climb`.
- `rotland/src/metro_render/hulls.rs`: added `tower_hulls_with_climb` (climb towers get `climb_hull_boxes`; non-climb towers keep the single upper box), `aabb_points`, and unit test `non_climb_towers_keep_exact_single_hull_and_climb_gets_multi_box_hull`.
- `rotland/src/streaming_metro.rs`:
  - extended `is_no_collider_prop` with `stair_digit_`, `roof_ladder_seg`, `stair_sign_`, `stair_arrow_` (`roof_hatch` keeps its 4 curb colliders).
  - updated `load_metro_chunk` to use `tower_hulls_with_climb`, load `fire_escape_towers` non-base part colliders + `ladders_from_markers` on brick base/crown markers, and wire `metro_climb_extras` (`ClimbLadder` -> `LadderRail`, `ClimbBox` -> 8-point compound boxes).
- `rotland/src/app/frame_world.rs`, `rotland/src/app/save_load.rs`, `rotland/src/app/start_world.rs`: wired `play.metro_radio` (`can_open_crate`, `interact_tick`, `advance_playback`, `to_save`/`restore_save` on `systems.extra["metro_radio"]`, and `metro_radio_console_pos` from `climb_towers` + `climb_in_chunk`).

### Collider counts (per tower)
- **Climb tower:**
  - Lobby base: 10–15 colliders (from `tw_<style>_<size>_base` via `tower_bases`).
  - `climb_hull_boxes`: 4 boxes per non-stop floor + 8 boxes per stop floor (4 slab + 4 outer wall) + up to 7 boxes for the roof deck around shaft & hatch (e.g., 16-floor Midtown tower with 3 stop floors = 13×4 + 3×8 + 7 = 83 hull boxes).
  - Tallest landmark mast: +6 `ClimbBox` colliders (1 platform slab + 5 rail segments).
- **Fire-escape tower:**
  - Base (`tw_brick_{16,24}_base`): 15 colliders (via `tower_bases`).
  - Upper hull: 1 box (via `tower_hulls_with_climb`).
  - Crown (`tw_brick_{16,24}_crown`): 4 colliders.
  - Floors (`tw_brick_{16,24}_floor*`): 10 colliders per `_floor`, 5 per `_floor_b` / `_floor_c`.
  - Total for an `F`-floor fire-escape tower: `20 + (5..10) * F` colliders (e.g., 6 floors ≈ 50–80 colliders).

### Notes
- `metro_spawn_ground` in `rotland/src/play_metro.rs` still rejects `metro.in_tower_footprint(...)` and spawns > 1.5 m above terrain, so street zombies never spawn inside towers.
- `cargo` / `tools/gate.sh` were not run in this subagent pass (per lead rules).` (collider counts per climb tower and per fire-escape tower).

## Worker notes

### Files changed
- `world/src/generator_contents.rs`: merged `climb_in_chunk_with` (loading `data/climb_floors.json` via `OnceLock`) and `rooftops_in_chunk` into `chunk_contents` (`pois`, `props`) and `chunk_crates_from` (`crates`), and added `WorldGenerator::metro_climb_extras(coord) -> (Vec<ClimbLadder>, Vec<ClimbBox>)`.
- `rotland/src/props.rs`: extended `is_no_shadow_metro_prop` with `stair_digit_`, `stair_sign_`, `stair_arrow_`, `roof_ladder_seg`.
- `rotland/src/metro_render/parts.rs`: added `chunk_tower_parts_with_climb` (drops `lobby_elevators` on climbable towers) and `tower_parts_for_index` (shared `parts_for_tower` helper).
- `rotland/src/metro_render/near.rs`: updated `place_chunk_towers` to pass `climb_towers(metro)` indices to `chunk_tower_parts_with_climb`.
- `rotland/src/metro_render/hulls.rs`: added `tower_hulls_with_climb` (climb towers get `climb_hull_boxes`; non-climb towers keep the single upper box), `aabb_points`, and unit test `non_climb_towers_keep_exact_single_hull_and_climb_gets_multi_box_hull`.
- `rotland/src/streaming_metro.rs`:
  - extended `is_no_collider_prop` with `stair_digit_`, `roof_ladder_seg`, `stair_sign_`, `stair_arrow_` (`roof_hatch` keeps its 4 curb colliders).
  - updated `load_metro_chunk` to use `tower_hulls_with_climb`, load `fire_escape_towers` non-base part colliders + `ladders_from_markers` on brick base/crown markers, and wire `metro_climb_extras` (`ClimbLadder` -> `LadderRail`, `ClimbBox` -> 8-point compound boxes).
- `rotland/src/app/frame_world.rs`, `rotland/src/app/save_load.rs`, `rotland/src/app/start_world.rs`: wired `play.metro_radio` (`can_open_crate`, `interact_tick`, `advance_playback`, `to_save`/`restore_save` on `systems.extra["metro_radio"]`, and `metro_radio_console_pos` from `climb_towers` + `climb_in_chunk`).

### Collider counts (per tower)
- **Climb tower:**
  - Lobby base: 10–15 colliders (from `tw_<style>_<size>_base` via `tower_bases`).
  - `climb_hull_boxes`: 4 boxes per non-stop floor + 8 boxes per stop floor (4 slab + 4 outer wall) + up to 7 boxes for the roof deck around shaft & hatch (e.g., 16-floor Midtown tower with 3 stop floors = 13×4 + 3×8 + 7 = 83 hull boxes).
  - Tallest landmark mast: +6 `ClimbBox` colliders (1 platform slab + 5 rail segments).
- **Fire-escape tower:**
  - Base (`tw_brick_{16,24}_base`): 15 colliders (via `tower_bases`).
  - Upper hull: 1 box (via `tower_hulls_with_climb`).
  - Crown (`tw_brick_{16,24}_crown`): 4 colliders.
  - Floors (`tw_brick_{16,24}_floor*`): 10 colliders per `_floor`, 5 per `_floor_b` / `_floor_c`.
  - Total for an `F`-floor fire-escape tower: `20 + (5..10) * F` colliders (e.g., 6 floors ≈ 50–80 colliders).

### Notes
- `metro_spawn_ground` in `rotland/src/play_metro.rs` still rejects `metro.in_tower_footprint(...)` and spawns > 1.5 m above terrain, so street zombies never spawn inside towers.
- `cargo` / `tools/gate.sh` were not run in this subagent pass (per lead rules).
