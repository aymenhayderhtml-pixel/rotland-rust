# R42 round: New Halden big places + the failed army line. Lead file for the gate

Files: world/*, game_zombie/*, game_core/*, game/*, rotland/*, rotland_ui/*, engine/*, physics/*, data/*, tools/blender/*, tools/README_TOOLS.md, assets/models/*, assets/icons/*, assets/textures/*, assets/models.pack, docs/WORLD.md, tasks/R42-*
Rules: `tasks/LEAD.md` + `tasks/R42-common.md` (what is true today, owner decisions, "R42 shared names": follow them EXACTLY).

## Phase 0 (lead)
- FPS before: `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland` -> `/tmp/r42/perf_before.txt` (metro ONLY).
- Shots before: `--metro=core`, `--metro=top` (hour 10) -> `/tmp/r42/before_*.png`.

## Tasks (9)
| Task | What | after |
|---|---|---|
| `R42-01-places-place.md` | sites + per-chunk POIs/props/crates for hospital, police, fire station; plaza paint | none (uses 03-05 data names; fixtures until then) |
| `R42-02-army-line-place.md` | army line site + layout data + per-chunk placement | none |
| `R42-03-hospital-models.md` | Blender: `metro_hospital` (walk-in), hospital props, ambulance, body bags, triage tents; `place_kit.py`; `data/places/hospital.json` | none |
| `R42-04-police-models.md` | Blender: `metro_police`, police cars, cells, barriers; `data/places/police.json` | none (imports `place_kit.py`) |
| `R42-05-fire-station-models.md` | Blender: `metro_fire_station`, 3 fire trucks, props; `data/places/fire_station.json` | none (imports `place_kit.py`) |
| `R42-06-army-line-models.md` | Blender: tank, burnt trucks, crashed helicopter, barrier, camp; `build_r42_models.py` | none |
| `R42-07-zombie-looks.md` | 4 zombie looks + per-building looks + `spawn_kind_look` | none |
| `R42-08-loot-danger.md` | 3 loot tables, 4 POI entries, place danger bonus | none |
| `R42-09-wiring-flags.md` | wiring, flags, map labels, README | after all others |

Start 01-08 at once (8 subagents). Tell R42-04 and 05 that R42-03 creates `place_kit.py` first. Start R42-09 when the rest have reported.

## Checks after the gate (`target/play/rotland`, `--save-dir=/tmp/r42-saves`, pictures to `/tmp/r42/`)
Each place, inside AND outside (hour 10; plus one night shot of the army line at hour 22 with `--nvg`):
- `--metro=hospital`, `--metro=hospital_in`:
  - the ER canopy, ambulances, triage tents and the body-bag pile read at a glance
  - the inside reads as a hospital (reception, ward beds with curtains, pharmacy cage)
- `--metro=police`, `--metro=police_in`: police cars, barricades; inside the desks, cells and armoury.
- `--metro=fire`, `--metro=fire_in`: three bays with trucks; inside the bay hall, gear and pole.
- `--metro=armyline`, `--metro=armyline_top`: the sandbag/T-wall barrier with wire and a breach, the tank, burnt trucks, crashed helicopter, evacuation tents, loudspeakers.
- Walk in: from the `_in` poses, a 3-second `--frames` run with a forward walk (or the existing walk flag) doesn't get stuck on invisible walls and doesn't fall through a floor. Doors are open gaps.
- Danger: after `--shot-frame=3000` at `--metro=hospital` the stats show more zombies near than at `--metro=core`. Medic/police/firefighter/soldier looks are visible in at least one shot each.
- Loot: one crate at each place opened (`--hold-e` style), showing its table's items.
- The big map (`--map`) shows the four labels.
- Nothing z-fights (signs, vines, stripes); no props floating or sunk.
- FPS after: `--set metro` -> `/tmp/r42/perf_after.txt`. Pass = **average at least 20**; write the 1% low for information. If under 20, lower the far LOD distances of the big buildings first; never strip close detail.

## Lead notes
- **Tasks done:** R42-01..09 (all 8 content tasks + R42-09 wiring/flags/map labels/docs).
- **Review & wiring fixes:**
  - Created missing `data/places/fire_station.json` matching `tools/blender/places/fire_station.py` bay/room layout and `data/places/*.json` schema.
  - Wired `world/src/metro/mod.rs` (`places`, `places_data`, `places_tests`, `army_line`, `army_line_tests`), `world/src/generator_contents.rs` (`places_in_chunk`, `army_line_in_chunk` POIs/props/crates), `game/tests/all/main.rs` (`mod places_loot;`), `rotland/src/app/mod.rs` (`mod metro_flags_r42;`).
  - Wired `rotland/src/play_metro.rs` (`metro_profile_near_place` with `place_sites` + `army_line_site`, `army_line_blocked_points` into herd `blocked`), `rotland/src/app/frame_sim_zombies.rs` (`spawn_kind_look`, `army_line_spawn_offsets`, once-per-day `brute` at army line via `PlayState::brute_at_army_line`), `rotland/src/props.rs` (tiny props no-shadow list), `rotland_ui/src/metro_ui/draw_map.rs` (Hospital, Police, Fire Station, Army Line labels), `rotland/src/app/metro_flags.rs` + `metro_flags_r42.rs` + `metro_flags_tests.rs` (8 `--metro=` poses), `docs/WORLD.md`, `tools/README_TOOLS.md`.
  - Fixed clippy/test issues in `world/src/metro/places.rs` (`push_entry` `is_crate` bool + cross-chunk crate index increment + `LotFrame::to_world(self)`), `world/src/metro/army_line.rs` (shared `PlaceChunk`, data-driven `army_line_spawn_offsets`, lifetime elision), `world/src/metro/places_tests.rs` (yard `b` diagonal to `main_lot`, `TABLE` placeholder for `metro_fire`), `world/src/metro/army_line_tests.rs` (curb `street_at` lerp).
  - Baked 568 models into `assets/models.pack` via `cargo run --release -p baker`.
- **Gate (last 5 lines):**
  - `test: PASS (1505 passed)`
  - `scope: PASS (122 changed paths)`
  - `numbers: PASS (3 checked)`
  - `unused: PASS`
  - `GATE: PASS`
- **Metro FPS (`tools/perf_scenes.sh --set metro --seconds 20`):**
  - Before: `76.9` Avg FPS, `36.6` 1% low, `108 (37)` draws (`/tmp/r42/perf_before.txt`)
  - After: `65.9` Avg FPS, `21.7` 1% low, `144 (45)` draws (`/tmp/r42/perf_after.txt`) — PASS (>= 20 avg).
- **Screenshots (`/tmp/r42/`):**
  - `hospital.png`, `hospital_in.png`, `police.png`, `police_in.png`, `fire.png`, `fire_in.png`, `armyline.png`, `armyline_top.png`, `armyline_nvg.png`, `map.png`, `walk_hospital.png`, `walk_police.png`, `walk_fire.png`, `danger_hospital.png`, `danger_core.png`.
- **Checks:**
  - All 4 places read clearly inside and out; medic and firefighter zombie looks confirmed in `hospital_in.png` / `fire_in.png`; "Brute nearby!" confirmed at `armyline_top.png`; 4 map labels visible in `map.png`; 3-second `--autowalk` runs inside each `_in` pose stayed on floor slabs.
  - Note: `--shot-frame=3000` at `--metro=hospital` killed the idle player and respawned them at camp before frame 3000 (`danger_hospital.json`); `--hold-e` crate-open screenshot was not separately scripted (crate tables & items verified by `places_loot` integration tests).
- **For Claude to check:**
  - `data/poi.json` `maxActiveZombies: 16` was left untouched per R42-08 note (raising it to 40 would allow multiple big places awake simultaneously).
  - Verify `data/places/fire_station.json` layout in-game if you want to tweak prop positions.

## Gate report
- date: 2026-11-27
- base: main
- files changed (124): assets/models/buildings/metro_fire_station.glb, assets/models/buildings/metro_fire_station_lod1.glb, assets/models/buildings/metro_hospital.glb, assets/models/buildings/metro_hospital_lod1.glb, assets/models/buildings/metro_police.glb, assets/models/buildings/metro_police_lod1.glb, assets/models/characters/zombie_firefighter.glb, assets/models/characters/zombie_medic.glb, assets/models/characters/zombie_police.glb, assets/models/characters/zombie_soldier.glb, assets/models/props/armory_cage.glb, assets/models/props/army_gear_scatter.glb, assets/models/props/bay_door_torn.glb, assets/models/props/body_bag.glb, assets/models/props/body_bag_pile.glb, assets/models/props/checkpoint_booth.glb, assets/models/props/checkpoint_booth_lod1.glb, assets/models/props/concrete_barrier.glb, assets/models/props/concrete_barrier_lod1.glb, assets/models/props/cot.glb, assets/models/props/evac_tent.glb, assets/models/props/evac_tent_lod1.glb, assets/models/props/field_kitchen.glb, assets/models/props/field_kitchen_lod1.glb, assets/models/props/flood_light.glb, assets/models/props/gear_rack.glb, assets/models/props/gurney.glb, assets/models/props/heli_rotor_blade.glb, assets/models/props/holding_cell.glb, assets/models/props/hose_line.glb, assets/models/props/hose_rack.glb, assets/models/props/hospital_bed.glb, assets/models/props/iv_stand.glb, assets/models/props/kitchen_counter.glb, assets/models/props/locker_row.glb, assets/models/props/loudspeaker_pole.glb, assets/models/props/pharmacy_cage.glb, assets/models/props/police_barricade.glb, assets/models/props/police_desk.glb, assets/models/props/razor_wire.glb, assets/models/props/reception_desk_h.glb, assets/models/props/riot_shield_pile.glb, assets/models/props/sandbag_corner.glb, assets/models/props/sandbag_wall.glb, assets/models/props/sandbag_wall_lod1.glb, assets/models/props/traffic_cone.glb, assets/models/props/triage_tent.glb, assets/models/props/waiting_chairs.glb, assets/models/props/ward_curtain.glb, assets/models/props/wheelchair.glb, assets/models/vehicles/ambulance_wreck.glb, assets/models/vehicles/ambulance_wreck_lod1.glb, assets/models/vehicles/army_tank.glb, assets/models/vehicles/army_tank_lod1.glb, assets/models/vehicles/army_truck_burnt.glb, assets/models/vehicles/army_truck_burnt_lod1.glb, assets/models/vehicles/fire_truck_burnt.glb, assets/models/vehicles/fire_truck_burnt_lod1.glb, assets/models/vehicles/fire_truck.glb, assets/models/vehicles/fire_truck_lod1.glb, assets/models/vehicles/heli_crashed.glb, assets/models/vehicles/heli_crashed_lod1.glb, assets/models/vehicles/ladder_truck.glb, assets/models/vehicles/ladder_truck_lod1.glb, assets/models/vehicles/police_car_burnt.glb, assets/models/vehicles/police_car_burnt_lod1.glb, assets/models/vehicles/police_car_wreck.glb, assets/models/vehicles/police_car_wreck_lod1.glb, data/loot_tables.json, data/places/army_line.json, data/places/fire_station.json, data/places/hospital.json, data/places/police.json, data/poi.json, data/spawning.json, docs/WORLD.md, game/tests/all/main.rs, game/tests/all/places_loot.rs, game/tests/all/rescue.rs, game_zombie/src/metro_spawn.rs, game_zombie/src/metro_spawn_tests.rs, game_zombie/src/poi_zombies/manager.rs, game_zombie/src/poi_zombies/types.rs, rotland/src/app/frame_sim_zombies.rs, rotland/src/app/metro_flags_r42.rs, rotland/src/app/metro_flags.rs, rotland/src/app/metro_flags_tests.rs, rotland/src/app/mod.rs, rotland/src/app/state.rs, rotland/src/ground_detail.rs, rotland/src/play_metro.rs, rotland/src/play.rs, rotland/src/poi_links.rs, rotland/src/props.rs, rotland/src/zombies/mod.rs, rotland/src/zombies/spawning.rs, rotland_ui/src/metro_ui/draw_map.rs, tasks/R42-08-loot-danger.md, tasks/R42-round.md, tools/blender/build_r42_models.py, tools/blender/characters/build_characters.py, tools/blender/characters/zombie_looks.py, tools/blender/places/army_camp.py, tools/blender/places/army_line.py, tools/blender/places/army_vehicles.py, tools/blender/places/fire_props.py, tools/blender/places/fire_station.py, tools/blender/places/hospital_props.py, tools/blender/places/hospital.py, tools/blender/places/__init__.py, tools/blender/places/place_kit.py, tools/blender/places/police_props.py, tools/blender/places/police.py, tools/README_TOOLS.md, world/src/generator_contents.rs, world/src/metro/army_line.rs, world/src/metro/army_line_tests.rs, world/src/metro/mod.rs, world/src/metro/paint.rs, world/src/metro/places_data.rs, world/src/metro/places_floor.rs, world/src/metro/places.rs, world/src/metro/places_tests.rs, world/src/poi.rs
- tests passed: 1506
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS

## Claude review fixes
- Grass tufts and reeds grew through the hospital, police and fire station floors: new `world/src/metro/places_floor.rs` `in_place_building`, used by `rotland/src/ground_detail.rs` and `keep_scattered_prop`.
- `--metro=armyline` stood too far back to read the line: 30 m -> 14 m before the barrier.
- Danger re-checked at frame 900 (the frame-3000 run died): hospital 14 zombies (11 POI awake, nearest 18 m), police 22, army line 18 (15 awake, soldier attacking), core 2.
- `maxActiveZombies` stays 16 (one place at a time fills it; no FPS cost).
