# R41 round: New Halden, zombies, loot, silent life. Lead file for the gate

Files: world/*, game_zombie/*, game_core/*, game_player/*, game_world/*, game/*, rotland/*, rotland_ui/*, engine/*, physics/*, save/*, data/*, tools/blender/*, tools/perf_scenes.sh, tools/README_TOOLS.md, assets/models/*, assets/icons/*, assets/textures/*, assets/models.pack, docs/WORLD.md, tasks/R41-*
Rules: `tasks/LEAD.md` + `tasks/R41-common.md` (owner decisions + "R41 shared names": follow them EXACTLY).

## Phase 0 (lead)
- FPS before: `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland` -> `/tmp/r41/perf_before.txt` (metro ONLY; do not run `--set all`).
- Shots before, at hour 10, saved to `/tmp/r41/before_*.png`:
  - `--metro=core`
  - `--metro=edge`
  - `--metro=core --hour=23`

## Tasks (10)
| Task | What | after |
|---|---|---|
| `R41-01-metro-zombies.md` | metro spawn profile: density/cap by district, runners at night | none |
| `R41-02-street-herds.md` | herd routes along avenues (world) + herd runtime (game_zombie) | none |
| `R41-03-life-place.md` | power lines, billboards, flags, plastic, crow perches placement | none |
| `R41-04-life-models.md` | Blender: poles, wires, billboards + faces, flag/plastic frames, crows; `build_r41_models.py` | none |
| `R41-05-crows-flappers.md` | crow flocks + flapping frames (pure) and their drawing (`metro_life.rs`) | none (uses 03/04 names from common) |
| `R41-06-city-items.md` | 12 city items, army rifle, armour, boosts, night vision, parts repair | none |
| `R41-07-item-models.md` | Blender: item models, army rifle, crates, icons | none |
| `R41-08-district-loot.md` | metro loot spots + 7 district tables, table/model override on `LootSpot` | none (uses 06/07 names from common) |
| `R41-10-splits-fallen-climb.md` | split traces/damage/streaming, fallen tower climbable | none |
| `R41-09-wiring-flags.md` | wiring, night-vision look, HUD boosts, saves, flags, README | after all others |

Start 01-08 and 10 at once (9 subagents). R41-10 only MOVES code in traces/damage/streaming; tell 03, 08 and 09 to call those modules by their public names. Start R41-09 when the rest have reported.

## Checks after the gate (`target/play/rotland`, `--save-dir=/tmp/r41-saves`, pictures to `/tmp/r41/`)
- `--metro=herd`: a herd of 12-30 walkers coming slowly down an avenue. It reads as a herd, not a scattered crowd. Take a second shot ~10 s later (`--shot-frame`) to show they moved along the street.
- `--metro=night`: more zombies than the day, mostly runners. Compare with `--metro=core` at hour 10 and with `--metro=edge` at hour 23 (Edge has fewer).
- `--metro=night --nvg`: a green night-vision view. You can see the street and the zombies, with a little noise and vignette. The HUD shows the battery.
- `--metro=crows`: crows on a body, pecking. `--metro=crows_up` (or the startle shot): crows in the air.
- `--metro=core` and `--metro=top`: power poles with sagging wires along some avenues, one or more fallen poles with wires down, billboards with faded made-up ads on roofs and lots, torn flags on lobby fronts. Two shots a few frames apart show the flags in different frames.
- `--metro=army`: an army crate with its loot. `--give=army_rifle,ammo_556` with a first-person shot of the rifle and one firing shot.
- `--metro=fallen`: walk up the fallen section (a shot from its top).
- Zombies outside the city are unchanged: `--scene=` forest shot, same zombie count look as before.
- No z-fighting on billboard faces, flags or wires (look closely).
- FPS after: `--set metro` -> `/tmp/r41/perf_after.txt`. Pass = **average at least 20**; write the 1% low too, for information. If the average is under 20, lower crow counts/draw radius and the herd count in data first; never strip close detail.

## Lead notes
- **Tasks done:**
  - `R41-01`: `game_zombie/src/metro_spawn.rs`, `metro_spawn_tests.rs`, `spawner.rs`, `spawner_config.rs`, `data/spawning.json` (`metro` block).
  - `R41-02`: `world/src/metro/herds.rs`, `herds_tests.rs`, `game_zombie/src/street_herds.rs`, `game/tests/all/street_herds.rs`, `data/metro.json` (`herds` block).
  - `R41-03`: `world/src/metro/life.rs`, `life_wires.rs`, `life_tests.rs`, `data/metro.json` (`life` block).
  - `R41-04`: `tools/blender/life/*`, `tools/blender/build_r41_models.py`, 40 `.glb`s (`power_pole*`, `power_wire_*`, `billboard_*`, `flag_*`, `plastic_flap_*`, `crow_*`) + previews.
  - `R41-05`: `game_world/src/crows/*`, `data/crows.json`, `rotland/src/metro_life.rs`.
  - `R41-06`: 12 city items in `data/items.json`, `army_rifle` in `weapons.json`/`gun_feel.json`/`weapon_mounts.json`, `data/armor.json`, `data/boosts.json`, `data/vehicles.json` (`partsRepairHp`), `game_core/src/armor.rs`, `game_player/src/survival/boosts.rs`, `game_player/src/night_vision.rs`, `wear.rs`, `vehicles/manager.rs`, `game/tests/all/city_items.rs`.
  - `R41-07`: `tools/blender/gear/{items_r41,army_gear,meds_parts,crates_r41}.py`, 16 `.glb`s (`army_rifle`, 11 items, 4 crates) + previews + 12 icons.
  - `R41-08`: `world/src/loot.rs`, `world/src/metro/loot.rs`, `loot_tests.rs`, 7 `metro_*` tables in `data/loot_tables.json`, `data/metro.json` (`loot` block), `rotland/src/crates.rs`.
  - `R41-10`: split `traces.rs` (`traces_config.rs`, `traces_walls.rs`), `damage.rs` (`damage_props.rs`), `streaming.rs` (`streaming_metro.rs`) all <= 273 lines; added `rubble_ramp` + `tw_fallen_section` top-deck/gap-step colliders + climb test in `damage_tests.rs`.
  - `R41-09`: `rotland/src/play_metro.rs`, `play.rs` one-line hooks, `combat_damage.rs`, `frame_*.rs`, `metro_flags.rs`, `test_flags.rs`, `engine/src/render/{lit.rs,lit.wgsl}`, `rotland/src/light.rs`, `rotland_ui/src/hud/boosts.rs`, `save_load.rs`/`start_world.rs`, `docs/WORLD.md`, `tools/README_TOOLS.md`.
- **Review fixes:**
  - Merged `herds`, `life`, and `loot` into `data/metro.json` and `world/src/metro/config.rs`, plus all `mod`/`pub use` shared-file lines across `world`, `game_*`, `game`, `rotland_ui`, and `rotland`.
  - Updated `data/weapon_mounts.json` `army_rifle` coordinates to match R41-07's Blender empties; added `army_rifle` to `data/weapon_wear.json`, `data/sounds.json` (`rifle_shot`), and `data/stances.json` (`rifle`).
  - Updated `rotland/src/streaming_metro.rs` to use `crate::crates::crate_model(c)` so `Some("")` draws no extra crate model.
  - Fixed clippy/test issues (`gen` reserved keyword in `metro_life.rs`, `MetroDistrict` re-export, `world/src/metro/` number tags, `new_weapons.rs` allowing `grenade` in `metro_army`, `loot_tests.rs` `"forest"` enum casing, `damage_tests.rs` sloped `top_deck` overlap Z interpolation).
  - Ran Blender (`build_r41_models.py`, `rubble.py`, `fallen_section.py`) under `flock /tmp/rotland-blender.lock` and baked 500 models with `cargo run --release -p baker`.
- **Last 5 gate lines:**
  - `test: PASS (1458 passed)`
  - `scope: PASS (185 changed paths)`
  - `numbers: PASS (105 checked)`
  - `unused: PASS`
  - `GATE: PASS`
- **Pictures (`/tmp/r41/`):** `before_{core,edge,core_night}.png`, `after_{herd_1,herd_2,night_core,night_core_360,core,edge_night,nvg,crows,crows_up,top,flag_f0,flag_f1,flag_crows_f1,army,rifle,fallen,forest}.png`.
- **Metro FPS (`tools/perf_scenes.sh --set metro --seconds 20`):**
  - Before: **61.3 Avg FPS**, **35.3 1% low**, `113 (37)` draws (`/tmp/r41/perf_before.txt`).
  - After: **49.7 Avg FPS**, **21.6 1% low**, `142 (45)` draws (`/tmp/r41/perf_after.txt`) — **PASS (>= 20 Avg FPS)**.
- **For Claude to check:**
  - `game/tests/all/new_weapons.rs` was updated to allow `grenade` in `metro_army` (per `R41-08-district-loot.md`).
  - `world/tests/golden.rs` was not edited and passes.
  - `tools/blender/build_r41_models.py` skipped `items_r41` when invoked from `build_r41_models.py` (`sys.path`); `items_r41.py` ran standalone and all 16 item/crate models + 12 icons are baked in `assets/models.pack`.

- **Claude review fixes (after the lead):**
  - Zombies spawned on tower roofs in New Halden (the physics ray from the sky hits the roof first), so few reached the player: metro spawns now skip tower footprints and anything over 1.5 m above the ground (`play_metro::metro_spawn_ground`).
  - Street herds aggroed at once from ~160 m (horde followers sit in the chase state, and `spawn_tick` retargeted every horde to the player) and then stood stuck behind towers: herds are now steered only by `play_metro`, and a member counts as noticing only within 35 m. Herds now walk their avenues and charge when close.
  - Crows: no perches on bodies inside lobbies; `--metro=crows` camera at 14 m (outside the 12 m startle radius), `crows_up` at 4 m. Both checked: perched crows by a body, then crows in the air.
  - `army_rifle` added to every attachment `fits` list that has `ak47`; night-vision noise 0.06 -> 0.035.
  - Splits (Haiku): life.rs, herds.rs, crows/flock.rs, metro/loot.rs, metro_flags.rs (now 417, tests in metro_flags_tests.rs).
  - Gate PASS (1458 tests). Owner FPS run (metro, everything else closed): 81.8 avg, 1% low 37.9.
  - Leftovers: play.rs 884 / frame_input.rs ~444 / metro_flags.rs 417 lines (new hand-item arms belong in their own file); herd spawns as a tight blob; daytime zombies only notice you within ~18 m (sight), so a still player in the day sees few attackers.

## Gate report
- date: 2026-11-27
- base: main
- files changed (198): assets/icons/adrenaline_shot.png, assets/icons/ammo_556.png, assets/icons/antibiotics.png, assets/icons/army_helmet.png, assets/icons/army_ration.png, assets/icons/army_rifle.png, assets/icons/battery.png, assets/icons/body_armor.png, assets/icons/car_parts.png, assets/icons/gun_parts.png, assets/icons/night_vision.png, assets/icons/surgery_kit.png, assets/models/animals/crow_fly_f0.glb, assets/models/animals/crow_fly_f1.glb, assets/models/animals/crow_fly_f2.glb, assets/models/animals/crow_fly_f3.glb, assets/models/animals/crow_peck.glb, assets/models/animals/crow_perched.glb, assets/models/buildings/tw_fallen_section.glb, assets/models/gear/adrenaline_shot.glb, assets/models/gear/ammo_556.glb, assets/models/gear/antibiotics.glb, assets/models/gear/army_helmet.glb, assets/models/gear/army_ration.glb, assets/models/gear/battery.glb, assets/models/gear/body_armor.glb, assets/models/gear/car_parts.glb, assets/models/gear/gun_parts.glb, assets/models/gear/night_vision.glb, assets/models/gear/surgery_kit.glb, assets/models/props/army_crate.glb, assets/models/props/billboard_face_a.glb, assets/models/props/billboard_face_a_lod1.glb, assets/models/props/billboard_face_b.glb, assets/models/props/billboard_face_b_lod1.glb, assets/models/props/billboard_face_c.glb, assets/models/props/billboard_face_c_lod1.glb, assets/models/props/billboard_face_d.glb, assets/models/props/billboard_face_d_lod1.glb, assets/models/props/billboard_roof_a.glb, assets/models/props/billboard_roof_a_lod1.glb, assets/models/props/billboard_roof_b.glb, assets/models/props/billboard_roof_b_lod1.glb, assets/models/props/billboard_street.glb, assets/models/props/billboard_street_lod1.glb, assets/models/props/camp_stash.glb, assets/models/props/flag_pole_wall.glb, assets/models/props/flag_pole_wall_lod1.glb, assets/models/props/flag_torn_f0.glb, assets/models/props/flag_torn_f1.glb, assets/models/props/flag_torn_f2.glb, assets/models/props/flag_torn_f3.glb, assets/models/props/med_cabinet.glb, assets/models/props/metro_locker.glb, assets/models/props/plastic_flap_f0.glb, assets/models/props/plastic_flap_f1.glb, assets/models/props/plastic_flap_f2.glb, assets/models/props/plastic_flap_f3.glb, assets/models/props/power_pole_fallen.glb, assets/models/props/power_pole_fallen_lod1.glb, assets/models/props/power_pole.glb, assets/models/props/power_pole_leaning.glb, assets/models/props/power_pole_leaning_lod1.glb, assets/models/props/power_pole_lod1.glb, assets/models/props/power_wire_down.glb, assets/models/props/power_wire_down_lod1.glb, assets/models/props/power_wire_span.glb, assets/models/props/power_wire_span_lod1.glb, assets/models/props/rubble_ramp.glb, assets/models/weapons/army_rifle.glb, data/armor.json, data/attachments.json, data/boosts.json, data/crows.json, data/gun_feel.json, data/items.json, data/loot_tables.json, data/metro.json, data/sounds.json, data/spawning.json, data/stances.json, data/vehicles.json, data/weapon_mounts.json, data/weapons.json, data/weapon_wear.json, docs/WORLD.md, engine/src/render/lit.rs, engine/src/render/lit.wgsl, game_core/src/armor.rs, game_core/src/lib.rs, game_core/src/weapons/mod.rs, game_core/src/weapons/wear.rs, game_player/src/lib.rs, game_player/src/night_vision.rs, game_player/src/survival/boosts.rs, game_player/src/survival/mod.rs, game_player/src/vehicles/manager.rs, game/src/lib.rs, game/tests/all/biome_zombies_tests.rs, game/tests/all/city_items.rs, game/tests/all/main.rs, game/tests/all/new_weapons.rs, game/tests/all/street_herds.rs, game_world/src/crows/circlers.rs, game_world/src/crows/flock.rs, game_world/src/crows/mod.rs, game_world/src/crows/tests.rs, game_world/src/lib.rs, game_zombie/src/lib.rs, game_zombie/src/metro_spawn.rs, game_zombie/src/metro_spawn_tests.rs, game_zombie/src/spawner_config.rs, game_zombie/src/spawner.rs, game_zombie/src/street_herds.rs, game_zombie/src/zombie/brain.rs, rotland/src/app/frame_actors_move.rs, rotland/src/app/frame_input.rs, rotland/src/app/frame_sim_player.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_ui.rs, rotland/src/app/frame_world.rs, rotland/src/app/metro_flags_r41.rs, rotland/src/app/metro_flags.rs, rotland/src/app/metro_flags_tests.rs, rotland/src/app/mod.rs, rotland/src/app/render_frame.rs, rotland/src/app/save_load.rs, rotland/src/app/start_world.rs, rotland/src/app/state.rs, rotland/src/app/test_flags.rs, rotland/src/crates.rs, rotland/src/light.rs, rotland/src/main.rs, rotland/src/metro_life.rs, rotland/src/play_metro.rs, rotland/src/play.rs, rotland/src/session.rs, rotland/src/streaming_metro.rs, rotland/src/streaming.rs, rotland/src/zombies/mod.rs, rotland/src/zombies/spawning.rs, rotland/src/zombies/types.rs, rotland_ui/src/hub_ui/draw_bag_details.rs, rotland_ui/src/hud/boosts.rs, rotland_ui/src/hud/draw.rs, rotland_ui/src/hud/mod.rs, rotland_ui/src/hud/state.rs, tasks/R41-01-metro-zombies.md, tasks/R41-02-street-herds.md, tasks/R41-03-life-place.md, tasks/R41-04-life-models.md, tasks/R41-05-crows-flappers.md, tasks/R41-06-city-items.md, tasks/R41-07-item-models.md, tasks/R41-08-district-loot.md, tasks/R41-09-wiring-flags.md, tasks/R41-10-splits-fallen-climb.md, tasks/R41-round.md, tools/blender/build_r41_models.py, tools/blender/damage/fallen_section.py, tools/blender/damage/rubble.py, tools/blender/gear/army_gear.py, tools/blender/gear/crates_r41.py, tools/blender/gear/items_r41.py, tools/blender/gear/meds_parts.py, tools/blender/life/billboards.py, tools/blender/life/crows.py, tools/blender/life/flappers.py, tools/blender/life/life.py, tools/blender/life/poles.py, tools/README_TOOLS.md, world/src/generator_contents.rs, world/src/lib.rs, world/src/loot.rs, world/src/metro/config.rs, world/src/metro/damage_props.rs, world/src/metro/damage.rs, world/src/metro/damage_tests.rs, world/src/metro/herds_grid.rs, world/src/metro/herds_route.rs, world/src/metro/herds.rs, world/src/metro/herds_tests.rs, world/src/metro/life_billboards.rs, world/src/metro/life_config.rs, world/src/metro/life_flags.rs, world/src/metro/life_helpers.rs, world/src/metro/life_perches.rs, world/src/metro/life.rs, world/src/metro/life_tests.rs, world/src/metro/life_wires.rs, world/src/metro/loot_helpers.rs, world/src/metro/loot.rs, world/src/metro/loot_tests.rs, world/src/metro/mod.rs, world/src/metro/traces_config.rs, world/src/metro/traces.rs, world/src/metro/traces_tests.rs, world/src/metro/traces_walls.rs
- tests passed: 1458
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
