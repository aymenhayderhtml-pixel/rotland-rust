# R42-09: Wiring, flags, map labels (after 01-08)

Owns:
- `world/src/generator_contents.rs` (the place + army line calls), `world/src/metro/state.rs` (only if places need caching there)
- `rotland/src/play_metro.rs` (the near-place flag for the spawn profile, the army-line brute, herd blocked points)
- `rotland/src/app/frame_sim_zombies.rs` (POI spawns use `spawn_kind_look`)
- `rotland/src/app/metro_flags.rs` / `metro_flags_r41.rs`, or a new `metro_flags_r42.rs` for the new poses (keep `metro_flags.rs` from growing), `rotland/src/app/test_flags.rs`
- `rotland_ui/src/map_ui/` (labels only)
- `docs/WORLD.md` (a short R42 section), `tools/README_TOOLS.md`

Read: tasks/R42-common.md, the Worker notes of R42-01..08, `world/src/generator_contents.rs`, `rotland/src/play_metro.rs`, `rotland/src/app/metro_flags*.rs`, `rotland/src/app/frame_sim_zombies.rs`, `rotland_ui/src/map_ui/` (how the metro name / town names are drawn).

## Wiring
1. **World:** add `places_in_chunk` and `army_line_in_chunk` to the per-chunk metro contents: POIs into the chunk POIs, props into the props, crates into `chunk_crates_from`. Props/vehicles get colliders like the other metro props (convex from the models). Tiny props cast no shadows (the R40 rule).
2. **Overgrowth/ground detail** may stay off the place lots, except the fire station `yards.c` (overgrown lot). Wire whatever R42-01/05 noted.
3. **Danger:**
   - `metro_profile_at` passes `near_place` = within `placeDanger.radius` of any place POI or the army line.
   - When the army line POI wakes, spawn one `brute` (with `bruteAtArmyLine`) once per day.
   - The army line POI's spawn points come from `army_line.json` (R42-02 put them there, or use the tent positions; state which).
4. **Zombie looks:** POI spawns call `spawn_kind_look(kind, pos, spawn.model)`.
5. **Herds:** add `army_line_blocked_points` to the herd `blocked` list.
6. **Map:** the big map shows "Hospital", "Police", "Fire Station" and "Army Line" labels at the places, small, in the existing label style; the minimap needs no labels.

## Flags
Each is a pose in the new file, a line in `tools/README_TOOLS.md`, and a test like the existing pose tests:
- `--metro=hospital`: 35 m in front of the ER canopy looking at it. `--metro=hospital_in`: inside the reception hall looking across to the ward.
- `--metro=police` / `police_in`: the front steps / the open office toward the cells.
- `--metro=fire` / `fire_in`: the forecourt facing the bays / inside the bay hall beside the fire truck.
- `--metro=armyline`: on the avenue 30 m outside the barrier looking at the breach, tank and helicopter beyond. `--metro=armyline_top`: 60 m up, pitched down over the whole line.

## Done when
Wired; the gate passes; the R42-round checks done. Fill `## Worker notes`.

## Worker notes
