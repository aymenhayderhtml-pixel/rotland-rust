# R40 round: New Halden, nature takes it back + survivor traces. Lead file for the gate

Files: world/*, rotland/*, rotland_ui/*, engine/*, physics/*, data/*, tools/blender/*, tools/perf_scenes.sh, tools/README_TOOLS.md, assets/models/*, assets/textures/*, assets/models.pack, docs/WORLD.md, tasks/R40-*
Rules: `tasks/LEAD.md` + `tasks/R40-common.md` (owner decisions + "R40 shared names": follow them EXACTLY).

## Phase 0 (lead)
- FPS before: `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland` -> `/tmp/r40/perf_before.txt` (metro ONLY; do not run `--set all`).
- Shots before: `--metro=core`, `--metro=edge`, `--metro=lobby`, `--metro=top` (hour 10) -> `/tmp/r40/before_*.png`.

## Tasks (8)
| Task | What | after |
|---|---|---|
| `R40-01-overgrowth-place.md` | overgrowth placement per chunk (streets, lots, walls, roofs), density per district | none |
| `R40-02-overgrowth-models.md` | Blender: vines, crack weeds, asphalt trees, shrubs, moss, roof bushes/trees, leaves | none |
| `R40-03-damage-place.md` | fallen / leaning tower / burnt block sites, blocked street, skip replaced towers | none |
| `R40-04-damage-models.md` | Blender: fallen section, rubble fields, leaning tower, burnt overlays, burnt car pile, ash | none |
| `R40-05-traces-models.md` | Blender: car walls, graffiti, posters, rooftop camp, boarded doors, sandbags, body pose, lobby dressing | none |
| `R40-06-traces-place.md` | car walls, graffiti, posters, camps, bodies, lobby dressing placement | after 03 + 07 (uses their names) |
| `R40-07-tower-variety.md` | `_floor_c` damaged floors for all styles/sizes + better crowns/roof props | none |
| `R40-08-wiring-flags.md` | chunk props + colliders + shadow rules, `--metro=fallen|leaning|burnt|carwall|camp`, landmark view fix, README | after 01-07 |

## Checks after the gate (target/play/rotland, `--save-dir=/tmp/r40-saves`, pictures to /tmp/r40/)
- `--metro=core`, `--metro=edge` at hour 10 and 18: it looks like nature is taking the city back (vines on walls, weeds in cracks, trees in the street, wild lots), still clearly a big city. The ground outside the city is still green.
- `--metro=fallen`, `--metro=leaning`, `--metro=burnt`: each reads at a glance, and you can climb the rubble.
- `--metro=carwall`: a stacked-car wall across a street with a way over. `--metro=camp`: a rooftop camp (camera high).
- `--metro=lobby`: a dressed lobby (columns, ceiling, debris, posters), not a bare hall.
- `--metro=landmark`: looking UP at the tallest landmark crown.
- `--metro=top`: roofs show water tanks and machinery, plus some roof greenery.
- No z-fighting flicker on vines, graffiti or posters (look closely).
- FPS after: `--set metro` -> `/tmp/r40/perf_after.txt`. Pass = **average at least 20**. Write the 1% low too, for information. If the average is under 20, lower the overgrowth density in data first; never strip close detail.

## Lead notes
- **Tasks done:** R40-01 (`overgrowth.rs`, `overgrowth_roofs.rs`, `overgrowth_tests.rs`), R40-02 (13 nature GLBs), R40-03 (`damage.rs`, `damage_tests.rs`), R40-04 (damage GLBs), R40-05 (18 trace/lobby GLBs), R40-06 (`traces.rs`, `traces_camps.rs`, `traces_tests.rs`), R40-07 (20 `_floor_c` + 10 rebuilt crowns + 4 `roof_*` GLBs + `tower_parts.rs` `DamageMode` & `tower_parts_tests.rs`), R40-08 (wiring in `generator_contents.rs`, `parts.rs`, `hulls.rs`, `props.rs`, `streaming.rs`, `metro_flags.rs`, `test_flags.rs`, `docs/WORLD.md`, `tools/README_TOOLS.md`).
- **Review fixes:**
  - Merged `overgrowth`, `damage`, `traces`, and `towers.damagedShare` into `data/metro.json` while preserving `[16, 24]` compact formatting for `towers_tests`.
  - Wired `world/src/metro/mod.rs`, `world/src/lib.rs`, `world/src/generator_metro.rs`, and `tools/blender/build_r40_models.py` (all 4 Blender groups -> `R40 BUILD OK`).
  - Fixed `tw_brick_16_floor_c_lod1` tri budget (312 -> <= 300) in `tools/blender/towers/brick.py`.
  - Fixed entrance-avenue corner overlap for `tree_asphalt`/`weeds_crack`/`moss_patch` in `overgrowth_roofs.rs`, sidewalk-corner overlap for bodies in `traces_camps.rs`, rotated city corner chunk range in `traces_tests.rs`, `parts.rs` mast test `no_damage`, and number-source tags in `damage.rs`/`overgrowth_roofs.rs`.
  - Fixed `--metro=camp` height in `metro_flags.rs` (`+3.65 m`, `-30°` high camera) so the camera sits above the crown hull.
- **Last 5 gate lines:**
  - `test: PASS (1415 passed)`
  - `scope: PASS (143 changed paths)`
  - `numbers: PASS (26 checked)`
  - `unused: PASS`
  - `GATE: PASS`
- **Pictures (`/tmp/r40/`):** `before_{core,edge,lobby,top}.png`, `after_{core,edge}_h10.png`, `after_{core,edge}_h18.png`, `after_{lobby,landmark,fallen,leaning,burnt,carwall,camp,top}_h10.png`.
- **Metro FPS (`tools/perf_scenes.sh --set metro --seconds 20`):**
  - Before: **55.1 Avg FPS**, 1% low 23.8 FPS (96 draws, 39 shadow)
  - After: **56.8 Avg FPS**, 1% low 29.0 FPS (137 draws, 44 shadow) — PASS (>= 20 Avg FPS).
- **For Claude to check:**
  - `rooftop_camp` in `traces_camps.rs` is placed at `last_floor.pos.y + fh` (`crown_y`), whereas the solid crown roof top is `crown_y + style.crown_height` (3.5 m on Brick); check if `rooftop_camp` (and `overgrowth_roofs` roof bushes/trees) should sit on `crown_y + crown_height`.
  - `world/tests/golden.rs` was not edited and passes.

## Gate report
- date: 2026-11-26
- base: main
- files changed (143): assets/models/buildings/burnt_floor_overlay.glb, assets/models/buildings/roof_antenna.glb, assets/models/buildings/roof_machinery.glb, assets/models/buildings/roof_stair_hut.glb, assets/models/buildings/roof_water_tank_ny.glb, assets/models/buildings/tw_brick_16_crown.glb, assets/models/buildings/tw_brick_16_crown_lod1.glb, assets/models/buildings/tw_brick_16_floor_c.glb, assets/models/buildings/tw_brick_16_floor_c_lod1.glb, assets/models/buildings/tw_brick_24_crown.glb, assets/models/buildings/tw_brick_24_crown_lod1.glb, assets/models/buildings/tw_brick_24_floor_c.glb, assets/models/buildings/tw_brick_24_floor_c_lod1.glb, assets/models/buildings/tw_deco_24_crown.glb, assets/models/buildings/tw_deco_24_crown_lod1.glb, assets/models/buildings/tw_deco_24_floor_c.glb, assets/models/buildings/tw_deco_24_floor_c_lod1.glb, assets/models/buildings/tw_deco_32_crown.glb, assets/models/buildings/tw_deco_32_crown_lod1.glb, assets/models/buildings/tw_deco_32_floor_c.glb, assets/models/buildings/tw_deco_32_floor_c_lod1.glb, assets/models/buildings/tw_fallen_section.glb, assets/models/buildings/tw_fallen_section_lod1.glb, assets/models/buildings/tw_glass_24_crown.glb, assets/models/buildings/tw_glass_24_crown_lod1.glb, assets/models/buildings/tw_glass_24_floor_c.glb, assets/models/buildings/tw_glass_24_floor_c_lod1.glb, assets/models/buildings/tw_glass_32_crown.glb, assets/models/buildings/tw_glass_32_crown_lod1.glb, assets/models/buildings/tw_glass_32_floor_c.glb, assets/models/buildings/tw_glass_32_floor_c_lod1.glb, assets/models/buildings/tw_leaning_tower.glb, assets/models/buildings/tw_leaning_tower_lod1.glb, assets/models/buildings/tw_slab_24_crown.glb, assets/models/buildings/tw_slab_24_crown_lod1.glb, assets/models/buildings/tw_slab_24_floor_c.glb, assets/models/buildings/tw_slab_24_floor_c_lod1.glb, assets/models/buildings/tw_slab_32_crown.glb, assets/models/buildings/tw_slab_32_crown_lod1.glb, assets/models/buildings/tw_slab_32_floor_c.glb, assets/models/buildings/tw_slab_32_floor_c_lod1.glb, assets/models/buildings/tw_stone_24_crown.glb, assets/models/buildings/tw_stone_24_crown_lod1.glb, assets/models/buildings/tw_stone_24_floor_c.glb, assets/models/buildings/tw_stone_24_floor_c_lod1.glb, assets/models/buildings/tw_stone_32_crown.glb, assets/models/buildings/tw_stone_32_crown_lod1.glb, assets/models/buildings/tw_stone_32_floor_c.glb, assets/models/buildings/tw_stone_32_floor_c_lod1.glb, assets/models/nature/grass_tall_patch.glb, assets/models/nature/leaves_litter.glb, assets/models/nature/moss_patch.glb, assets/models/nature/roof_bush.glb, assets/models/nature/roof_tree.glb, assets/models/nature/sapling_city.glb, assets/models/nature/shrub_wild.glb, assets/models/nature/tree_asphalt.glb, assets/models/nature/vine_cornice.glb, assets/models/nature/vine_wall_l.glb, assets/models/nature/vine_wall_m.glb, assets/models/nature/vine_wall_s.glb, assets/models/nature/weeds_crack.glb, assets/models/props/ash_patch.glb, assets/models/props/boarded_door.glb, assets/models/props/body_sheet_b.glb, assets/models/props/car_burnt_pile.glb, assets/models/props/car_wall_stack.glb, assets/models/props/graffiti_arrow.glb, assets/models/props/graffiti_dead_inside.glb, assets/models/props/graffiti_no_food.glb, assets/models/props/graffiti_turn_back.glb, assets/models/props/graffiti_went_north.glb, assets/models/props/graffiti_x.glb, assets/models/props/lobby_ceiling_hang.glb, assets/models/props/lobby_chair_tipped.glb, assets/models/props/lobby_column.glb, assets/models/props/lobby_debris.glb, assets/models/props/lobby_light_broken.glb, assets/models/props/lobby_rug.glb, assets/models/props/poster_wall.glb, assets/models/props/rooftop_camp.glb, assets/models/props/rubble_field_l.glb, assets/models/props/rubble_field_m.glb, assets/models/props/sandbags_line.glb, data/metro.json, docs/WORLD.md, rotland/src/app/metro_flags.rs, rotland/src/app/test_flags.rs, rotland/src/metro_render/hulls.rs, rotland/src/metro_render/near.rs, rotland/src/metro_render/parts.rs, rotland/src/metro_render/tests.rs, rotland/src/props.rs, rotland/src/streaming.rs, tasks/R40-01-overgrowth-place.md, tasks/R40-02-overgrowth-models.md, tasks/R40-05-traces-models.md, tasks/R40-06-traces-place.md, tasks/R40-round.md, tools/blender/build_r40_models.py, tools/blender/damage/burnt_overlay.py, tools/blender/damage/car_burnt_pile.py, tools/blender/damage/damage.py, tools/blender/damage/fallen_section.py, tools/blender/damage/leaning_tower.py, tools/blender/damage/rubble.py, tools/blender/overgrowth/ground_weeds.py, tools/blender/overgrowth/og_util.py, tools/blender/overgrowth/overgrowth.py, tools/blender/overgrowth/trees_shrubs.py, tools/blender/overgrowth/vines.py, tools/blender/towers/brick.py, tools/blender/towers/damaged_parts.py, tools/blender/towers/deco.py, tools/blender/towers/glass.py, tools/blender/towers/roof_parts.py, tools/blender/towers/roof_props.py, tools/blender/towers/slab.py, tools/blender/towers/stone.py, tools/blender/towers/towers_r40.py, tools/blender/traces/camp.py, tools/blender/traces/car_wall.py, tools/blender/traces/decals.py, tools/blender/traces/graffiti_font.py, tools/blender/traces/lobby_dressing.py, tools/blender/traces/traces.py, tools/README_TOOLS.md, world/src/generator_contents.rs, world/src/generator_metro.rs, world/src/lib.rs, world/src/metro/config.rs, world/src/metro/damage.rs, world/src/metro/damage_tests.rs, world/src/metro/mod.rs, world/src/metro/overgrowth_roofs.rs, world/src/metro/overgrowth.rs, world/src/metro/overgrowth_tests.rs, world/src/metro/state.rs, world/src/metro/tower_parts.rs, world/src/metro/tower_parts_tests.rs, world/src/metro/traces_camps.rs, world/src/metro/traces.rs, world/src/metro/traces_tests.rs
- tests passed: 1415
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
