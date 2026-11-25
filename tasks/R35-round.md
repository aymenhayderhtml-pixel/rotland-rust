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
