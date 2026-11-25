# R31 round: NEW WORLD ENGINE + 4 NEW BIOMES (desert, snow, swamp, mountains). Lead file for the gate

Files: data/biomes.json, data/world_layout.json, data/scatter.json, data/spawning.json, data/animals.json, data/loot_tables.json, data/poi.json, world/*, game_*/*, game/*, rotland/*, rotland_ui/*, engine/src/cli.rs, tools/blender/props/*, tools/blender/palette.py, tools/blender/common.py, tools/blender/build_biome_props.py, assets/models/props/*, assets/textures/palette_c.png, tools/perf_scenes.sh, tools/README_TOOLS.md, docs/CODEMAP.md, docs/WORLD.md, tasks/R31-*
Rules: `tasks/LEAD.md`. This is the first of 4 "0.3 World" rounds (R31 engine+biomes, R32 weather/temperature/biome zombies, R33 places/rivers/roads, R34 map/polish/speed). Weapons moved to R35.

## Owner decisions (2026-11-25, in docs/DECISIONS.md)
- **Rings layout:** today's world (camp, plains, forest, Ashford, the lake) stays EXACTLY the same inside a core circle. Outside it, each direction leads to a different biome, and the farther out you go, the higher the **danger tier** (more zombies, better loot).
- **Endless world**, like today (no edge).
- **Light temperature** later (R32). In R31 the biome data only stores temperatures and the thirst factor; no new bar yet.
- Old saves must keep working: the camp, Ashford, the lake, base parts and terrain heights inside the core must not move by even 1 cm.

## Phase 0 (lead, before the subagents start)
1. Take "before" screenshots for comparison (release build, `--save-dir=/tmp/r31-saves`): camp (no flags), `--at=300,300` (forest), and `--at=-384,1792` (Ashford). Keep them in `/tmp/r31-before/`.
2. Run `tools/perf_scenes.sh --seconds 20`, and keep the table for Lead notes.
3. Read `world/src/generator.rs`, `world/src/biome.rs`, `world/src/scatter.rs`, `data/scatter.json` and `world/tests/golden.rs` yourself, so you can check the subagents' work.

## Tasks (about 8 subagents; they all start together, because the shared names below are fixed)
| Task | What | Kind |
|---|---|---|
| `R31-01-biome-core.md` | biome list + data + ring/sector layout + danger tiers (pure logic, tests) | code |
| `R31-02-terrain.md` | heights + ground colours per biome, split `generator.rs`, core stays exact | code |
| `R31-03-scatter.md` | trees/rocks/props per biome, POIs per biome, build each chunk's contents once | code |
| `R31-04-gameplay-keys.md` | remove the "forest"/"plains" strings everywhere; zombies/animals/loot per biome + tier | code |
| `R31-05-tools-hud.md` | world map picture tool, `--biome=` teleport, minimap colours, "Entering: Desert" banner | code |
| `R31-06-models-desert.md` | desert models + the new colour sheet C | Blender script |
| `R31-07-models-snow-mountain.md` | snow + mountain models | Blender script |
| `R31-08-models-swamp.md` | swamp models | Blender script |

## R31 shared names (every subagent uses EXACTLY these)
- `world::BiomeId` (in `world/src/biome/mod.rs`; `biome.rs` becomes the folder `world/src/biome/`): `#[repr(u8)] enum BiomeId { Plains = 0, Forest = 1, Desert = 2, Snow = 3, Swamp = 4, Mountains = 5 }`, plus `BiomeId::ALL: [BiomeId; 6]`, `fn key(self) -> &'static str` ("plains", "forest", "desert", "snow", "swamp", "mountains"), and `fn from_key(&str) -> Option<BiomeId>`. Plains and Forest keep their numbers, because the golden test compares `biome as i32`.
- `world::biome::registry::{BiomeRegistry, BiomeDef, BiomeGround, BiomeHeight}` reads `data/biomes.json`. Use `BiomeRegistry::load() -> Result<BiomeRegistry, String>` (via `paths::repo_path`), `BiomeRegistry::from_json(&str)` for tests, and `fn get(&self, id: BiomeId) -> &BiomeDef`. JSON fields (snake_case) and Rust fields are the same:
  ```
  { "biomes": [ { "key": "desert", "name": "Desert",
      "map_color": [222,196,140], "fog_tint": [235,215,170],
      "ground": { "base": [..], "alt": [..], "slope": [..], "peak": [..] or null, "peak_height": 0.0 },
      "height": { "base": 3.0, "amplitude": 6.0, "frequency": 0.004, "octaves": 3, "ridged": false, "dune": 0.0 },
      "scatter": [ same rule objects as data/scatter.json "forest"/"plains" ],
      "zombie_density": 0.6, "loot_table": "desert", "animals": ["rabbit"],
      "temperature_day": 38.0, "temperature_night": 12.0, "thirst_mult": 1.6, "footstep": "sand" } ] }
  ```
  Plains and forest also get entries (map colour, name, zombie density, loot table, temps, footstep), but their **height, ground and scatter stay the old code and `scatter.json`** (core exactness). Their `height`/`ground`/`scatter` fields may be filled with the old values for reference, and are used only outside the core.
- `world::biome::layout::{WorldLayout, WorldLayoutData, BiomeSample}` reads `data/world_layout.json`:
  `pub struct BiomeSample { pub primary: BiomeId, pub secondary: BiomeId, pub blend: f32 /*0 = all primary, up to 0.5*/, pub outer: f32 /*0 in the core, 1 past the blend band*/, pub tier: u8 /*0 in the core, 1..=5 outside*/ }`.
  `WorldLayout::new(data: &WorldLayoutData, seed: i32, origin: glam::Vec2 /*camp pad centre*/, reference_dir: glam::Vec2 /*camp->Ashford, or +X*/)` and `fn sample(&self, x: f32, z: f32) -> BiomeSample` (outer biome only: the primary is the outer sector biome; the generator mixes in the core biome by `outer`).
- `WorldGenerator` (R31-02 owns it) gains `pub fn biome_sample(&self, x: f32, z: f32) -> BiomeSample` (the full answer: in the core, primary = old Plains/Forest rule and outer = 0) and `pub fn tier(&self, x: f32, z: f32) -> u8`. `biome()` keeps its signature and returns `biome_sample().primary`. `forest_weight()` keeps working (in the core: exactly as today; outside: 0 for non-forest biomes).
- New colour sheet C (`tools/blender/palette.py` `COLORS_C`, written to `assets/textures/palette_c.png`, used by `common.palette_material(sheet="C")`; R31-06 adds it). Exactly these 16 names in this order: `sand_light, sand_dark, sandstone, sandstone_dark, cactus_green, cactus_dark, dry_brush, bone_white, snow_white, snow_shadow, ice_blue, pine_snow_dark, swamp_moss, swamp_mud, reed_tan, swamp_bark`.
- New model names (each `assets/models/props/<name>.glb`, trees also `_lod1`/`_lod2`):
  desert `cactus_tall, cactus_round, desert_shrub, rock_sandstone_01, rock_sandstone_02, cattle_skull, tree_desert_dead (+lod1, lod2)`;
  snow/mountain `pine_snow (+lod1, lod2), rock_ice_01, snow_drift, snow_shrub, boulder_big, rock_cliff_01`;
  swamp `tree_swamp (+lod1, lod2), reeds, swamp_log_mossy, swamp_stump, lily_mud_patch`.
- Danger tier effects (from `data/world_layout.json` `tiers`, index = tier-1): `zombie_mult` multiplies the spawn density, and `loot_rolls_bonus` adds extra loot rolls in crates and POIs.

## After the gate passes (lead checks)
1. Rebuild the new props once: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_biome_props.py > /tmp/r31-props.log 2>&1` (0 `Traceback`, ends `BUILD OK`), then `cargo run --release -p baker`.
2. `cargo run --release -p world --example worldmap -- --seed=42 --size=12000 --out=/tmp/r31/worldmap.png` (R31-05). LOOK at it: camp in the middle, today's green core, 4 biome sectors outside, soft borders, mountains reaching high.
3. Same screenshots as Phase 0. Camp, forest and Ashford must look the SAME as before (compare them side by side).
4. One screenshot per new biome: `--biome=desert`, `--biome=snow`, `--biome=swamp` and `--biome=mountains` (plus `--cam-yaw` to taste). Each must clearly look like its biome: ground colour, models, shape. Fix what's clearly wrong (green grass in the desert, floating cacti, all-white snow without shading).
5. `tools/perf_scenes.sh --seconds 20` with the 4 new biome scenes added (R31-05 adds them). Each biome must reach at least the forest's average FPS minus 15%.
6. Lead notes: each task done or not, the last 5 gate lines, the before/after FPS table, all screenshot paths (`/tmp/r31/...`), and anything Claude must check.

## Lead notes
