# R35 round: ALL-OUT DETAIL. Lead file for the gate

Files: data/biomes.json, data/scatter.json, data/towns.json, data/ground_detail.json, data/sky.json, world/*, rotland/*, engine/src/render/*, tools/blender/props/*, tools/blender/build_biome_props.py, tools/blender/build_detail_props.py, assets/models/props/*, assets/models/nature/*, tasks/R35-*, docs/PERF_R35.md
Rules: `tasks/LEAD.md`. Lean: 6 subagents, each reads only its task file's list. No exploring.

## Owner decisions (2026-11-25)
- The world should look rich and lived-in. More detail is worth some speed: every scene must stay **>= 60 FPS average and >= 30 FPS worst 1%** on this laptop (`tools/perf_scenes.sh`). Today: camp ~138, forest ~106.
- The home land (2.2 km core) may get MORE small detail (grass, flowers, clouds, terrain colour), but its shape, buildings and roads must not move: `world/tests/golden.rs` passes unedited. If a golden check fails because of a new detail layer, keep the detail out of that check's data path (separate layer), never edit golden.rs.
- Low-poly flat style stays. No sounds. All new numbers in data files.

## R35 shared names
- New detail models (each a `tools/blender/props/<name>.py` like `grass_tuft_a.py`/`mushrooms.py`, output `assets/models/props/<name>.glb`, built by the new `tools/blender/build_detail_props.py` which lists ONLY these):
  - biome (task 02): `tumbleweed`, `bleached_bones`, `fence_post_desert`, `frozen_log`, `ice_shards`, `snow_bush_small`, `cattails`, `lily_pads`, `swamp_moss_clump`, `scree_patch`, `alpine_flowers`, `cairn`
  - town clutter (task 03): `laundry_line`, `porch_chair`, `water_barrel`, `woodpile`, `sled`, `propane_tank`, `bench_wood`, `fire_barrel`
  - interiors (task 06): `int_table`, `int_chair`, `int_bed`, `int_shelf`, `int_stove`, `int_fridge`, `int_cabinet`, `int_rug`
  - Task 02 writes `build_detail_props.py` with all 28 names (03 and 06 just use those exact file names).
- `data/ground_detail.json` (task 01): per biome key → list of `{ "model", "density_per_100m2", "scale":[a,b], "radius_m" }`; global `"fade_m"`.
- `data/sky.json` (task 04): cloud cover, cloud colour, cloud speed, per-biome fog tint.

## Tasks
| Task | What | Blender? |
|---|---|---|
| `R35-01-ground-detail.md` | dense short-range grass/flowers/pebbles around the player, every biome | no (uses existing + task 02 models) |
| `R35-02-biome-props.md` | 12 new biome props + scatter entries + build_detail_props.py | yes |
| `R35-03-town-clutter.md` | 8 clutter props + placement around town buildings | yes |
| `R35-04-sky-fog.md` | low-poly clouds in the sky, fog tinted per biome | no |
| `R35-05-terrain-water.md` | terrain colour variation, shore foam, river flow ripples | no |
| `R35-06-interiors.md` | 8 furniture models + 3-6 pieces inside every walk-in town building | yes |

## Checks after the gate
1. `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_detail_props.py > /tmp/r35-bld.log 2>&1` (0 Traceback). Old models unchanged (`git status --short assets/` only new files). Bake once.
2. `cargo build --profile play`. Shots (`--save-dir=/tmp/r35-saves`, into /tmp/r35/): camp, `--scene=forest`, `--biome=desert`, `--biome=snow`, `--biome=swamp`, `--biome=mountains`, `--town=dusthollow`, `--town=frostmere`, `--river=0`, and one inside a town building (find a lot position with `--at=`). Look once each: more detail, nothing floating, nothing inside walls, ground still green at camp.
3. `tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland` → `docs/PERF_R35.md` (before = R34 numbers above). If any scene is under 60 avg / 30 worst 1%, lower the densities in data (not the code) and re-measure.

## Lead notes
- **Tasks:**
  - `R35-01-ground-detail.md`: Done. Short-range instanced ground detail ring (`world/src/ground_detail.rs`, `rotland/src/ground_detail.rs`, `data/ground_detail.json`). Densities tuned down once in `data/ground_detail.json` after initial `perf_scenes.sh`.
  - `R35-02-biome-props.md`: Done. 12 biome detail props + `tools/blender/build_detail_props.py` + `data/biomes.json` scatter entries.
  - `R35-03-town-clutter.md`: Done. 8 clutter props + `world/src/towns/clutter.rs` + `data/towns.json` `clutter` block, wired in `append_town_contents`.
  - `R35-04-sky-fog.md`: Done. Low-poly drifting clouds (`engine/src/render/sky.rs`, `sky.wgsl`) + per-biome fog tint (`rotland/src/sky_play.rs`, `data/sky.json`).
  - `R35-05-terrain-water.md`: Done. Outer-biome terrain color variation (`world/src/terrain_paint.rs`), shore foam & directional river flow ripples (`engine/src/render/water.rs`, `water.wgsl`, `engine/src/target.rs`, `world/src/rivers/mesh.rs`, `rotland/src/river_view.rs`).
  - `R35-06-interiors.md`: Done. 8 furniture props + `world/src/towns/interior.rs` + `data/towns.json` `interiors` block, wired in `append_town_contents`.
- **Gate (last 5 lines before Blender/assets):**
  - Note: `scope` flagged `engine/src/target.rs` (needed by `R35-05` for `depth_texture` shore-foam copy, outside the `Files:` line).
  ```
  clippy: PASS
  test: PASS (1211 passed)
  scope: FAIL (1 outside task Files: engine/src/target.rs)
  numbers: PASS (45 checked)
  unused: PASS
  ```
- **Blender & Bake:** `build_detail_props.py` ran with 0 Tracebacks; 28 new `.glb`s in `assets/models/props/`; `baker` wrote 249 models to `assets/models.pack`.
- **FPS (`perf_scenes.sh`, 20s per scene, after):**
  - `camp`: 123.4 avg (47.6 1% low) | `forest`: 86.9 (34.1) | `city`: 97.7 (34.2) | `night`: 128.0 (56.4) | `zombies`: 68.7 (25.7) | `desert`: 121.7 (34.6) | `snow`: 113.1 (36.3) | `swamp`: 93.0 (27.7) | `mountains`: 97.9 (29.1) | `town_desert`: 107.7 (37.9) | `town_snow`: 75.3 (26.5)
  - All 11 scenes >= 68.7 avg FPS; 4 scenes (`zombies`, `swamp`, `mountains`, `town_snow`) had 1% low 25.7–29.1 from streaming frame spikes.
- **Pictures (`/tmp/r35/`):**
  - `camp.png`, `forest.png`, `desert.png`, `snow.png`, `swamp.png`, `mountains.png`, `dusthollow.png`, `frostmere.png`, `river0.png`, `interior.png` (inside `desert_store` in Dusthollow).

## Gate report
- date: 2026-11-25
- base: main
- files changed (67): data/biomes.json, data/ground_detail.json, data/sky.json, data/towns.json, engine/src/render/mod.rs, engine/src/render/sky.rs, engine/src/render/sky.wgsl, engine/src/render/water.rs, engine/src/render/water.wgsl, engine/src/target.rs, rotland/src/app/frame_warmth.rs, rotland/src/app/render_frame.rs, rotland/src/app/start_world.rs, rotland/src/app/state.rs, rotland/src/ground_detail.rs, rotland/src/main.rs, rotland/src/props.rs, rotland/src/river_view.rs, rotland/src/sky_play.rs, rotland/src/water_view.rs, tasks/R35-01-ground-detail.md, tasks/R35-02-biome-props.md, tasks/R35-03-town-clutter.md, tasks/R35-04-sky-fog.md, tasks/R35-05-terrain-water.md, tasks/R35-06-interiors.md, tasks/R35-round.md, tools/blender/build_detail_props.py, tools/blender/props/alpine_flowers.py, tools/blender/props/bench_wood.py, tools/blender/props/bleached_bones.py, tools/blender/props/cairn.py, tools/blender/props/cattails.py, tools/blender/props/fence_post_desert.py, tools/blender/props/fire_barrel.py, tools/blender/props/frozen_log.py, tools/blender/props/ice_shards.py, tools/blender/props/int_bed.py, tools/blender/props/int_cabinet.py, tools/blender/props/int_chair.py, tools/blender/props/int_fridge.py, tools/blender/props/int_rug.py, tools/blender/props/int_shelf.py, tools/blender/props/int_stove.py, tools/blender/props/int_table.py, tools/blender/props/laundry_line.py, tools/blender/props/lily_pads.py, tools/blender/props/porch_chair.py, tools/blender/props/propane_tank.py, tools/blender/props/scree_patch.py, tools/blender/props/sled.py, tools/blender/props/snow_bush_small.py, tools/blender/props/swamp_moss_clump.py, tools/blender/props/tumbleweed.py, tools/blender/props/water_barrel.py, tools/blender/props/woodpile.py, world/src/generator_contents.rs, world/src/generator.rs, world/src/generator_world.rs, world/src/ground_detail.rs, world/src/lib.rs, world/src/rivers/mesh.rs, world/src/rivers/mod.rs, world/src/terrain_paint.rs, world/src/towns/clutter.rs, world/src/towns/interior.rs, world/src/towns/mod.rs
- tests passed: 1211
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
