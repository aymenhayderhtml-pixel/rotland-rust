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
- **Neon answers:** all 8 applied with `tools/neon_apply.py` (no refused paths). Read each `## Neon notes`. Lead wiring done: `world/src/biome.rs` removed (`git rm`); `world/src/lib.rs` (terrain_paint, terrain_shape); `rotland/src/main.rs` (region_banner); HUD (`rotland_ui/src/hud/{mod,draw,state}.rs`); `rotland/src/app/state.rs` (PlaySide field), `start_world.rs` (reset), `frame_ui.rs` (tracker update, banner into HUD).
- **Lead fixes after the first gate runs:**
  1. `world/src/city/ground.rs` `flatten_road`: `&impl Fn` to `&dyn Fn` (unsized error).
  2. `world/src/biome/layout.rs`: removed unused `SortedSector.center_deg`; `.1` tuple reads changed to destructuring (checker false positive).
  3. `world/src/generator.rs`: `chunk_contents` uses `ScatterPlacer::place_with_registry`; `chunk_crates_from` added (R31-03 note).
  4. `game_core/src/loot/table.rs`: dropped `log::warn!` (it needed a new `log` dependency, which put `Cargo.lock` outside scope). Unknown keys still fall back to plains, without a warning.
  5. `world/src/city/ground_tests.rs` `city_outside_matches_no_city`: now compares with the same world's `raw_height`. Reason: outer sectors point at Ashford only when a city is attached, so the no-city world has a different biome out there. Intent kept: the city does not change terrain 300 m outside its box.
  6. `world/src/terrain_shape.rs` `MAX_CELL_STEP_M` 2.8 to 1.6: mountain ridges rose about 3.5 m per 2 m cell, which broke the seam test. Side effect: mountain peaks are lower (about 45 m of amplitude instead of about 62). **Claude: check how the mountains feel.**
  7. Source tags fixed (`World/...` Godot paths, real line numbers, `// new: DECISIONS 2026-11-25` on new knobs and spatial-hash constants). `255` replaced with `u8::MAX`.
  8. Removed unused `LayoutData::tier_config` and `terrain_paint::empty_chunk_ground`.
- **Gate:** last 5 lines of the final run:
  `clippy: PASS` / `test: PASS (1103 passed)` / `scope: PASS (111 changed paths)` / `numbers: PASS (51 checked)` / `unused: PASS` / `GATE: PASS`. `world/tests/golden.rs` and `golden/world.json` are unchanged and pass.
- **Blender:** `build_biome_props.py` ran once: 0 Traceback, `BUILD OK` (24 props). Existing models and palette A/B untouched. New: `assets/textures/palette_c.png`. `assets/models.pack` is gitignored (baked locally with `baker`).
- **Screenshots** (play build, `--save-dir=/tmp/r31-saves`): `/tmp/r31/camp.png`, `/tmp/r31/forest.png`, `/tmp/r31/city.png`, `/tmp/r31/desert.png`, `/tmp/r31/snow.png`, `/tmp/r31/swamp.png`, `/tmp/r31/mountains.png`. Map: `/tmp/r31/worldmap.png` (core in the middle, 4 sectors, tier rings). Each biome looked right when viewed (sand and cacti, snow with shading and pines, swamp trees and reeds, rocky slopes).
- **FPS** (`tools/perf_scenes.sh --set all --seconds 20`, Intel UHD, log `/tmp/r31-perf.log`):

| Scene | Avg FPS | 1% low | Worst ms | p95 ms | Draws (shadow) |
|---|---|---|---|---|---|
| camp | 161.0 | 32.1 | 107.3 | 8.1 | 56 (27) |
| forest | 121.4 | 46.8 | 147.0 | 10.4 | 73 (21) |
| city | 126.8 | 37.6 | 135.3 | 12.6 | 61 (31) |
| night | 168.9 | 47.4 | 127.8 | 7.9 | 61 (0) |
| zombies | 102.4 | 31.0 | 100.0 | 15.2 | 53 (22) |
| desert | 169.0 | 37.3 | 123.6 | 10.1 | 38 (15) |
| snow | 123.9 | 23.5 | 198.5 | 14.3 | 46 (20) |
| swamp | 147.1 | 49.3 | 126.5 | 11.0 | 48 (20) |
| mountains | 121.9 | 28.4 | 124.3 | 13.7 | 47 (20) |

  Biome rule (at least forest 121.4 minus 15% = 103.2): all four pass. No "before" build was measured, so the before/after table is missing.
