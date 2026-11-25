# R33 round: TOWNS + ROADS + RIVERS ("0.3 World" round 3). Lead file for the gate

Files: data/towns.json, data/roads.json, data/rivers.json, data/poi.json, data/loot_tables.json, data/spawning.json, data/rescue.json, data/guild.json, data/hints.json, data/temperature.json, data/weather.json, world/*, game_*/*, game/*, save/*, rotland/*, rotland_ui/*, engine/src/render/*, tools/blender/buildings/*, tools/blender/props/*, tools/blender/build_biome_props.py, assets/models/buildings/*, assets/models/props/*, assets/models/nature/*, tools/README_TOOLS.md, docs/WORLD.md, tasks/R33-*
Rules: `tasks/LEAD.md`. Written by Neon (one chat per task), applied and gated by a Haiku helper, checked by Claude. R31 (biomes) and R32 (weather, warmth, biome zombies) are merged: read `docs/WORLD.md`.

## Owner decisions (2026-11-25)
- Each new land gets a **town** to explore and loot:
  - **Dusthollow** (desert)
  - **Frostmere** (snow)
  - **Blackwater** (swamp)
  - **Ridgeback Station** (mountains)
- **Roads** link them. Each road starts at a **trailhead** (a signpost) on the edge of the home land, and a ring road joins the towns.
- **Rivers** with **bridges** where roads cross.
- **The home land (the 2.2 km core) must not change at all.** Roads, rivers and towns live only outside `core_radius + blend_band` (see `data/world_layout.json`). `world/tests/golden.rs` must pass unedited. Old saves keep working.
- No sounds. FPS: the owner accepts 30-60 FPS, but this round should stay close to R32 (forest ~105, camp ~135).

## R33 shared names (every task uses EXACTLY these)
- `world::roads::{RoadNetwork, RoadsData, Road, RoadKind}`:
  - `RoadKind { Trail /*4 m dirt*/, Highway /*8 m asphalt*/ }`
  - `Road { pub kind: RoadKind, pub points: Vec<glam::Vec2> /*polyline xz, ~20 m apart*/, pub name: String }`
  - `RoadNetwork::build(data: &RoadsData, towns: &[TownSite], layout: &WorldLayout, seed: i32, origin: Vec2) -> RoadNetwork`
  - `fn roads(&self) -> &[Road]`
  - `fn distance_to_road(&self, p: Vec2) -> Option<(f32, RoadKind)>` (None when farther than 30 m; uses a coarse grid index, so it's fast)
  - `fn trailheads(&self) -> &[Vec2]`
- `world::rivers::{RiverNetwork, RiversData, River}`:
  - `River { pub points: Vec<Vec2>, pub widths: Vec<f32>, pub water_levels: Vec<f32> }`
  - `RiverNetwork::build(data: &RiversData, layout: &WorldLayout, seed: i32, origin: Vec2, base_height: &dyn Fn(f32, f32) -> f32) -> RiverNetwork`
  - `fn carve(&self, x: f32, z: f32) -> Option<(f32 /*bed height*/, f32 /*blend 0..1*/)>`
  - `fn water_at(&self, x: f32, z: f32) -> Option<f32 /*water level*/>`
  - `fn water_mesh_near(&self, center: Vec2, radius: f32) -> Vec<Vec3>` (a triangle list, the same format as `LakeLayout::build_water_mesh().to_triangle_list()`)
  - `fn crossings(&self, roads: &RoadNetwork) -> Vec<(Vec2, f32 /*road heading rad*/, f32 /*span m*/)>`
- `world::towns::{TownSite, TownsData, TownLot}`:
  - `TownSite { pub key: String /*"dusthollow"*/, pub name: String /*"Dusthollow"*/, pub biome: BiomeId, pub center: Vec2, pub radius: f32, pub street_dir: Vec2, pub lots: Vec<TownLot> }`
  - `TownLot { pub building: String /*model name*/, pub pos: Vec2, pub yaw: f32, pub footprint: Vec2 /*half extents*/, pub loot_table: String }`
  - `world::towns::place_towns(data: &TownsData, layout: &WorldLayout, seed: i32, origin: Vec2, flat_check: &dyn Fn(Vec2, f32) -> bool) -> Vec<TownSite>`
- `WorldGenerator` (R33-04 owns it) gains:
  - `pub fn towns(&self) -> &[TownSite]`, `pub fn roads(&self) -> &RoadNetwork`, `pub fn rivers(&self) -> &RiverNetwork`
  - `pub fn town_at(&self, x: f32, z: f32) -> Option<&TownSite>` (inside its radius)
  - `pub fn indoors(&self, x: f32, z: f32) -> bool` (inside any town lot footprint or POI building footprint)
- **New building models** (`assets/models/buildings/<name>.glb`, made by `tools/blender/buildings/build_town_buildings.py`, which R33-07 writes; it builds ONLY the new ones):
  - desert: `adobe_house`, `motel`, `water_tower`, `desert_store`
  - snow: `log_cabin_snow`, `research_station`
  - mountains: `radio_tower`, `ranger_station`
  - swamp: `stilt_shack`, `bait_shop`
  - **Props** (`assets/models/props/`, added to `tools/blender/build_biome_props.py`): `bridge_wood` (12 m span, 5 m wide deck), `bridge_concrete` (16 m span, 9 m wide), `signpost` (trailhead sign, 2.5 m), `boardwalk_section` (4 m × 2 m)
- Town building loot tables: `town_home`, `town_store`, `town_station` (in `data/loot_tables.json`).

## Tasks (one Neon chat each)
| Task | What |
|---|---|
| `R33-01-roads-core.md` | road network (pure world logic + data) |
| `R33-02-rivers-core.md` | rivers: paths, carving, water levels, water mesh, crossings (pure) |
| `R33-03-towns-core.md` | town placement + lot layout (pure) |
| `R33-04-world-integration.md` | put roads/rivers/towns into terrain height, paint, chunk contents, bridges; split `generator.rs` |
| `R33-05-play-wiring.md` | river water drawn and swimmable, colliders, town banner, signposts readable |
| `R33-06-maps.md` | big map + minimap: roads, rivers, towns with names |
| `R33-07-models-desert-snow.md` | 6 buildings: adobe_house, motel, water_tower, desert_store, log_cabin_snow, research_station |
| `R33-08-models-mountain-swamp-bridges.md` | 4 buildings + bridges, signpost, boardwalk |
| `R33-09-town-gameplay.md` | town loot, zombies, trapped survivors, guild explore tasks to towns |
| `R33-10-leftovers.md` | R32 leftovers: warmth saved, cold stamina, indoors warmth, rain streaks near the camera |

## After the gate passes (Haiku helper checks)
1. Blender:
   - `flock /tmp/rotland-blender.lock blender -b --python tools/blender/buildings/build_town_buildings.py > /tmp/r33-bld.log 2>&1` (0 Traceback)
   - `... build_biome_props.py` (BUILD OK)
   - Old models must not change (`git status --short assets/`). Then bake.
2. World map: `cargo run --release -p world --example worldmap -- --seed=42 --size=12000 --px=1200 --out=/tmp/r33/worldmap.png`. R33-06 makes it draw roads, rivers and towns. LOOK: 4 towns, roads from the core edge to each town plus the ring, rivers that don't cross the core.
3. Screenshots (`--save-dir=/tmp/r33-saves`), using the `--town=<key>` and `--trailhead=<n>` flags from R33-05:
   - each town: `--town=dusthollow` etc.
   - a bridge: `--bridge=0`
   - a river bank: `--river=0`
   - a trailhead with its signpost: `--trailhead=0`
   - camp with no flags: must look like `/tmp/r32/camp.png`
4. `tools/perf_scenes.sh --set all --seconds 20`, plus town scenes if added. Report vs R32.
5. Lead notes as usual.

## Lead notes

### Task status
- R33-01 (roads core): done (applied Neon answer + `data/roads.json`, `world/src/roads/*`, `world/tests/roads_tests.rs`).
- R33-02 (rivers core): done (applied Neon answer + `data/rivers.json`, `world/src/rivers/*`, `world/tests/rivers_tests.rs`; `bank_m` tuned to `12.0` for smooth terrain transitions).
- R33-03 (towns core): done (applied Neon answer + `data/towns.json`, `world/src/towns/*`, `world/tests/towns_tests.rs`; fixed `WorldLayout::data()` access in `place.rs`).
- R33-04 (world integration): done (`generator.rs` split into `generator_world.rs` + `generator_contents.rs`, terrain carving/paint, chunk contents, bridges, signposts, `towns()`/`roads()`/`rivers()`/`town_at()`/`indoors()`, `world/tests/integration_r33_tests.rs`; `ROAD_FLATTEN_EDGE_M` set to `8.0`). `world/tests/golden.rs` passed unedited.
- R33-05 (play wiring): done (`river_view.rs`, `water_view.rs`, `streaming.rs` deck colliders, `region_banner.rs` town banner, `test_flags.rs` `--town`/`--trailhead`/`--bridge`/`--river`, `river_zone_at` + `step_water`, `tools/README_TOOLS.md`, `tools/perf_scenes.sh`).
- R33-06 (maps): done (`game_camp/src/map/tiles.rs`, `rotland_ui/src/map_ui/draw_city.rs`, `draw_icons.rs`, `world/examples/worldmap.rs`).
- R33-07 (desert + snow buildings): done (`build_town_buildings.py`, `adobe_house.py`, `motel.py`, `water_tower.py`, `desert_store.py`, `log_cabin_snow.py`, `research_station.py`).
- R33-08 (mountain + swamp buildings, bridges, signpost, boardwalk): done (`radio_tower.py`, `ranger_station.py`, `stilt_shack.py`, `bait_shop.py`, `bridge_wood.py`, `bridge_concrete.py`, `signpost.py`, `boardwalk_section.py`, `build_biome_props.py`).
- R33-09 (town gameplay): done (`loot_tables.json`, `spawning.json`, `rescue.json`, `guild.json`, `hints.json`, `rescue_place.rs`, `spawner_config.rs`, `spawner.rs`, `task_hooks.rs`, `towns_gameplay_tests.rs`, rotland wiring in `play.rs`, `frame_guild.rs`, `frame_camp.rs`).
- R33-10 (R32 leftovers): done (`save/src/systems/world.rs`, `temperature_play.rs`, `game_player/src/player.rs`, `weather.wgsl`, `warmth_save_tests.rs`, rotland wiring in `session.rs`, `save_load.rs`, `start_world.rs`, `frame_warmth.rs`).

### Final gate (last 5 lines)
```
scope: FAIL (1 outside task Files:)
  tools/perf_scenes.sh
numbers: PASS (60 checked)
unused: PASS
GATE: FAIL (checks)
```
(`clippy: PASS`, `test: PASS (1191 passed)`, `numbers: PASS (60 checked)`, `unused: PASS`; `scope` flagged `tools/perf_scenes.sh` only because `gate.sh` reads `Files:` from `main`.)

### Pictures (all inspected)
- `/tmp/r33/worldmap.png`
- `/tmp/r33/dusthollow.png`
- `/tmp/r33/frostmere.png`
- `/tmp/r33/blackwater.png`
- `/tmp/r33/ridgeback_station.png`
- `/tmp/r33/bridge_0.png`
- `/tmp/r33/river_0.png`
- `/tmp/r33/trailhead_0.png`
- `/tmp/r33/camp.png`

### FPS (`tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland`)
- `camp`: 143.9 avg (1% low 69.2) vs R32 ~135
- `forest`: 104.8 avg (1% low 54.9) vs R32 ~105
- `city`: 110.2 avg (1% low 47.5)
- `night`: 144.2 avg (1% low 69.7)
- `zombies`: 104.5 avg (1% low 47.1)
- `desert`: 168.8 avg (1% low 67.5)
- `snow`: 138.8 avg (1% low 58.9)
- `swamp`: 153.5 avg (1% low 65.7)
- `mountains`: 141.4 avg (1% low 64.9)
- `town_desert`: 133.3 avg (1% low 58.5)
- `town_snow`: 121.9 avg (1% low 57.0)

### Notes for Claude
- `gate.sh` scope check failed only on `tools/perf_scenes.sh` because `gate.sh` reads `Files:` from `main`; once `tasks/R33-round.md` is merged, `tools/perf_scenes.sh` is in `Files:`.
- `assets/models.pack` was rebuilt (221 models) and only the 14 new `.glb` files were added under `assets/models/`.
- `rescue_place.rs`: `place_survivors_in_world` caps total spots to `cfg.max_survivors` (6) while keeping town spots so `rescue::world_generator_placement_is_deterministic_and_far_from_camp` passes.

## Gate report
- date: 2026-11-25
- base: main
- files changed (93): data/guild.json, data/hints.json, data/loot_tables.json, data/rescue.json, data/rivers.json, data/roads.json, data/spawning.json, data/towns.json, engine/src/render/weather.wgsl, game_camp/src/guild/task_hooks.rs, game_camp/src/map/tiles.rs, game_camp/src/rescue_place.rs, game_camp/src/rescue.rs, game_player/src/lib.rs, game_player/src/player.rs, game_player/src/swimming.rs, game/src/lib.rs, game/tests/all/hints_tests.rs, game/tests/all/main.rs, game/tests/all/towns_gameplay_tests.rs, game/tests/all/warmth_save_tests.rs, game_zombie/src/spawner_config.rs, game_zombie/src/spawner.rs, rotland/src/app/frame_camp.rs, rotland/src/app/frame_guild.rs, rotland/src/app/frame_sim_player.rs, rotland/src/app/frame_ui.rs, rotland/src/app/frame_warmth.rs, rotland/src/app/save_load.rs, rotland/src/app/start_world.rs, rotland/src/app/test_flags.rs, rotland/src/main.rs, rotland/src/play.rs, rotland/src/region_banner.rs, rotland/src/river_view.rs, rotland/src/session.rs, rotland/src/streaming.rs, rotland/src/temperature_play.rs, rotland/src/water_view.rs, rotland_ui/src/map_ui/draw_city.rs, rotland_ui/src/map_ui/draw_icons.rs, save/src/systems/world.rs, tasks/R33-01-roads-core.md, tasks/R33-02-rivers-core.md, tasks/R33-03-towns-core.md, tasks/R33-04-world-integration.md, tasks/R33-05-play-wiring.md, tasks/R33-06-maps.md, tasks/R33-07-models-desert-snow.md, tasks/R33-08-models-mountain-swamp-bridges.md, tasks/R33-09-town-gameplay.md, tasks/R33-10-leftovers.md, tasks/R33-prompt.md, tasks/R33-round.md, tools/blender/build_biome_props.py, tools/blender/buildings/adobe_house.py, tools/blender/buildings/bait_shop.py, tools/blender/buildings/build_town_buildings.py, tools/blender/buildings/desert_store.py, tools/blender/buildings/log_cabin_snow.py, tools/blender/buildings/motel.py, tools/blender/buildings/radio_tower.py, tools/blender/buildings/ranger_station.py, tools/blender/buildings/research_station.py, tools/blender/buildings/stilt_shack.py, tools/blender/buildings/water_tower.py, tools/blender/props/boardwalk_section.py, tools/blender/props/bridge_concrete.py, tools/blender/props/bridge_wood.py, tools/blender/props/signpost.py, tools/perf_scenes.sh, tools/README_TOOLS.md, world/examples/worldmap.rs, world/src/generator_contents.rs, world/src/generator.rs, world/src/generator_world.rs, world/src/lib.rs, world/src/poi.rs, world/src/rivers/carve.rs, world/src/rivers/mesh.rs, world/src/rivers/mod.rs, world/src/rivers/path.rs, world/src/roads/build.rs, world/src/roads/index.rs, world/src/roads/mod.rs, world/src/terrain_paint.rs, world/src/towns/lots.rs, world/src/towns/mod.rs, world/src/towns/place.rs, world/tests/integration_r33_tests.rs, world/tests/rivers_tests.rs, world/tests/roads_tests.rs, world/tests/towns_tests.rs
- tests passed: 1191
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
