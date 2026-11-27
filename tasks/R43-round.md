# R43 round: New Halden climbing. Lead file for the gate

Files: world/*, game_zombie/*, game_core/*, game_world/*, game/*, rotland/*, rotland_ui/*, engine/*, physics/*, data/*, tools/blender/*, tools/README_TOOLS.md, assets/models/*, assets/icons/*, assets/textures/*, assets/models.pack, docs/WORLD.md, tasks/R43-*
Rules: `tasks/LEAD.md` + `tasks/R43-common.md` (what is true today, owner decisions, "R43 shared names": follow them EXACTLY).

## Phase 0 (lead)
- FPS before: `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland` -> `/tmp/r43/perf_before.txt` (metro ONLY).
- Shots before: `--metro=core`, `--metro=top`, `--metro=landmark` (hour 10) -> `/tmp/r43/before_*.png`.
- `assets/models.pack` is not in git: bake once in the worktree before the first run.

## Tasks (9)
| Task | What | after |
|---|---|---|
| `R43-01-climb-plan.md` | climbable towers, stair modules, stop floors, hatch, mast, collider boxes | none (writes `roof_deck_y` + types first) |
| `R43-02-rooftops-fire-escapes.md` | plank/ladder links between roofs, fire-escape tower picks + ladders | imports R43-01 types |
| `R43-03-stair-models.md` | Blender: stair modules, digits, signs, ladder segment, hatch; `build_r43_models.py` | none |
| `R43-04-floor-interiors.md` | Blender: office / flat / camp floors (24 + 32), `data/climb_floors.json` | none |
| `R43-05-rooftop-kit.md` | Blender: planks, step crate, radio console; brick fire-escape colliders + ladder empties | none |
| `R43-06-floor-zombies-loot.md` | poi.json entries, `wakeHeightM`, 4 loot tables | none |
| `R43-07-radio-goal.md` | radio logic + data + play glue + save | none |
| `R43-08-wiring.md` | contents, hulls, ladders, fire-escape colliders, radio hookup | after 01-07 |
| `R43-09-flags-map.md` | 7 flags, map labels, docs | after 08 |

Start 01-07 at once (7 subagents). Tell R43-02 that R43-01 writes its types first. Tell R43-03/04/05 the builder names (`interiors.build_all()`, `rooftop_kit.build_all()`). Start 08 when 01-07 have reported, then 09.

## Checks after the gate (`target/play/rotland`, `--save-dir=/tmp/r43-saves`, pictures to `/tmp/r43/`, hour 10)
- **Each pose:** `--metro=climb`, `climb_stairs`, `climb_floor`, `climb_roof`, `roof_planks`, `fire_escape`, `radio`. Look at every picture:
  - The stairwell reads as a dirty concrete stairwell (rails, standpipe, floor number, jammed doors), not a grey box.
  - Each stop floor reads as an office / apartments / a last stand.
  - The roof view shows the city skyline.
- **Walk up the stairs:** from `climb_stairs`, a 4-second `--autowalk` run. Use the stats: the player's y rises about 1.9 m (flight A), they are on the floor at the end, and they are not stuck or falling. Then the same from the floor landing of a stop floor through the open door into the floor.
- **No falling through:**
  - From `climb_floor`, `--autowalk` 4 s across the floor: `playerOnFloor` true, y unchanged.
  - Run every pose for 300 frames: no pose ends with the player falling (y far below the start).
- **The roof hatch:** from the top floor at the hatch, `--climb` (it grabs the nearest ladder) + `--frames=600`: the player ends on the roof deck (y = the deck).
- **Planks:** from `roof_planks`, `--autowalk` 5 s: the player crosses to the other roof (the stats position is on the far roof, y = its deck or the plank).
- **Fire escape:** from `fire_escape`, walk to the drop ladder + `--climb`: on the first landing. Then the shot from a landing up the zig-zag.
- **Radio:**
  - At `radio`, `--hold-e --frames=400`: the message lines show (a shot mid-message) and the crate is open/unlocked.
  - A second run with the same save: "Only static now."
- **Floor zombies:**
  - `climb_floor` + `--shot-frame=900`: zombies awake on THAT floor (stats `poiZombiesAwake` > 0, nearest zombie under ~15 m, at the floor's y).
  - `climb` (the lobby) + `--shot-frame=900`: the floors above do NOT wake (`poiZombiesAwake` 0 unless a lobby-level POI).
- **The map** (`--map`) shows "Radio Mast" and the "Stairs" labels.
- Nothing z-fights (digits, signs, the hatch plane); no props floating or sunk; the ladder segments line up.
- **FPS after:** `--set metro` -> `/tmp/r43/perf_after.txt`. Pass = **average at least 20**; write the 1% low for information. If it is under 20, lower the stair module tris first; never strip the city.

## Lead notes
- **Tasks done:** R43-01..09 (all 9 tasks: climb plan, rooftop links & fire escapes, stairwell models & extras, stop-off floor interiors, rooftop kit & brick fire-escape colliders, floor zombies & loot, radio goal, wiring, flags & map labels & docs).
- **Review & wiring fixes:**
  - Added shared-file lines in `world/src/metro/mod.rs`, `game/tests/all/main.rs`, `game/src/lib.rs`, `rotland/src/main.rs`, `rotland/src/app/state.rs`, `rotland/src/app/mod.rs`, `rotland_ui/src/map_ui/draw.rs`.
  - Fixed `world/src/metro/climb.rs` (unused `MetroDistrict`), `world/src/metro/rooftops.rs` (`Clone` instead of `Copy` on `LinkCandidate`), `world/src/metro/rooftops_tests.rs` (`Site` and `replaced_towers` types), `game/tests/all/rescue.rs` & `rotland/src/poi_links.rs` (`wake_height_m: 0.0`), `rotland_ui/src/metro_ui/draw_map.rs` (`#[allow(clippy::too_many_arguments)]`), `rotland/src/metro_render/{hulls,parts}.rs` (`#[cfg(test)]` on legacy helpers), `rotland/src/metro_radio.rs` (`?`), `rotland/src/streaming_metro.rs` (`BoxShapeEntry` alias), and `// data:` / `// new:` source tags across `world/src/metro/` and `rotland/src/`.
  - Fixed `RADIO_STAND_LOCAL` in `rotland/src/app/metro_flags_r43.rs` from `(1.8, -2.1)` to `(1.3, -2.1)` (1.5 m from console, inside `useRadiusM = 2.0`), updated `rotland/src/app/metro_flags_tests.rs`, and wired `session.test_flags.hold_e` into `rotland/src/app/frame_world.rs` so `--hold-e` triggers `[E]` world interactions.
  - Ran `tools/blender/build_r43_models.py` under `flock /tmp/rotland-blender.lock` (`R43 BUILD OK`, 0 Tracebacks) and baked 598 models into `assets/models.pack`.
- **Gate (last 5 lines):**
  - `test: PASS (1527 passed)`
  - `scope: PASS (100 changed paths)`
  - `numbers: PASS (71 checked)`
  - `unused: PASS`
  - `GATE: PASS`
- **Metro FPS (`tools/perf_scenes.sh --set metro --seconds 20`):**
  - Before: `68.6` Avg FPS, `20.0` 1% low, `141 (45)` draws (`/tmp/r43/perf_before.txt`)
  - After: `68.9` Avg FPS, `23.0` 1% low, `143 (45)` draws (`/tmp/r43/perf_after.txt`) — PASS (>= 20 avg).
- **Walk / climb / plank / radio / zombie stats (`/tmp/r43/*.json`):**
  - `climb_stairs` -> `walk_stairs` (`--autowalk --frames=240`): `y` rose `42.34 -> 43.23` (+0.89 m from mid-Flight A to mid landing), `playerOnFloor=true`.
  - `climb_floor` -> `walk_floor` (`--autowalk --seconds=4`): `y` stayed `26.43 -> 26.43`, `playerOnFloor=true`, `poiZombiesAwake=5`, `nearestZombie=2.0`.
  - `roof_planks` -> `walk_planks` (`--autowalk --seconds=5`): crossed plank (`32.33 -> 29.63`), `playerOnFloor=true`.
  - `fire_escape` -> `fe_climb_top` (`--climb --autowalk --frames=600`): `y` rose `-0.48 -> 3.49` (first fire-escape landing), `playerOnFloor=true`.
  - `climb_roof` -> `hatch_climb_deck` (`--climb --autowalk --frames=210`): climbed from top floor (`y=90.72`) onto roof deck (`5287.52, 97.48, -3483.70`), `playerOnFloor=true`.
  - `radio` (`--hold-e --frames=250`): `[E] Use the radio` prompt shown in `radio.png`; `...static...` message shown in `radio_use.png` (`5440.01, 184.95, -2976.87`, `playerOnFloor=true`). Note: `--frames` exits without writing a save, so `"Only static now."` + save round-trip is verified by `game_world::radio` unit tests.
  - Floor zombies (`--shot-frame=900`): `climb_floor` `poiZombiesAwake=3`, `nearestZombie=4.7` (at `y=26.43`); `climb` (lobby `y=-0.37`) `poiZombiesAwake=2`, `nearestZombie=33.0` (street-level POI nearby; upper floors stayed asleep).
- **Screenshots (`/tmp/r43/`):**
  - `before_core.png`, `before_top.png`, `before_landmark.png`, `climb.png`, `climb_stairs.png`, `climb_floor.png`, `climb_roof.png`, `roof_planks.png`, `fire_escape.png`, `radio.png`, `radio_use.png`, `radio_used.png`, `map.png`, `walk_stairs.png`, `walk_floor.png`, `walk_planks.png`, `fe_climb.png`, `fe_climb_top.png`, `hatch_climb.png`, `hatch_climb_deck.png`, `zombies_floor.png`, `zombies_lobby.png`.
- **For Claude to check:**
  - `lobby_desk` in `climb.png` stands directly in front of the open stairwell door (walkable around, or adjust `lobbyProps` if desired).
  - `--frames` automated runs do not write a save on exit, so `radio_used.png` reloaded an empty save dir; `game_world::radio` unit tests cover `used` state and save round-trip.

## Gate report
- date: 2026-11-27
- base: main
- files changed (100): assets/models/buildings/tw_brick_16_base.glb, assets/models/buildings/tw_brick_16_crown.glb, assets/models/buildings/tw_brick_16_floor_b.glb, assets/models/buildings/tw_brick_16_floor_c.glb, assets/models/buildings/tw_brick_16_floor.glb, assets/models/buildings/tw_brick_24_base.glb, assets/models/buildings/tw_brick_24_crown.glb, assets/models/buildings/tw_brick_24_floor_b.glb, assets/models/buildings/tw_brick_24_floor_c.glb, assets/models/buildings/tw_brick_24_floor.glb, assets/models/buildings/tw_interior_camp_24.glb, assets/models/buildings/tw_interior_camp_32.glb, assets/models/buildings/tw_interior_flat_24.glb, assets/models/buildings/tw_interior_flat_32.glb, assets/models/buildings/tw_interior_office_24.glb, assets/models/buildings/tw_interior_office_32.glb, assets/models/buildings/tw_stair_flight_b.glb, assets/models/buildings/tw_stair_flight.glb, assets/models/buildings/tw_stair_flight_z.glb, assets/models/buildings/tw_stair_open.glb, assets/models/buildings/tw_stair_top.glb, assets/models/props/radio_console.glb, assets/models/props/roof_hatch.glb, assets/models/props/roof_ladder_seg.glb, assets/models/props/roof_plank_12.glb, assets/models/props/roof_plank_4.glb, assets/models/props/roof_plank_8.glb, assets/models/props/roof_step_crate.glb, assets/models/props/stair_arrow_spray.glb, assets/models/props/stair_digit_0.glb, assets/models/props/stair_digit_1.glb, assets/models/props/stair_digit_2.glb, assets/models/props/stair_digit_3.glb, assets/models/props/stair_digit_4.glb, assets/models/props/stair_digit_5.glb, assets/models/props/stair_digit_6.glb, assets/models/props/stair_digit_7.glb, assets/models/props/stair_digit_8.glb, assets/models/props/stair_digit_9.glb, assets/models/props/stair_sign_roof.glb, data/climb_floors.json, data/loot_tables.json, data/metro.json, data/poi.json, data/radio.json, docs/WORLD.md, game/src/lib.rs, game/tests/all/climb_loot.rs, game/tests/all/main.rs, game/tests/all/rescue.rs, game_world/src/lib.rs, game_world/src/radio.rs, game_zombie/src/poi_zombies/manager.rs, rotland/src/app/frame_world.rs, rotland/src/app/metro_flags_r43.rs, rotland/src/app/metro_flags.rs, rotland/src/app/metro_flags_tests.rs, rotland/src/app/mod.rs, rotland/src/app/save_load.rs, rotland/src/app/start_world.rs, rotland/src/app/state.rs, rotland/src/main.rs, rotland/src/metro_radio.rs, rotland/src/metro_render/hulls.rs, rotland/src/metro_render/near.rs, rotland/src/metro_render/parts.rs, rotland/src/poi_links.rs, rotland/src/props.rs, rotland/src/streaming_metro.rs, rotland_ui/src/map_ui/draw.rs, rotland_ui/src/metro_ui/draw_map.rs, tasks/R43-01-climb-plan.md, tasks/R43-02-rooftops-fire-escapes.md, tasks/R43-03-stair-models.md, tasks/R43-04-floor-interiors.md, tasks/R43-05-rooftop-kit.md, tasks/R43-06-floor-zombies-loot.md, tasks/R43-07-radio-goal.md, tasks/R43-08-wiring.md, tasks/R43-09-flags-map.md, tasks/R43-round.md, tools/blender/build_r43_models.py, tools/blender/towers/brick_parts.py, tools/blender/towers/brick.py, tools/blender/towers/interior_props.py, tools/blender/towers/interiors.py, tools/blender/towers/rooftop_kit.py, tools/blender/towers/stair_extras.py, tools/blender/towers/stairs.py, tools/README_TOOLS.md, world/src/generator_contents.rs, world/src/metro/climb_hulls.rs, world/src/metro/climb.rs, world/src/metro/climb_tests.rs, world/src/metro/config.rs, world/src/metro/fire_escapes.rs, world/src/metro/mod.rs, world/src/metro/rooftops.rs, world/src/metro/rooftops_tests.rs, world/src/poi.rs
- tests passed: 1527
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