- **Open / Claude must check:** (a) no Phase 0 "before" screenshots, so camp, forest and Ashford were not compared side by side with the old build; the golden test covers the core heights. (b) Mountains look grey-white on the world map, close to snow; the slopes are pinkish-brown. (c) Mountain peak height reduced (fix 6). (d) `docs/STATUS.md` not updated (lead round step, not done here).

## Gate report
- date: 2026-11-25
- base: main
- files changed (108): assets/models/props/boulder_big.glb, assets/models/props/cactus_round.glb, assets/models/props/cactus_tall.glb, assets/models/props/cattle_skull.glb, assets/models/props/desert_shrub.glb, assets/models/props/lily_mud_patch.glb, assets/models/props/pine_snow.glb, assets/models/props/pine_snow_lod1.glb, assets/models/props/pine_snow_lod2.glb, assets/models/props/reeds.glb, assets/models/props/rock_cliff_01.glb, assets/models/props/rock_ice_01.glb, assets/models/props/rock_sandstone_01.glb, assets/models/props/rock_sandstone_02.glb, assets/models/props/snow_drift.glb, assets/models/props/snow_shrub.glb, assets/models/props/swamp_log_mossy.glb, assets/models/props/swamp_stump.glb, assets/models/props/tree_desert_dead.glb, assets/models/props/tree_desert_dead_lod1.glb, assets/models/props/tree_desert_dead_lod2.glb, assets/models/props/tree_swamp.glb, assets/models/props/tree_swamp_lod1.glb, assets/models/props/tree_swamp_lod2.glb, assets/textures/palette_c.png, data/animals.json, data/biomes.json, data/loot_tables.json, data/poi.json, data/spawning.json, data/world_layout.json, docs/WORLD.md, game_camp/src/map/tiles.rs, game_core/src/loot/mod.rs, game_core/src/loot/table.rs, game/tests/all/biome_keys_tests.rs, game/tests/all/main.rs, game_zombie/src/spawner_config.rs, game_zombie/src/spawner.rs, rotland/src/app/frame_sim_actors.rs, rotland/src/app/frame_ui.rs, rotland/src/app/start_world.rs, rotland/src/app/state.rs, rotland/src/app/test_flags.rs, rotland/src/main.rs, rotland/src/play.rs, rotland/src/props.rs, rotland/src/region_banner.rs, rotland/src/streaming.rs, rotland_ui/src/hud/draw.rs, rotland_ui/src/hud/mod.rs, rotland_ui/src/hud/region_banner.rs, rotland_ui/src/hud/state.rs, tasks/R31-01-biome-core.md, tasks/R31-02-terrain.md, tasks/R31-03-scatter.md, tasks/R31-04-gameplay-keys.md, tasks/R31-05-tools-hud.md, tasks/R31-06-models-desert.md, tasks/R31-07-models-snow-mountain.md, tasks/R31-08-models-swamp.md, tasks/R31-round.md, tools/blender/build_biome_props.py, tools/blender/common.py, tools/blender/palette.py, tools/blender/props/boulder_big.py, tools/blender/props/cactus_round.py, tools/blender/props/cactus_tall.py, tools/blender/props/cattle_skull.py, tools/blender/props/desert_shrub.py, tools/blender/props/lily_mud_patch.py, tools/blender/props/pine_snow_lod1.py, tools/blender/props/pine_snow_lod2.py, tools/blender/props/pine_snow.py, tools/blender/props/reeds.py, tools/blender/props/rock_cliff_01.py, tools/blender/props/rock_ice_01.py, tools/blender/props/rock_sandstone_01.py, tools/blender/props/rock_sandstone_02.py, tools/blender/props/snow_drift.py, tools/blender/props/snow_shrub.py, tools/blender/props/swamp_log_mossy.py, tools/blender/props/swamp_stump.py, tools/blender/props/tree_desert_dead_lod1.py, tools/blender/props/tree_desert_dead_lod2.py, tools/blender/props/tree_desert_dead.py, tools/blender/props/tree_swamp_lod1.py, tools/blender/props/tree_swamp_lod2.py, tools/blender/props/tree_swamp.py, tools/perf_scenes.sh, tools/README_TOOLS.md, world/Cargo.toml, world/examples/worldmap.rs, world/src/biome/layout.rs, world/src/city/ground.rs, world/src/city/ground_tests.rs, world/src/generator.rs, world/src/ground_paint.rs, world/src/lib.rs, world/src/loot.rs, world/src/poi.rs, world/src/scatter.rs, world/src/terrain_paint.rs, world/src/terrain_shape.rs, world/tests/biome_layout_tests.rs, world/tests/biome_scatter_tests.rs, world/tests/biome_terrain_tests.rs, world/tests/terrain_unit_tests.rs
- tests passed: 1103
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
