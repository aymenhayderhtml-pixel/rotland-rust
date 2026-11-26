# R38 round: New Halden groundwork (skyline, streets, highway jam). Lead file for the gate

Files: world/*, rotland/*, rotland_ui/*, engine/*, physics/*, save/*, game/*, data/*, tools/blender/*, tools/perf_scenes.sh, tools/README_TOOLS.md, assets/models/*, assets/models.pack, docs/BUDGETS.md, docs/WORLD.md, tasks/R38-*
Rules: `tasks/LEAD.md` + `tasks/R38-common.md` (owner decisions + "R38 shared names": follow them EXACTLY).

## Phase 0 (lead, before subagents)
1. Raise `data/budgets.json` and add the paragraph to `docs/BUDGETS.md` (numbers in R38-common "Budgets").
2. Baseline: `cargo build --profile play`, then `tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland` and save it to `/tmp/r38/perf_before.txt`.

## Tasks (8 subagents)
| Task | What | after |
|---|---|---|
| `R38-01-metro-layout.md` | `world/src/metro/`: config, placement (~6 km, swamp/snow seam), block grid, streets, districts, reserved blocks, flat ground | none |
| `R38-02-metro-towers.md` | tower spots: heights per district, 3-5 landmarks | none (use shared names) |
| `R38-03-metro-highway.md` | 4-lane highway from the nearest town, flattened ground | none (use shared names) |
| `R38-04-metro-jam.md` | evacuation jam placement (pure) + per-chunk props + colliders | none (use shared names) |
| `R38-05-jam-models.md` | Blender: jam cars (open doors, burnt), buses, army truck, luggage, bodies, gantry sign | none |
| `R38-06-metro-render.md` | street ground, near placeholder towers + colliders, far skyline batch | none (use shared names) |
| `R38-07-haze-map.md` | grey-green city haze, big map + minimap, "Entering New Halden" banner | none |
| `R38-08-metro-wiring.md` | wire it all into rotland, `--metro=` flags, perf scene `metro` | after 01-07 report |

## Checks after the gate (target/play/rotland, `--save-dir=/tmp/r38-saves`, pictures to /tmp/r38/)
- `--metro=town_view`: from the nearest town the skyline rises through the haze, landmarks clearly taller.
- `--metro=highway` at hour 10 and hour 18: the jam is dense and looks like a real evacuation (open doors, luggage, a bus on its side, bodies in some cars); skyline ahead.
- `--metro=edge`, `--metro=core`: street canyons, sidewalks, towers have colliders (walk into one: you stop), the grass outside the city stays green.
- `--metro=top`: whole city from above, avenues, ragged edge, reserved blocks empty.
- `--metro=core --hour=23`: pitch dark, no city lights.
- Each new model preview in `assets/models/_previews/` looked at.
- Ashford, camp and the 4 towns look the same as before (`--town=blackwater`, camp spawn).
- Big map shows New Halden; "Entering New Halden" banner appears.
- FPS: `tools/perf_scenes.sh --set all --seconds 20` -> `/tmp/r38/perf_after.txt`. **Every scene's 1% low must be at least 20 FPS**, including the new `metro` scene. Detail is fine, under 20 is not.
- An old save loads (make one with the before-binary in Phase 0).

## Lead notes
Lead: Claude (Opus) with Haiku coder subagents (owner's choice, 2026-11-26; DeepSeek free quota ran out).
- Tasks 01-07 done by Haiku coders; 08 split into 08a (world side) and 08b (game side). All done.
- Review fixes by the lead:
  - R38-05 first pass was ~2k-triangle boxes: sent back; now cars 8.5k-10k, buses 13-14k, truck 12.9k, props 3-9k.
  - Jam vehicles and the barrier had no colliders (walk-through): added box colliders (`add_bbox_collider` in tools/blender/metro/jam_util.py).
  - Engine: `DrawItem.fog_scale` (lit.rs + lit.wgsl). Above 1.0 means a far item: fog distance divided by it and no sun shadow. Used by the skyline so it shows kilometres away through the 190 m fog.
  - `light::lift_fog_for_height`: the camera sees farther the higher it is (rooftops later in R43); no change on the ground.
  - `--metro=` spots: highway = shoulder 350 m before the city looking at it; top = 260 m up along the central avenue; town_view = where the highway leaves the town (the forest hid the skyline from the town centre).
  - Ground detail (snow and grass tufts) is kept off the highway and the city streets.
  - The banner placeholder was replaced and dead getters removed. Number tags were added.
- Gate (last lines): clippy PASS; test PASS (1366 passed); scope PASS (104 changed paths); numbers PASS; unused PASS; GATE: PASS.
- Pictures (/tmp/r38/): town_view_10, highway_10, highway_18, edge_10, core_10, top_10, core_23 (night), camp, blackwater.
  - Skyline visible from the highway; small from the town end (about 2 km).
  - The jam is dense on the lanes leaving the city.
  - Street canyon in the core with grey-green haze.
  - Night is pitch dark with no city lights.
  - The camp grass is still green.
- FPS (perf_scenes --set all, 20 s; before in /tmp/r38/perf_before.txt): every scene's 1% low is at least 20.
  - metro (new): 150 avg / 64 low.
  - forest: 81 / 20 (was 77 / 23).
  - The first camp run read 43 / 10, but reruns gave 90 / 28 and 130 / 71 (noise).
- Known, for later rounds:
  - The sidewalk and curb paint has jagged edges (the paint follows the terrain grid).
  - Far skyline towers are flat grey (no windows) and use the normal fog colour, not the haze colour.
  - The bus on its side reads plainly.
  - No screenshot of the big map yet (no flag opens it; unit tests cover it).
  - `streaming.rs` is about 445 lines.

## Gate report
- date: 2026-11-26
- base: main
- files changed (104): assets/models/props/body_seated.glb, assets/models/props/body_sheet.glb, assets/models/props/clothes_spill.glb, assets/models/props/highway_barrier.glb, assets/models/props/highway_sign_gantry.glb, assets/models/props/luggage_pile.glb, assets/models/props/suitcase_open.glb, assets/models/vehicles/army_truck_jam.glb, assets/models/vehicles/bus_city.glb, assets/models/vehicles/bus_city_side.glb, assets/models/vehicles/car_jam_hatch_burnt.glb, assets/models/vehicles/car_jam_hatch.glb, assets/models/vehicles/car_jam_pickup_burnt.glb, assets/models/vehicles/car_jam_pickup.glb, assets/models/vehicles/car_jam_sedan_burnt.glb, assets/models/vehicles/car_jam_sedan.glb, assets/models/vehicles/car_jam_suv_burnt.glb, assets/models/vehicles/car_jam_suv.glb, assets/models/vehicles/car_jam_van_burnt.glb, assets/models/vehicles/car_jam_van.glb, data/budgets.json, data/map.json, data/metro.json, data/sky.json, docs/BUDGETS.md, engine/src/render/lit.rs, engine/src/render/lit.wgsl, rotland/examples/humanoid_view.rs, rotland/src/app/frame_ui.rs, rotland/src/app/metro_flags.rs, rotland/src/app/minimap_metro.rs, rotland/src/app/minimap.rs, rotland/src/app/mod.rs, rotland/src/app/render_frame.rs, rotland/src/app/start_world.rs, rotland/src/app/state.rs, rotland/src/app/test_flags.rs, rotland/src/ground_detail.rs, rotland/src/light.rs, rotland/src/main.rs, rotland/src/metro_haze.rs, rotland/src/metro_play.rs, rotland/src/metro_render/boxes.rs, rotland/src/metro_render/hulls.rs, rotland/src/metro_render/mod.rs, rotland/src/metro_render/near.rs, rotland/src/metro_render/skyline.rs, rotland/src/metro_render/tests.rs, rotland/src/region_banner.rs, rotland/src/streaming.rs, rotland_ui/src/lib.rs, rotland_ui/src/map_ui/draw.rs, rotland_ui/src/map_ui/state.rs, rotland_ui/src/metro_ui/draw_map.rs, rotland_ui/src/metro_ui/haze.rs, rotland_ui/src/metro_ui/minimap.rs, rotland_ui/src/metro_ui/mod.rs, rotland_ui/src/metro_ui/outline.rs, rotland_ui/src/metro_ui/tests.rs, tasks/R38-01-metro-layout.md, tasks/R38-02-metro-towers.md, tasks/R38-03-metro-highway.md, tasks/R38-04-metro-jam.md, tasks/R38-05-jam-models.md, tasks/R38-06-metro-render.md, tasks/R38-07-haze-map.md, tasks/R38-08-metro-wiring.md, tasks/R38-round.md, tools/blender/build_r38_models.py, tools/blender/metro/jam_bus.py, tools/blender/metro/jam_car_extras.py, tools/blender/metro/jam_car_interior.py, tools/blender/metro/jam_car_shell.py, tools/blender/metro/jam_cars.py, tools/blender/metro/jam_font.py, tools/blender/metro/jam_props_bags.py, tools/blender/metro/jam_props_bodies.py, tools/blender/metro/jam_props_road.py, tools/blender/metro/jam_truck.py, tools/blender/metro/jam_util.py, tools/perf_scenes.sh, tools/README_TOOLS.md, world/src/generator_contents.rs, world/src/generator_metro.rs, world/src/generator.rs, world/src/generator_world.rs, world/src/lib.rs, world/src/metro/config.rs, world/src/metro/highway_height.rs, world/src/metro/highway.rs, world/src/metro/highway_tests.rs, world/src/metro/jam_chunks.rs, world/src/metro/jam.rs, world/src/metro/jam_tests.rs, world/src/metro/jam_vehicles.rs, world/src/metro/layout.rs, world/src/metro/layout_tests.rs, world/src/metro/mod.rs, world/src/metro/paint.rs, world/src/metro/place.rs, world/src/metro/state.rs, world/src/metro/towers.rs, world/src/metro/towers_tests.rs, world/src/terrain_paint.rs
- tests passed: 1366
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
