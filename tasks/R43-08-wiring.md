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
Wired; the gate passes; fill `## Worker notes` (collider counts per climb tower and per fire-escape tower).

## Worker notes
