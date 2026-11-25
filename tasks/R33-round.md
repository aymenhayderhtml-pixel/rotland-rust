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
