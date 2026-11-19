# Rotland Code Map

Comprehensive architecture and codebase map for the Rotland Rust workspace (`/home/aymen/Documents/rotland-rust`).

## 1. Game Boot & Runtime Architecture

### How the Game Boots

1. **Entry Point (`rotland/src/main.rs:main()`)**:
   - Initializes logging (`env_logger`) with filters defaulting to `"info,wgpu_core=warn,wgpu_hal=warn"`.
   - Instantiates `R0` with default settings (seed 42, hour 12.0) and calls `engine::run(R0 { .. })`.
2. **Engine Runner (`engine::app::run`)**:
   - Parses command-line arguments into `EngineArgs`.
   - Creates the `winit` event loop and main window (default size 1920x1080, title "Rotland").
   - Initializes `Gpu` context (wgpu on Vulkan surface), `FrameTarget` (LDR/HDR offscreen targets), `Blit` pipeline, `UiLayer` (egui 0.37), `FrameTimer`, `Input`, and `FrameStats`.
   - Enters event loop, calling `game.init(&gpu, &args)` once GPU context is ready, then ticking `game.update(&input, dt)`, `game.render(&gpu, &target, &mut encoder)`, and `game.ui(&ctx)` each frame.
3. **Game Trait (`engine::app::Game`)**:
   - `fn init(&mut self, gpu: &Gpu, args: &EngineArgs)`: One-time setup, asset loading, pipeline creation.
   - `fn update(&mut self, input: &Input, dt: f32)`: Simulation tick (player controller, physics, AI, animations).
   - `fn render(&mut self, gpu: &Gpu, target: &FrameTarget, encoder: &mut wgpu::CommandEncoder)`: Forward drawing passes (shadows, terrain chunks, instanced props, animated meshes).
   - `fn ui(&mut self, ctx: &egui::Context)`: Immediate-mode HUD, menus, and inventory drawing.
   - `fn display(&self) -> Option<DisplayRequest>`: Dynamic render scale and vsync adjustments.
   - `fn cursor_grab(&self) -> Option<bool>`: Requests mouse cursor capture or release.
   - `fn should_quit(&self) -> bool`: Game exit flag.
   - `fn stats(&self) -> GameStats`: Custom telemetry metrics serialized to stats JSON.
   - `fn screenshot_request(&mut self) -> Option<PathBuf>`: Frame capture request path.
4. **Rotland `R0` Implementation (`rotland/src/main.rs` + `rotland/src/app/`, split in R10)**:
   - `main.rs` (~230 lines) keeps `main()`, the `Game` impl and a short `update()` that calls the frame steps in this order. `R0` holds one field per state group, defined in `app/state.rs`: `RenderState`, `WorldState`, `ActorsState`, `PlaySide`, `UiState`, `SessionState`, `AudioState` (each step takes the groups it needs as separate borrows).
   - Frame steps: `app/frame_input.rs` (keys, actions) -> `app/frame_actors.rs` (player, camera, ladders, melee, tree strikes) -> `app/frame_world.rs` (needs, crafting timers, shooting, E interactions, spawner, clock) -> `app/frame_audio.rs` (listener, footsteps) -> `app/frame_camp.rs` (hints, stats, bounties, map reveal, farm) -> `app/frame_sim.rs` (zombies, POI zombies, voices, guards, wildlife, dogs; `app/frame_cars.rs` car ramming) -> `app/render_frame.rs` (chunk streaming, scene, skinned batches) -> `app/frame_ui.rs` (HUD, bubbles, camp screens, hub, menus).
   - Start and saves: `app/startup.rs` (init, flags), `app/start_world.rs` (build the world, restore a save), `app/save_load.rs` (save now), `app/stats.rs` (stats JSON, display request).
   - Other folders split in R10: `rotland/src/hud/` (one file per HUD widget; `icons.rs` = item picture cache), `rotland/src/zombies/` (types, spawning, combat glue, rendering). New pure helpers: `game/src/dir.rs` (yaw forward/right), `game/src/hud_math.rs`, `game/src/zombie/grid.rs`, `humanoid/src/ik.rs` + `swing.rs`, `save/src/systems/{weapons,settlements}.rs`, `rotland/src/hub_ui/{item_text,draw_crafting}.rs`.
   - Note: the per-file tables in section 2 were written before R10 and do not list these new files yet.

### Crate Dependency Order

Topological dependency order (from lowest foundation to top-level game):

```text
1. paths          - Project folder locator (zero internal dependencies)
2. pack           - Model pack binary format & schema (depends on: paths)
3. baker          - Offline asset baker tool (depends on: paths, pack)
4. world          - Pure procedural world generation (dev-depends on: paths)
5. physics        - Rapier3D physics & character mover (depends on: world, pack; dev-depends on: paths)
6. save           - Save file format & slot management (dev-depends on: paths)
7. audio          - Sound registry, 3D math, and mixer (depends on: paths)
8. engine         - wgpu renderer, winit app, egui UI (depends on: pack)
9. humanoid       - Skeletal animation blend & weapon hold (depends on: paths, engine, pack)
10. game          - Pure gameplay systems & simulation (depends on: engine, physics, paths, world, pack)
11. rotland       - Executable & wiring (depends on: paths, engine, pack, world, physics, game, save, audio, humanoid)
```

### Runtime File Locations

- **`paths::repo_path(relative)`**: Resolves runtime paths by inspecting the `ROTLAND_REPO` environment variable, or searching parent directories of current working directory / executable for `Cargo.lock` and `data/`.
- **`data/*.json`**: Game data definitions (items, recipes, zombies, wildlife, economy, quests, hints) loaded via `paths::repo_path("data/<file>.json")`.
- **`assets/models.pack`**: Baked binary asset pack containing all 3D meshes, markers, colliders, skeletons, and animation clips; loaded via `pack::ModelPack::load(&paths::repo_path("assets/models.pack"))`.
- **`assets/audio/` via `data/sounds.json`**: Sound registry defined in `data/sounds.json` mapping IDs to audio files located under `assets/audio/sfx` and `assets/audio/ambient`.

## 2. Crates & File Directory

### Crate `paths`

Runtime repository root and asset path locator. Zero internal dependencies. (1 files)

| File | What it does | Key pub items |
|---|---|---|
| `paths/src/lib.rs` | Finds project root (Cargo.lock and data/) at runtime via ROTLAND_REPO or parent directory search. | `repo_root()`, `repo_path()` |

### Crate `pack`

Binary 3D model pack serialization format (postcard). Zero internal dependencies. (7 files)

| File | What it does | Key pub items |
|---|---|---|
| `pack/examples/list.rs` | CLI utility listing models, vertex counts, bounds, markers, and clips in models.pack with optional prefix filtering. | `main()` |
| `pack/src/lib.rs` | Rotland 3D model pack binary format. | `ConvexCollider`, `Marker`, `Model`, `ModelPack`, `PACK_VERSION`, `Channel` (+6 more) |
| `pack/src/marker.rs` | Named empty node exported from Blender (e.g. loot_1, spawn_1, grip_L) and convex collider bounds. | `Marker`, `ConvexCollider` |
| `pack/src/model.rs` | Model pack format data structures: Model, ModelPack container, and version constant. | `PACK_VERSION`, `Model`, `ModelPack` |
| `pack/src/skeleton.rs` | Hierarchical bone skeleton, joints, and skeletal animation clips with TRS channels. | `Skeleton`, `Joint`, `Clip`, `Channel`, `ChannelKind`, `Interpolation` |
| `pack/src/vertex.rs` | Vertex layout shared by baker, asset pack and GPU renderer (32 bytes: pos, normal, color, bone). | `PackVertex` |
| `pack/tests/roundtrip.rs` | Tests PackVertex memory layout and verifies postcard serialization roundtrip for models, markers, skeletons, and clips. | `test_pack_vertex_size_and_alignment()`, `test_pack_roundtrip()` |

### Crate `baker`

Offline asset baking tool converting Blender GLTF (.glb) files into compact models.pack. (8 files)

| File | What it does | Key pub items |
|---|---|---|
| `baker/src/bake.rs` | Core baking pipeline orchestrating GLB scanning, geometry processing, palette coloring, and pack writing. | `bake_glb()` |
| `baker/src/color.rs` | Palette texture sampling and color space conversion for shared palette textures. | `sample_palette_nearest()`, `factor_to_srgb()` |
| `baker/src/lib.rs` | Asset baker library transforming Blender .glb assets into compact ModelPack binaries. | `bake_glb` |
| `baker/src/main.rs` | CLI executable entry point for the asset baker: scans Blender GLB models and bakes them into assets/models.pack. | `main()` |
| `baker/src/mesh.rs` | Extracts mesh primitives, vertex positions, normals, palette colors, and bone skin weights from GLTF models. | `ExtractedGeometry`, `extract_geometry()` |
| `baker/src/skeleton.rs` | Extracts armatures, inverse bind matrices, and animation channels (translation, rotation, scale) from GLTF. | `extract_skeleton_and_clips()` |
| `baker/src/transform.rs` | Coordinate space transformation math (converting Blender Z-up to engine Y-up coordinates). | `compute_world_transforms()`, `collect_skeleton_nodes()`, `extract_markers()` |
| `baker/tests/bake_test.rs` | Integration tests verifying that all GLB models bake cleanly, skeletons have clips, and pack loads back identically. | `test_every_glb_bakes_without_error_and_has_triangles()`, `test_zombie_01_skeleton_and_clips()`, `test_tree_pine_vertex_colours_in_palette()`, `test_pack_saves_and_loads_back_identically()` |

### Crate `world`

Pure procedural world generation: terrain meshes, biomes, scatter placement, settlements, POIs, day/night celestial math. (16 files)

| File | What it does | Key pub items |
|---|---|---|
| `world/src/biome.rs` | Biome classification enum and rules determining terrain color, flora distribution, and ground texture. | `BiomeId` |
| `world/src/color.rs` | RGBA color utilities and conversions for terrain vertex coloring and ground painting. | `Color` |
| `world/src/daynight.rs` | Pure celestial math computing sun direction, ambient light, sky color, and fog as functions of time of day. | `MAX_SUN_ENERGY`, `MOON_ENERGY`, `MOON_COLOR`, `SHADOW_MIN_ENERGY`, `NIGHT_AMBIENT`, `DAY_AMBIENT` (+17 more) |
| `world/src/generator.rs` | Procedural chunk terrain generator producing heightmaps, triangle meshes, vertex colors, and biome distribution. | `GroundTriangle`, `ChunkGround`, `WorldGenerator` |
| `world/src/ground_paint.rs` | Ground paint rules blending terrain textures (grass, dirt, rock, paths) based on slope and noise. | `GroundPaint` |
| `world/src/lib.rs` | World generation crate root: exports terrain generator, biomes, scatter placer, settlements, POIs, and clock. | `BiomeId`, `Color`, `DayNightLook`, `SkyColors`, `ChunkGround`, `GroundTriangle` (+28 more) |
| `world/src/loot.rs` | Places loot container markers and loot spots within world chunks and settlements. | `LootSpot`, `LootPlacer` |
| `world/src/math.rs` | Mathematical helper functions including linear interpolation (lerp) and smooth step. | `lerp()`, `smooth_step()` |
| `world/src/noise.rs` | FastNoise-Lite wrapper configured to produce deterministic fractal noise matching Godot FastNoiseLite. | `GodotNoise` |
| `world/src/poi.rs` | Point-of-Interest building placement, perimeter calculation, building definitions, and zombie spawn anchors. | `PoiInstance`, `PoiBuildingDef`, `PoiConfigData`, `PoiConfig`, `PoiGeometry`, `PoiPlacer` |
| `world/src/random.rs` | Bit-exact port of .NET System.Random (Knuth subtractive generator) for reproducible procedural seeds. | `DotnetRandom` |
| `world/src/scatter.rs` | Deterministic Poisson/grid scatter placer for trees, rocks, bushes, and prop instances per chunk. | `PropInstance`, `ScatterRule`, `ScatterVisibility`, `ScatterLootData`, `ScatterConfigData`, `ScatterConfig` (+1 more) |
| `world/src/settlement.rs` | Settlement layout generator flattening terrain pads and positioning camp fences, gates, and buildings. | `SettlementConfigData`, `SettlementConfig`, `SettlementSite`, `SettlementGround`, `SettlementPlacer` |
| `world/src/time.rs` | In-game time-of-day clock tracking hours, days, minute progression, and sunrise/sunset events. | `ClockChanges`, `Clock` |
| `world/tests/golden.rs` | Golden test verifying that Rust terrain generation outputs bit-exact heights and vertices matching Godot world.json. | `terrain_matches_godot()` |
| `world/tests/terrain_unit_tests.rs` | Unit tests verifying terrain generator determinism, height ranges, seamless chunk borders, and settlement pads. | `test_determinism()`, `test_different_seeds()`, `test_heights_in_sane_range()`, `test_chunk_edges_match_seamlessly()` |

### Crate `physics`

Rapier3D collision detection and kinematic character controller for player, zombies, and terrain. (6 files)

| File | What it does | Key pub items |
|---|---|---|
| `physics/examples/bench25.rs` | Performance benchmark measuring Rapier3D physics simulation time with 25 moving character capsules. | `main()` |
| `physics/src/lib.rs` | Rapier3D collision physics crate root: exposes PhysicsWorld, CharacterMover, and collision shapes. | `CharacterMover`, `Hit`, `MoveResult`, `GROUP_CHARACTER`, `GROUP_WORLD`, `PhysicsWorld` (+5 more) |
| `physics/src/mover.rs` | Kinematic character controller computing slope sliding, gravity, obstacle pushing, and grounded states. | `CharacterMover` |
| `physics/src/types.rs` | Physics data structures including GroundHit, CharacterMove, CollisionMask, and RaycastResult. | `GROUP_WORLD`, `GROUP_CHARACTER`, `Hit`, `MoveResult` |
| `physics/src/world.rs` | Rapier3D physics world managing rigid bodies, terrain heightfield colliders, static trimeshes, and raycasts. | `CHUNK_SIZE`, `CHUNK_CELL`, `CHUNK_SAMPLES`, `CHUNK_CELLS`, `PhysicsWorld` |
| `physics/tests/physics_tests.rs` | Integration tests verifying capsule gravity landing, sliding along walls, slope limits, and chunk mesh lifecycles. | `test_capsule_dropped_above_flat_chunk_lands_and_reports_on_floor()`, `test_walking_into_box_slides_and_does_not_pass_through()`, `test_walking_up_30_degree_ramp_works_60_degree_does_not()`, `test_ground_height_matches_world_generator()` |

### Crate `save`

Save game serialization, slot management, and Godot save file importer/exporter. (15 files)

| File | What it does | Key pub items |
|---|---|---|
| `save/examples/check_godot.rs` | Read-only utility inspecting existing Godot save slots and checking field compatibility with Rust types. | `main()` |
| `save/src/error.rs` | Error types for save/load operations, JSON parsing, file I/O, and missing slot errors. | `SaveError` |
| `save/src/file.rs` | Save file structures (HeaderData, SystemData, SaveFile) maintaining compatibility with Godot save format. | `FORMAT_VERSION`, `utc_now_iso8601()`, `format_utc_timestamp()`, `Systems`, `SaveFile` |
| `save/src/lib.rs` | Save crate root: re-exports SaveFile, SlotManager, SaveError, and subsystem save structures. | `SaveError`, `format_utc_timestamp`, `utc_now_iso8601`, `SaveFile`, `Systems`, `FORMAT_VERSION` (+22 more) |
| `save/src/slots.rs` | SlotManager managing save slot directories (1-3), active save paths, Godot imports, and atomic writes. | `DEFAULT_SLOT_COUNT`, `SlotState`, `SlotInfo`, `normalize_path()`, `is_subpath_or_same()`, `default_base_dir()` (+5 more) |
| `save/src/systems/difficulty.rs` | Serialization for difficulty settings, death rules, and custom damage/hunger multipliers. | `DeathRule`, `DifficultySaveData` |
| `save/src/systems/inventory.rs` | Serialization for player inventory slots, item IDs, quantities, durability, and hotbar binding. | `SavedSlot`, `InventorySaveData` |
| `save/src/systems/mod.rs` | Subsystem save data serializers module root. | `DeathRule`, `DifficultySaveData`, `InventorySaveData`, `SavedSlot`, `PlayerSaveData`, `Vec3Data` (+7 more) |
| `save/src/systems/player.rs` | Serialization for player position, rotation, health, stamina, camera perspective, and active vehicle. | `Vec3Data`, `PlayerSaveData` |
| `save/src/systems/stats.rs` | Serialization for career player statistics (kills, headshots, deaths, distance traveled). | `StatsSaveData` |
| `save/src/systems/time.rs` | Serialization for world time-of-day, current hour, and day count. | `TimeSaveData` |
| `save/src/systems/wallet.rs` | Serialization for cash on hand and camp bank account balance. | `WalletSaveData` |
| `save/src/systems/world.rs` | Serialization for world seed, generator configuration, and discovered settlements. | `WorldSaveData` |
| `save/src/systems/worldstate.rs` | Serialization for dynamic world state: felled trees, cleared POIs, visited buildings, and stray animals. | `DroppedItemData`, `CampfireData`, `WorldStateSaveData` |
| `save/tests/saves.rs` | Integration tests for save slot creation, serialization round-trip, Godot save importing, and corruption resistance. | `round_trip_save_file()`, `slot_info_empty_damaged_filled()`, `fixture_parses_and_preserves_unknown_systems()`, `guard_refuses_godot_and_default_folders()` |

### Crate `audio`

Sound effects library, 3D spatial audio math, and rodio mixer. (4 files)

| File | What it does | Key pub items |
|---|---|---|
| `audio/src/lib.rs` | Audio crate root: exposes sound manager, one-shot playback, spatial sound positioning, volume controls, and a continuous RPM-pitched `car_engine_loop` handle. | `Audio`, `EventDef`, `SoundData`, `SoundDef`, `distance_db`, `linear_to_db` (+1 more) |
| `audio/src/library.rs` | SoundLibrary registry parsing data/sounds.json and mapping sound IDs to audio file paths and playback options. | `SoundDef`, `EventDef`, `SoundData` |
| `audio/src/mix.rs` | Pure math for 3D spatial audio: calculates volume attenuation and stereo panning based on listener distance and angle. | `distance_db()`, `pan()`, `linear_to_db()` |
| `audio/tests/files_decode.rs` | Verifies that every audio file referenced in data/sounds.json exists on disk and decodes without errors. | `every_sound_file_decodes()` |

### Crate `engine`

Custom wgpu rendering engine on Vulkan: winit windowing, frame targets, blit, timing, egui UI, and stats. (19 files)

| File | What it does | Key pub items |
|---|---|---|
| `engine/src/anim.rs` | CPU skeletal animation evaluator sampling clips and blending joint matrices for skinned rendering. | `Rig` |
| `engine/src/app.rs` | Winit application lifecycle loop: manages window events, frame timing, blit pipeline, and Game trait invocation. | `GameStats`, `DisplayRequest`, `DisplayChange`, `check_display_change()`, `Game`, `run()` |
| `engine/src/capture.rs` | Offscreen frame grabber and screenshot saver writing RGBA GPU render targets to PNG files. | `save_png()` |
| `engine/src/cli.rs` | Engine CLI argument parser handling engine flags (--shot, --size, --scale, --seconds, etc.) and storing game flags. | `EngineArgs` |
| `engine/src/gpu.rs` | WGPU context manager: adapter selection, device/queue creation, surface format configuration, and vsync toggling. | `Gpu`, `DEPTH_FORMAT` |
| `engine/src/input.rs` | Input state tracker managing keyboard keys, mouse buttons, cursor motion delta, scroll, and capture. | `Input` |
| `engine/src/lib.rs` | Rotland custom wgpu engine root: exports application runner, GPU context, render targets, UI layer, and input. | `check_display_change`, `run`, `DisplayChange`, `DisplayRequest`, `Game`, `GameStats` (+6 more) |
| `engine/src/render/camera.rs` | 3D camera projection and view matrix calculator supporting perspective, FOV, and aspect ratio. | `Camera`, `view_matrix()`, `ortho()` |
| `engine/src/render/frustum.rs` | Camera view frustum plane extractor and bounding box intersection tester for GPU draw call culling. | `Frustum` |
| `engine/src/render/lit.rs` | Lit forward mesh renderer drawing static chunk geometry and prop instances with directional sun lighting. | `Lighting`, `SpotLight`, `SpotLightUniform`, `srgb_to_linear()`, `DrawItem`, `FrameCounts` (+1 more) |
| `engine/src/render/mesh.rs` | GPU vertex and index buffer allocations, mesh uploading, and draw call batching primitives. | `COLOR_VERTEX_LAYOUT`, `GpuMesh`, `PackVertex as ColorVertex` |
| `engine/src/render/mod.rs` | Renderer module root: re-exports camera, frustum, lighting, pipelines, shadows, and skinned meshes. | `Camera`, `DrawItem`, `FrameCounts`, `Lighting`, `LitRenderer`, `SpotLight` (+5 more) |
| `engine/src/render/pipelines.rs` | WGPU render pipelines: `mesh` uses CW fronts for terrain; `skinned` uses CCW fronts for glTF and procedural models. | `Layouts`, `Pipelines`, `depth_state()` |
| `engine/src/render/shadow.rs` | Cascaded / directional sun shadow map renderer computing depth orthographic projection and shadow pass. | `SHADOW_FORMAT`, `ShadowSettings`, `ShadowMap`, `light_view_proj()` |
| `engine/src/render/skinned.rs` | Skinned mesh renderer uploading bone matrix palettes and rendering animated humanoid/zombie models. | `SkinInstance`, `SkinnedBatch`, `GrowBuffer` |
| `engine/src/stats.rs` | Frame-time statistical aggregator tracking frame times, average FPS, percentiles (p50/p95/p99), and worst 1%. | `FrameStats`, `StatsSummary` |
| `engine/src/target.rs` | Offscreen HDR/LDR render targets, depth buffer management, and fullscreen upscale/downscale blit pipeline. | `FrameTarget`, `COLOR_FORMAT`, `Blit` |
| `engine/src/timing.rs` | High-precision frame timer tracking delta time, total elapsed duration, and tick counting. | `FrameTimer` |
| `engine/src/ui.rs` | egui 0.37 integration layer rendering UI passes on top of wgpu and winit. | `UiLayer` |

### Crate `humanoid`

Humanoid skeletal animation blending, state machines, one-shot actions, and weapon attachments. (5 files)

| File | What it does | Key pub items |
|---|---|---|
| `humanoid/src/action.rs` | One-shot action animation layer manager (gun bash, melee swing, reload, tool strike) with cross-fade blending. | `ActionState` |
| `humanoid/src/animator.rs` | Humanoid skeletal animator state machine: blends idle/walk/run, jump air, crouch, and layers one-shot actions. | `HumanoidAnimator` |
| `humanoid/src/blend.rs` | Locomotion speed thresholds, blend weight calculations, and skeletal joint transform interpolation. | `WALK_BLEND`, `WALK_CLIP_SPEED`, `RUN_BLEND`, `RUN_CLIP_SPEED`, `BLEND_RATE`, `ACTION_FADE_IN` (+9 more) |
| `humanoid/src/lib.rs` | Humanoid animation crate root: exports HumanoidAnimator, HumanoidState, and weapon attachment utilities. | `ActionState`, `HumanoidAnimator`, `locomotion_weights`, `tempo_for`, `Locomotion`, `LocomotionWeights` (+15 more) |
| `humanoid/src/weapon.rs` | Right-hand weapon attachment positioning using weapon hold offsets from data/weapons.json and model markers. | `HAND_GRIP_OFFSET`, `HAND_GRIP_ROT_DEG`, `REST_POSE_DEG`, `weapon_hand_offset()`, `weapon_hold_for()`, `weapon_world_matrix()` (+1 more) |

### Crate `game`

Pure, testable gameplay simulation systems: player, inventory, items, combat, zombies, animals, dogs, camp, farming, economy, bounties, vehicles, trees. (221 files)

Integration tests all live in `game/tests/all/` and build as ONE test program (`game/tests/all/main.rs` lists them with `mod <file>;`). New test file: put it there and add its `mod` line.

| File | What it does | Key pub items |
|---|---|---|
| `game/src/animals/ai.rs` | Wild animal mind and state machine. | `AnimalAIOutput`, `AnimalAI` |
| `game/src/animals/ai_states.rs` | State-specific think methods for AnimalAI. | *(internal)* |
| `game/src/animals/butcher.rs` | Butchering timer for one dead animal. | `AnimalButcher` |
| `game/src/animals/combat.rs` | A boar's fighting logic. | `FIGHT_COOLDOWN`, `FIGHT_TICK`, `AnimalAttackHit`, `AnimalCombat` |
| `game/src/animals/config.rs` | Configuration and species data parsed from `data/animals.json`. | `AnimalSpeciesData`, `AnimalSpawnData`, `AnimalBehaviourData`, `AnimalButcherData`, `AnimalConfigData` |
| `game/src/animals/herd.rs` | A group of animals that spooks together. | `AnimalHerd` |
| `game/src/animals/hunting.rs` | The Hunting skill's effects on animals. | `AnimalHunting` |
| `game/src/animals/instance.rs` | One wild animal instance. | `PARKED_Y`, `AnimalInstance` |
| `game/src/animals/loot.rs` | What a butchered animal gives (item id and count). | `MEAT_ITEM`, `HIDE_ITEM`, `ANTLER_ITEM`, `AnimalLoot` |
| `game/src/animals/manager.rs` | Top-level Animals system managing active animals, pool, spawning, hits, and butchering. | `AnimalDrawState`, `AnimalFrame`, `AnimalUpdateContext`, `Animals` |
| `game/src/animals/manager_update.rs` | Update simulation step for Animals. | *(internal)* |
| `game/src/animals/mod.rs` | Wild animals and hunting logic ported from scripts/Animals/*.cs. | `AnimalAI`, `AnimalAIOutput`, `AnimalButcher`, `AnimalAttackHit`, `AnimalCombat`, `FIGHT_COOLDOWN` (+34 more) |
| `game/src/animals/motion.rs` | An animal's walking and motion using CharacterMover. | `STOP_DISTANCE`, `GRAVITY`, `AnimalMotion` |
| `game/src/animals/pool.rs` | Object pooling for AnimalInstance by model id. | `AnimalPool` |
| `game/src/animals/rng.rs` | Deterministic pseudo-random number generator for animals. | `AnimalRng` |
| `game/src/animals/senses.rs` | When a wild animal notices the player. | `AnimalSenses` |
| `game/src/animals/spawner.rs` | Animal spawning logic matching AnimalSpawner.cs. | `MAX_SLOPE_DEG`, `SLOPE_PROBE_DIST`, `FLOOR_OFFSET`, `HERD_SPACING`, `SPAWN_ATTEMPTS_PER_TICK`, `try_ground_at()` (+3 more) |
| `game/src/animals/state.rs` | Animal state enum. | `AnimalState` |
| `game/src/bounties/board.rs` | The notice board in camp (port of scripts/Bounties/BountyBoard.cs). | `BountyBoard` |
| `game/src/bounties/config.rs` | One bounty template loaded from data/bounties.json. | `KILL_ZOMBIES`, `KILL_AT_NIGHT`, `CLEAR_POI`, `LOOT_CRATES`, `DELIVER_ITEMS`, `HUNT_ANIMALS` (+4 more) |
| `game/src/bounties/def.rs` | One bounty on the board (offered) or in hand (active). | `BountyReward`, `BountyAcceptError`, `BountyClaimError`, `BountySaveError`, `BountyDef` |
| `game/src/bounties/manager.rs` | Manages offered bounties, accepted active bounties, claimed bounties, and progress tracking. | `BountyManager` |
| `game/src/bounties/mod.rs` | Module root and symbol re-exports for the bounties subsystem. | `BountyBoard`, `BountyConfig`, `BountyConfigData`, `BountyLoadError`, `BountyTemplateData`, `CLEAR_POI` (+12 more) |
| `game/src/bounties/progress.rs` | Plain call when a zombie is killed. | *(internal)* |
| `game/src/bounties/roller.rs` | Pure rules for the board: which bounty day it is, and what the daily and weekly lists hold. | `BountyRoller` |
| `game/src/bounties/save.rs` | Serializes a BountyDef into a Godot save dictionary format. | `to_save_bounty()`, `from_save_bounty()`, `to_save_manager()`, `from_save_manager()` |
| `game/src/camera_rig.rs` | Orbiting third-person / first-person camera rig, ported from Godot's CameraRig.cs. | `MOUSE_SENSITIVITY`, `MIN_ZOOM`, `MAX_ZOOM`, `DEFAULT_ZOOM`, `ZOOM_STEP`, `TRANSITION_DURATION` (+13 more) |
| `game/src/camp/chat.rs` | Chat and speech bubble logic for camp residents (port of scripts/Settlement/CampChat.cs and CampChatBubble.cs). | `CampChatBubble`, `CampChat` |
| `game/src/camp/config.rs` | Configuration and data loader for camp life (port of scripts/Settlement/CampLifeConfig.cs and CampLifeData.cs). | `CampLifeData`, `CampLifeConfig` |
| `game/src/camp/director.rs` | Camp life director coordinating residents, chats, and guards (port of scripts/Settlement/CampLifeDirector.cs). | `CampLifeDirector` |
| `game/src/camp/guard.rs` | Guard behavior and state on settlement platforms (port of scripts/Settlement/SettlementGuard.cs). | `SettlementGuard` |
| `game/src/camp/math.rs` | Math helpers for camp life rules (port of scripts/Settlement/CampMath.cs). | `CampMath` |
| `game/src/camp/mod.rs` | Camp life and settlement logic (port of scripts/Settlement/). | `CampChat`, `CampChatBubble`, `CampLifeConfig`, `CampLifeData`, `CampLifeDirector`, `SettlementGuard` (+15 more) |
| `game/src/camp/npc.rs` | Settlement NPC representations and interaction prompts (port of scripts/Settlement/SettlementNpc.cs). | `prompt_for_role()`, `SettlementNpc` |
| `game/src/camp/planner.rs` | Camp activity scheduler and job assigner (port of scripts/Settlement/CampPlanner.cs). | `CampPlanner` |
| `game/src/camp/resident.rs` | Camp resident state and movement (port of scripts/Settlement/CampResident.cs). | `ResidentOutput`, `CampResident` |
| `game/src/camp/types.rs` | Core types and enums for camp life (port of scripts/Settlement/Camp*.cs). | `CampState`, `CampJob`, `CampEvent`, `CampNext`, `CampClock` |
| `game/src/camp/watcher.rs` | Settlement visit tracking and respawn camp persistence (port of scripts/Settlement/SettlementWatcher.cs). | `SettlementWatcher` |
| `game/src/camp_guards/config.rs` | Configuration data for camp guards (loaded from data/settlements.json). | `CampGuardConfig` |
| `game/src/camp_guards/guards.rs` | Guard shooting logic and state machine (port of scripts/Settlement/SettlementGuard.cs). | `GuardState`, `CampGuards` |
| `game/src/camp_guards/mod.rs` | Camp guards shooting system (port of SettlementGuard.cs and GuardRifle.cs). | `*` |
| `game/src/camp_guards/types.rs` | Types for camp guards and shooting frame output. | `GuardPost`, `GuardTarget`, `GuardShot`, `GuardFrame`, `GuardRifle` |
| `game/src/campfire/fire.rs` | Total in-game burn hours for a fueled campfire. | `BURN_HOURS`, `WOOD_ID`, `ROOT_LIFT`, `SECONDS_PER_HOUR`, `CampfireInteractResult`, `Campfire` (+1 more) |
| `game/src/campfire/manager.rs` | Checks if `pos` is within `CAMPFIRE_RANGE` (4.0m flat) of any lit campfire. | `near_campfire()`, `find_lit_near()`, `settlement_campfire_from_markers()`, `CampfireManager` |
| `game/src/campfire/mod.rs` | Campfire system: campfire instances, placement, burning logic, visuals, and persistence. | `Campfire`, `CampfireInteractResult`, `BURN_HOURS`, `CAMPFIRE_RANGE`, `ROOT_LIFT`, `SECONDS_PER_HOUR` (+22 more) |
| `game/src/campfire/placer.rs` | Distance in front of the player where the campfire is placed. | `PLACE_DISTANCE`, `CAMPFIRE_SPACING`, `CAMPFIRE_KIT_ID`, `PlaceCampfireError`, `is_flat()`, `place_campfire()` |
| `game/src/campfire/save.rs` | Godot SaveId for WorldState, which holds placed campfires. | `WORLDSTATE_SAVE_ID`, `CAMPFIRES_KEY`, `StoredCampfire`, `world_to_chunk()`, `to_save_campfires()`, `from_save_campfires()` |
| `game/src/campfire/visuals.rs` | Base light energy in Godot units. | `BASE_LIGHT_ENERGY`, `RING_RADIUS`, `WARM_LIGHT_COLOR`, `OMNI_RANGE`, `CampfireVisuals` |
| `game/src/crafting/mod.rs` | Module root and symbol re-exports for the crafting subsystem. | `RecipeDef`, `RecipeItem`, `RecipeXp`, `CAMPFIRE_RANGE`, `CAMPFIRE_STATION`, `HAND_STATION` (+8 more) |
| `game/src/crafting/recipe.rs` | Metres from a lit campfire that a campfire craft allows. | `HAND_STATION`, `CAMPFIRE_STATION`, `CAMPFIRE_RANGE`, `RecipeItem`, `RecipeXp`, `RecipeDef` |
| `game/src/crafting/registry.rs` | Registry of all recipes loaded from data/recipes.json. | `RecipeLoadError`, `RecipeRegistry` |
| `game/src/crafting/system.rs` | Information on craft progress during an active craft. | `CraftProgress`, `CraftFinishResult`, `CancelReason`, `CraftCancelResult`, `CraftUpdate`, `CraftingSystem` |
| `game/src/damage.rs` | One hit on anything with health (port of scripts/Combat/DamageInfo.cs). | `DamageInfo` |
| `game/src/difficulty/config.rs` | Difficulty settings shape from data/difficulty.json (port of scripts/Difficulty/DifficultyConfigData.cs and DifficultyConfig.cs). | `DifficultyPresetData`, `DeathRuleData`, `DifficultyConfigData` |
| `game/src/difficulty/death_rule.rs` | What happens to carried items and equipment on player death. | `DeathRule`, `DeathPenaltyResult`, `tally_inventory()` |
| `game/src/difficulty/mod.rs` | Module root and symbol re-exports for the difficulty subsystem. | `DeathRuleData`, `DifficultyConfigData`, `DifficultyPresetData`, `tally_inventory`, `DeathPenaltyResult`, `DeathRule` (+2 more) |
| `game/src/difficulty/run.rs` | Runtime difficulty applied during gameplay (port of scripts/Difficulty/RunDifficulty.cs). | `RunDifficulty` |
| `game/src/difficulty/settings.rs` | One world's difficulty settings (port of scripts/Difficulty/DifficultySettings.cs). | `DifficultySettings` |
| `game/src/dogs/breeder.rs` | Dog breeder interaction at camp: purchasing a puppy with dollars. | `DogBreeder` |
| `game/src/dogs/combat.rs` | Combat behaviour for dogs: scanning, barking, biting, and taking hits. | `DogBite`, `DogCombat` |
| `game/src/dogs/config.rs` | Configuration data for dogs from data/dog.json. | `DogConfigData` |
| `game/src/dogs/dog.rs` | The pet dog entity representation. | `Dog` |
| `game/src/dogs/dog_update.rs` | Simulation tick, locomotion, and player following for Dog. | *(internal)* |
| `game/src/dogs/food.rs` | Which bag items the dog eats (feedItems in dog.json). | `DogFood` |
| `game/src/dogs/growth.rs` | Growth calculation for breeder puppies over time. | `DogGrowth` |
| `game/src/dogs/manager.rs` | Dogs manager owning the pet dog and wild strays, coordinating commands and state. | `Dogs` |
| `game/src/dogs/manager_update.rs` | Simulation update and stray spawning step for Dogs manager. | *(internal)* |
| `game/src/dogs/mod.rs` | Dogs module for Rotland: companion pet dog, commands, combat, and stray encounters. | `DogBreeder`, `DogBite`, `DogCombat`, `DogConfigData`, `Dog`, `DogFood` (+12 more) |
| `game/src/dogs/mode.rs` | Orders and modes for the dog. | `DogMode` |
| `game/src/dogs/motion.rs` | Dog walking and movement using CharacterMover. | `GRAVITY`, `DogMotion` |
| `game/src/dogs/save.rs` | Serialization and deserialization matching Godot DogManager save keys. | `DOG_SAVE_ID` |
| `game/src/dogs/steering.rs` | Obstacle avoidance steering feelers for dogs. | `DogSteering` |
| `game/src/dogs/stray.rs` | Stray dogs sitting at house/cabin lots, fleeing if rushed, and their spawner. | `DogPoiSite`, `Stray`, `StrayDogSpawner` |
| `game/src/dogs/types.rs` | Frame and draw types for dog rendering and simulation context. | `DogDrawState`, `DogFrame`, `DogUpdateContext` |
| `game/src/economy/bank.rs` | Weight cap for the bank. | `BANK_SAVE_ID`, `BANK_MAX_WEIGHT`, `DEFAULT_BANK_SLOTS`, `Bank` |
| `game/src/economy/config.rs` | One line of the trader's full stock in data/economy.json. | `StockEntry`, `EconomyLoadError`, `EconomyConfigData`, `EconomyConfig` |
| `game/src/economy/mod.rs` | Module root and symbol re-exports for the economy subsystem. | `Bank`, `BANK_MAX_WEIGHT`, `BANK_SAVE_ID`, `DEFAULT_BANK_SLOTS`, `EconomyConfig`, `EconomyConfigData` (+14 more) |
| `game/src/economy/rules.rs` | Why one unit cannot be bought right now. | `BuyBlocker`, `WeaponUnloader`, `buy_price()`, `sell_price()`, `buy_blocker()`, `buy()` (+2 more) |
| `game/src/economy/trader.rs` | What the trader has in stock to sell. | `TRADER_SAVE_ID`, `TraderStock` |
| `game/src/economy/wallet.rs` | The player's cash wallet. | `WALLET_SAVE_ID`, `Wallet` |
| `game/src/farming/actions.rs` | Tills a new bed at position on flat ground. | *(internal)* |
| `game/src/farming/config.rs` | Farming XP gained for each player action. | `FarmXpData`, `FarmConfigData`, `FarmLoadError`, `FarmConfig` |
| `game/src/farming/def.rs` | How far a crop has grown. | `CropStage`, `CropDef`, `PlotAction`, `TillError`, `PlotView` |
| `game/src/farming/manager.rs` | Owns every bed, the growth clock, charges, and farming rules. | `FarmManager` |
| `game/src/farming/mod.rs` | Module root and symbol re-exports for the farming subsystem. | `FarmConfig`, `FarmConfigData`, `FarmLoadError`, `FarmXpData`, `CropDef`, `CropStage` (+7 more) |
| `game/src/farming/record.rs` | The state of one bed: position, crop planted, growth hours, and watering status. | `PlotRecord` |
| `game/src/farming/rules.rs` | Pure farming rules: stages, watering, yield, spacing, and ground checks. | `FarmRules` |
| `game/src/farming/save.rs` | Godot save ID for the farming system. | `SAVE_ID`, `to_save_plot()`, `from_save_plot()`, `to_save_manager()`, `from_save_manager()` |
| `game/src/farming/trample.rs` | True when any field bed has a planted crop (camp beds are immune). | *(internal)* |
| `game/src/flowfield.rs` | Flow field: one shared search from the target (the player) gives every chasing zombie the way to walk around trees, rocks and buildings. | `FlowField` |
| `game/src/flowfield_scan.rs` | Works out which flow-field cells a walker can stand in, by asking the physics world about each cell centre. | `MAX_FLOOR_ANGLE_DEG`, `cell_centre()`, `scan_blocked()` |
| `game/src/guns.rs` | Gun state for one held gun: rounds in the magazine, fire cooldown and reloading. | `ReloadDone`, `Gun` |
| `game/src/guns_fire.rs` | Where a shot goes: spread rays from the aim direction, and what each ray hits first. | `GunRng`, `Ray`, `GunTarget`, `GunHit`, `fire_rays()`, `shot_noise()` (+1 more) |
| `game/src/health.rs` | Why a hit did nothing. | `IgnoredReason`, `DamageResult`, `HealthEvent`, `Health` |
| `game/src/hints/config.rs` | Hint configuration data loader and validator. | `HintConfigData`, `HintConfig` |
| `game/src/hints/context.rs` | Environmental context passed to HintSystem::tick to evaluate dynamic triggers. | `HintContext` |
| `game/src/hints/def.rs` | One hint definition from data/hints.json. | `HintDef` |
| `game/src/hints/mod.rs` | Hint and onboarding tip system (port of scripts/UI/ hint logic). | `HintConfig`, `HintConfigData`, `HintContext`, `HintDef`, `HintSystem`, `GAP_SECONDS` (+8 more) |
| `game/src/hints/system.rs` | First-time tips system: trigger polling, sequential queuing, and display lifetime. | `POLL_SECONDS`, `GAP_SECONDS`, `HintSystem` |
| `game/src/hints/triggers.rs` | Hint trigger names, parsing, and validation vocabulary. | `TRIGGER_NAMES`, `NUMBER_TRIGGERS`, `DONE_NAMES`, `split()`, `parse_number()`, `split_npc()` (+1 more) |
| `game/src/input.rs` | Rebindable key and button bindings, matching Godot's `project.godot` default physical keys. | `InputBindings`, `Actions`, `*` |
| `game/src/items/def.rs` | Categories of items in Rotland (matches scripts/Items/ItemRegistry.cs Categories). | `ItemCategory`, `ItemEffects`, `ItemDef` |
| `game/src/items/hotbar.rs` | The first six backpack slots, bound to keys 1–6 (from scripts/Player/Hotbar.cs). | `HOTBAR_SIZE`, `HotbarAction`, `Hotbar` |
| `game/src/items/inventory.rs` | The player's bag: fixed slots holding stacks of items, limited by weight (port of scripts/Items/Inventory.cs). | `Inventory` |
| `game/src/items/mod.rs` | Module root and symbol re-exports for the items subsystem. | `ItemCategory`, `ItemDef`, `ItemEffects`, `Hotbar`, `HotbarAction`, `HOTBAR_SIZE` (+15 more) |
| `game/src/items/registry.rs` | Registry of all item definitions from data/items.json (port of scripts/Items/ItemRegistry.cs). | `ItemLoadError`, `ItemRegistry` |
| `game/src/items/save.rs` | One stack entry in save data (port of scripts/Items/InventorySave.cs). | `SavedSlot`, `InventorySaveData` |
| `game/src/items/stack.rs` | Default backpack slot count (24 slots, from scripts/Items/Inventory.cs). | `DEFAULT_SLOT_COUNT`, `DEFAULT_MAX_WEIGHT`, `WEIGHT_EPSILON`, `ItemStack`, `InventoryEvent` |
| `game/src/items/start.rs` | One line of data/start_inventory.json (port of scripts/Items/StartItemEntry.cs). | `StartItemEntry`, `StartInventoryLoadError`, `load_start_inventory_entries()`, `start_inventory()`, `start_inventory_from_path()` |
| `game/src/keybinds/actions.rs` | List of all 31 bindable actions and their Godot defaults. | `BindableAction`, `KEYBINDS_ALL_ACTIONS`, `find_action()` |
| `game/src/keybinds/keycode_map.rs` | Two-way mapping between Godot key codes (including Shift modifier mask) and winit KeyCode. | `KEYBINDS_SHIFT_MASK`, `KEYBINDS_CODE_MASK`, `godot_code_to_key()`, `key_to_godot_code()`, `describe_godot_code()` |
| `game/src/keybinds/mod.rs` | Key rebinding logic matching Godot's `scripts/Core/KeyBindings.cs`. | `KeyBindings`, `find_action`, `BindableAction`, `KEYBINDS_ALL_ACTIONS`, `describe_godot_code`, `godot_code_to_key` (+3 more) |
| `game/src/ladders/builder.rs` | Prefix for ladder bottom markers. | `ladders_from_markers()` |
| `game/src/ladders/climb.rs` | Input for climbing ladders each frame. | `ClimbInput`, `ClimbPhase`, `CLIMB_SPEED`, `TRANSITION_SECONDS`, `START_GRACE`, `SETTLE_LIFT` (+2 more) |
| `game/src/ladders/end.rs` | The interactable target at one end of a ladder. | `LadderEnd` |
| `game/src/ladders/mod.rs` | Ladders system: rails, ends, model marker pairing, and player climb state machine. | `ladders_from_markers`, `ClimbInput`, `ClimbPhase`, `LadderClimb`, `CLIMB_SPEED`, `GRAB_RANGE` (+6 more) |
| `game/src/ladders/rail.rs` | One ladder: its foot and deck positions (from `ladder_bottom*` / `ladder_top*` model markers). | `LadderRail` |
| `game/src/ladders/tick.rs` | Updates ladder interaction and climbing each frame before normal player move. | `climb_tick()` |
| `game/src/lib.rs` | # game Rotland gameplay logic: player controller, camera rig, input actions, stamina, health and weapon data. | `*`, `DamageInfo`, `DamageResult`, `Health`, `HealthEvent`, `IgnoredReason` (+21 more) |
| `game/src/loot/difficulty.rs` | Re-exports difficulty types from `crate::difficulty` to avoid code duplication. | `DeathRuleData`, `DifficultyConfigData`, `DifficultyPresetData` |
| `game/src/loot/mod.rs` | Module root and symbol re-exports for the loot subsystem. | `DeathRuleData`, `DifficultyConfigData`, `DifficultyPresetData`, `crate_seed`, `roll`, `roll_cash` (+10 more) |
| `game/src/loot/roller.rs` | Bit-exact port of C# LootRoller (scripts/World/LootRoller.cs). | `LootRoller`, `crate_seed()`, `scale_rolls()`, `roll()`, `roll_cash()` |
| `game/src/loot/state.rs` | Tracks which crates have been looted across the world and per chunk (port of WorldState._looted). | `LootState` |
| `game/src/loot/table.rs` | One weighted line of a loot table: an item and how many of it a roll can give (port of scripts/World/LootEntryData.cs). | `LootLoadError`, `LootEntryData`, `LootCashData`, `LootTableData`, `LootTableRegistry` |
| `game/src/loot/world_items.rs` | An item lying on the ground (port of scripts/World/DroppedItem.cs and scripts/Items/WorldItem.cs). | `WorldItem`, `WorldItems` |
| `game/src/map/buildings.rs` | Building discovery and map notes (port of scripts/World/WorldMap.cs NoteNearbyBuildings). | `PoiSiteInfo` |
| `game/src/map/compass.rs` | Pure compass mathematics: bearings, relative angles, marker strip positioning, and cardinal/ordinal direction naming. | `HALF_VIEW_DEG`, `relative_angle_deg()`, `bearing_to_view_deg()`, `marker_x()`, `direction_name()` |
| `game/src/map/explore.rs` | Explored-area memory: which 8m map cells the player has seen. | `CELL_SIZE`, `CELLS_PER_CHUNK`, `MapExplore` |
| `game/src/map/icons.rs` | Map icon kinds and color constants. | `MapIconKind`, `OUTLINE_COLOR`, `CAMP_COLOR`, `BUILDING_COLOR`, `DIM_COLOR`, `BOUNTY_COLOR` (+3 more) |
| `game/src/map/markers.rs` | Map marker collection and building letter helpers. | `MapMarkerKind`, `MapMarker`, `letter_for()`, `collect_markers()` |
| `game/src/map/math.rs` | Pure map mathematics: world to screen projection, screen to world projection, and zoom stepping. | `SCALES`, `MAP_DEFAULT_ZOOM`, `world_to_screen()`, `screen_to_world()`, `step_zoom()` |
| `game/src/map/mod.rs` | Map, compass markers, and exploration system (port of scripts/UI/ and scripts/World/ map logic). | `*`, `bearing_to_view_deg`, `direction_name`, `marker_x`, `relative_angle_deg`, `HALF_VIEW_DEG` (+29 more) |
| `game/src/map/state.rs` | Map state: explored cells, found buildings, last death spot, and player waypoint. | `MapBuilding`, `MapState` |
| `game/src/map/tiles.rs` | Chunk tile rendering: height and biome based procedural colors. | `TEXELS_PER_CHUNK`, `CLIFF_SLOPE`, `TEXEL_METRES`, `UNEXPLORED_COLOR`, `CLIFF_COLOR`, `render_chunk_tiles()` |
| `game/src/melee.rs` | Melee swings (port of scripts/Combat/MeleeHandler.cs and scripts/Combat/GunBash.cs). | `Attacker`, `MeleeTarget`, `MeleeHit`, `SwingState`, `MeleeSwing`, `gun_bash()` |
| `game/src/player.rs` | Stamina pool for sprinting and jumping (from Stamina.cs and PlayerStaminaUse.cs). | `WALK_SPEED`, `SPRINT_SPEED`, `CROUCH_SPEED`, `JUMP_VELOCITY`, `ACCELERATION`, `DECELERATION` (+11 more) |
| `game/src/poi_zombies/manager.rs` | POI zombie spawner and building clear tracker. | `DEFAULT_APPROACH_DISTANCE`, `DEFAULT_MAX_ACTIVE_ZOMBIES`, `DEFAULT_DESPAWN_DISTANCE`, `PoiZombies` |
| `game/src/poi_zombies/mod.rs` | Module root and symbol re-exports for the poi_zombies subsystem. | `PoiZombies`, `DEFAULT_APPROACH_DISTANCE`, `DEFAULT_DESPAWN_DISTANCE`, `DEFAULT_MAX_ACTIVE_ZOMBIES`, `from_save_visited_pois`, `to_save_visited_pois` (+4 more) |
| `game/src/poi_zombies/save.rs` | Persistence for visited POI buildings. | `POI_ZOMBIES_SAVE_ID`, `POI_ZOMBIES_SAVE_KEY`, `to_save_visited_pois()`, `from_save_visited_pois()` |
| `game/src/poi_zombies/types.rs` | Lightweight description of a POI building site for zombie wake and spawn checks. | `PoiZombieSite`, `PoiSpawn` |
| `game/src/profile_stats/mod.rs` | Lifetime statistics for the player profile. | `STATS_MAX_STEP_METRES`, `STATS_SAVE_ID`, `STATS_LOAD_ORDER`, `ProfileStats` |
| `game/src/skills/def.rs` | Stats and scaling for one skill (port of scripts/Skills/SkillDef.cs). | `SkillDef` |
| `game/src/skills/mod.rs` | Module root and symbol re-exports for the skills subsystem. | `SkillDef`, `SkillRegistry`, `combat_xp_for_kill`, `AgilityTracker`, `HEADSHOT_MULTIPLIER`, `METRES_PER_AGILITY_XP` (+2 more) |
| `game/src/skills/registry.rs` | Registry of all skills loaded from data/skills.json (port of scripts/Skills/SkillRegistry.cs). | `SkillRegistry` |
| `game/src/skills/rewards.rs` | Headshot kill XP multiplier (port of scripts/Skills/XpRewards.cs HeadshotMultiplier). | `HEADSHOT_MULTIPLIER`, `METRES_PER_AGILITY_XP`, `combat_xp_for_kill()`, `AgilityTracker` |
| `game/src/skills/system.rs` | Current level and partial XP inside the level for one skill. | `SkillProgress`, `SkillSystem` |
| `game/src/spawner.rs` | Zombie spawning logic in the open world, ported from scripts/Enemies/ZombieSpawner.cs. | `MAX_SPAWN_SLOPE_DEG`, `SLOPE_PROBE_DIST`, `CLEARANCE_RADIUS`, `CLEARANCE_HEIGHT`, `FLOOR_OFFSET`, `SPAWN_ATTEMPTS_PER_TICK` (+9 more) |
| `game/src/spawner_config.rs` | Configuration data and order types for the zombie spawner, loaded from data/spawning.json. | `SpawnConfig`, `SpawnConfigData`, `CampDensityRule`, `SpawnTypeTable`, `SpawnTypeEntry`, `SpawnerRng` (+3 more) |
| `game/src/survival/config.rs` | Rates and thresholds for one survival need (port of scripts/Survival/NeedDef.cs). | `NeedDef`, `StaminaDef`, `WellFedDef`, `SurvivalData` |
| `game/src/survival/mod.rs` | Module root and symbol re-exports for the survival subsystem. | `NeedDef`, `StaminaDef`, `SurvivalData`, `WellFedDef`, `NeedStat`, `apply_item_effects` (+2 more) |
| `game/src/survival/need.rs` | One survival need (hunger or thirst), ported from scripts/Survival/NeedStat.cs. | `NeedStat` |
| `game/src/survival/well_fed.rs` | Well-fed regeneration logic (port of scripts/Survival/WellFedRegen.cs). | `WellFedRegen`, `apply_item_effects()`, `update_stamina_regen_multiplier()` |
| `game/src/trees/falling.rs` | Falling tree animation state. | `TREE_FALL_DURATION`, `FallingTree` |
| `game/src/trees/felling.rs` | Tree felling manager. | `TREE_FELL_HITS`, `TREE_WOOD_MIN`, `TREE_WOOD_MAX`, `TREE_DROP_OFFSET`, `TREE_ITEM_HALF_HEIGHT`, `WOOD_ITEM_ID` (+3 more) |
| `game/src/trees/mod.rs` | Tree felling system: axe strikes, felling countdown, fall animation, and wood rewards. | `FallingTree`, `TREE_FALL_DURATION`, `TreeFelling`, `TreeStrikeResult`, `HIT_WOOD_SOUND`, `TREE_DROP_OFFSET` (+15 more) |
| `game/src/trees/save.rs` | Persistence for fallen trees. | `TREE_WORLDSTATE_SAVE_ID`, `TREE_FALLEN_KEY`, `to_save_fallen_trees()`, `from_save_fallen_trees()` |
| `game/src/trees/strike.rs` | Tree spatial query and identification. | `TreeHit`, `TreeCandidate`, `is_tree_kind()`, `tree_id()`, `parse_tree_id()`, `find_tree_strike()` |
| `game/src/vehicles/body.rs` | 4WD off-road body physics: suspension, anti-roll, hill-hold, dual-height crash sweeps, and flip recovery. | `HUB_REST`, `CarBody` |
| `game/src/vehicles/camera.rs` | Multi-mode Chase/Close/Hood `CarCamera` with dynamic FOV, orbit, and shake. | `CHASE_DISTANCE`, `MIN_DISTANCE`, `PIVOT_HEIGHT`, `SMOOTHING`, `ORBIT_MOUSE_RAD_PER_PIXEL`, `ORBIT_KEY_RAD_PER_SECOND` (+3 more) |
| `game/src/vehicles/car.rs` | Vehicle simulation and animated draw state for wheels, driver door, and tailgate. | `WHEEL_RADIUS`, `NOISE_INTERVAL`, `CarDrawState`, `CarStepEvents`, `Car`, `car_tag()` |
| `game/src/vehicles/config.rs` | Shape of data/vehicles.json (port of scripts/Vehicles/VehicleConfigData.cs). | `VehicleConfigData` |
| `game/src/vehicles/controls.rs` | Driver inputs for throttle, steering, braking, lights, horn, and `[R]` flip recovery. | `CarControls`, `CarInput` |
| `game/src/vehicles/fuel.rs` | The petrol tank: litres in, litres out (port of scripts/Vehicles/CarFuel.cs). | `CarFuel` |
| `game/src/vehicles/gauge.rs` | Formats the driving gauge text (port of scripts/Vehicles/CarGauge.cs). | `describe_gauge()` |
| `game/src/vehicles/health.rs` | Car condition, crash damage, wreck state, and repair health. | `CarHealth` |
| `game/src/vehicles/manager.rs` | Vehicle lifecycle, driving frame events, and `[T]` scrap-metal repairs. | `CarFrame`, `Cars` |
| `game/src/vehicles/mod.rs` | Module root and symbol re-exports for the vehicles subsystem. | `*`, `CAR_SYSTEM_ID` |
| `game/src/vehicles/point.rs` | What an interaction point on a car does (port of scripts/Vehicles/CarPointKind.cs). | `CarPointKind`, `CarPoint` |
| `game/src/vehicles/run_over.rs` | Dual-height swept vehicle collision against targets in the car's path. | `CarRunOver` |
| `game/src/vehicles/save.rs` | Serializes cars data matching Godot CarManager.cs Save (system id "cars"). | `CAR_SYSTEM_ID`, `to_save()`, `from_save()` |
| `game/src/vehicles/spawn_rules.rs` | Stable FNV-1a hash over text, matching Godot C# CarSpawnRules.cs. | `CAMP_CAR_ID`, `SPAWN_SALT`, `COLOUR_SALT`, `FUEL_SALT`, `PARKING_GAP`, `stable_hash()` (+9 more) |
| `game/src/weapons.rs` | Melee weapons swing; guns fire. | `WeaponKind`, `WeaponAnim`, `WeaponHold`, `WeaponScope`, `WeaponDef`, `WeaponRegistry` (+1 more) |
| `game/src/zombie/attack.rs` | The bite timing (port of the timing and reach rules in ZombieAttack.cs). | `BITE_REACH_MARGIN`, `AttackHit`, `ZombieAttack` |
| `game/src/zombie/brain.rs` | The zombie's brain (port of ZombieAI.cs, ZombieMotion.cs, ZombieWounds.cs, ZombieCrawl.cs). | `BrainInput`, `AnimClip`, `BrainOutput`, `Numbers`, `Step`, `ZombieBrain` |
| `game/src/zombie/brain_actions.rs` | State execution for ZombieBrain (port of ZombieAI.cs, ZombieAI.Actions.cs and ZombieWounds.cs). | *(internal)* |
| `game/src/zombie/brain_extras.rs` | Extra test hooks and state transitions for ZombieBrain (R3-09 extras). | `IDLE_MIN`, `IDLE_MAX`, `WANDER_MIN`, `WANDER_MAX` |
| `game/src/zombie/corpses.rs` | Remembers where zombies died recently, so an idle zombie nearby may stop to eat (port of ZombieCorpses.cs). | `ZombieCorpses` |
| `game/src/zombie/defs.rs` | Zombie types and behaviour numbers, read from `data/zombies.json` (same key names as the Godot version). | `ZombieDef`, `ZombieBehaviourDef`, `ZombieDefs` |
| `game/src/zombie/herd.rs` | Group behaviours: herd sighting alerts and horde shared wandering (port of ZombieHerd.cs and ZombieHorde.cs). | `HerdZombie`, `find_chasing_near()`, `ZombieHorde` |
| `game/src/zombie/hideouts.rs` | Checks if a target is out of reach in a hideout (roof or ladder) to trigger banging (port of ZombieHideouts.cs). | `is_under_roof()` |
| `game/src/zombie/mod.rs` | Zombie logic, pure (no GPU): data from `data/zombies.json`, state, senses and bite timing. | `AttackHit`, `ZombieAttack`, `AnimClip`, `BrainInput`, `BrainOutput`, `ZombieBrain` (+12 more) |
| `game/src/zombie/rng.rs` | Deterministic random numbers for one zombie's brain. | `Rng` |
| `game/src/zombie/senses.rs` | Sight, hearing and memory for one zombie (port of ZombieSenses.cs and the hearing rule in Zombie.cs). | `CHECK_INTERVAL`, `ALWAYS_SEE_RANGE`, `EYE_HEIGHT`, `CHEST_HEIGHT`, `Noise`, `hears()` (+1 more) |
| `game/src/zombie/state.rs` | What a zombie is doing right now (port of ZombieState.cs). | `ZombieState` |
| `game/src/zombie/steering.rs` | Obstacle avoidance and stuck escape for zombies (port of ZombieSteering.cs). | `FEELER_LENGTH`, `FEELER_ANGLE_DEG`, `KNEE_HEIGHT`, `SIDESTEP_MEMORY`, `ARRIVE_DISTANCE`, `GROUND_PROBE_ABOVE` (+14 more) |
| `game/src/zombie_voices/mod.rs` | Zombie voices module (groans and attack snarls). | `*` |
| `game/src/zombie_voices/system.rs` | Zombie vocalizations logic (port of scripts/Audio/ZombieVoices.cs). | `SCAN_INTERVAL`, `MIN_GROAN_GAP`, `MAX_GROAN_GAP`, `AUDIBLE_RANGE`, `AUDIBLE_RANGE_SQ`, `MAX_VOICES` (+4 more) |
| `game/src/zombie_voices/types.rs` | Types for zombie vocalizations (ports of VoiceZombie, VoiceState, VoiceEvent). | `VoiceState`, `VoiceZombie`, `VoiceEvent` |
| `game/tests/all/animals_tests.rs` | Tests for animals functionality. | `test_data_loads_completely()`, `test_spawn_picks_on_real_data_for_fixed_seed()`, `test_flee_moves_away_from_player()`, `test_boar_charges_and_hits()` |
| `game/tests/all/bounties_tests.rs` | Tests for bounties functionality. | `bounties_rolls_3_fixed_seeds_stable_and_valid()`, `bounties_accept_progress_complete_reward()`, `bounties_expiry()`, `bounties_board_refresh_per_day()` |
| `game/tests/all/camp_guards_tests.rs` | Integration tests for camp guards shooting logic (port of SettlementGuard.cs). | `test_config_values_match_data_file()`, `test_zombie_in_range_shot_at_fire_rate()`, `test_out_of_range_zombie_not_shot()`, `test_blocked_by_obstacle_not_shot()` |
| `game/tests/all/camp_tests.rs` | Integration tests for camp life and settlement NPCs (tasks/R5-04-camp-life.md). | `camp_life_config_loads_and_validates_repo_file()`, `settlement_guard_scans_targets_and_fires()`, `residents_planned_3_hours_follow_schedule_and_count_jobs()`, `chat_picks_are_stable_for_fixed_seed()` |
| `game/tests/all/campfire_tests.rs` | Tests for campfire functionality. | `test_placing_campfire_on_flat_ground_refused_on_steep_or_too_close()`, `test_fuel_burns_down_over_seconds_and_hours()`, `test_near_campfire_radius()`, `test_settlement_fixed_campfire()` |
| `game/tests/all/combat_tests.rs` | f32 fields compared with a tolerance: 0.15 is not exact in f32, so exact equality would fail. | `damage_reduces_health()`, `damage_at_zero_kills_once()`, `damage_is_capped_at_remaining_health()`, `invulnerability_ignores_damage_then_wears_off()` |
| `game/tests/all/crafting_tests.rs` | Tests for crafting functionality. | `recipes_load_from_repo_and_validate()`, `recipe_can_craft_checks_inputs_station_and_alive()`, `crafting_removes_inputs_and_adds_outputs_with_xp()`, `crafting_respects_inventory_weight_and_stack_limits()` |
| `game/tests/all/difficulty_tests.rs` | Tests for difficulty functionality. | `difficulty_config_loads_and_validates()`, `death_rule_lose_all_clears_inventory()`, `death_rule_keep_all_preserves_inventory()`, `death_rule_keep_half_removes_half_stacks()` |
| `game/tests/all/dogs_tests.rs` | Tests for dogs functionality. | `test_data_loads_completely()`, `test_following_dog_walks_toward_player()`, `test_dog_bites_zombie_in_range()`, `test_feeding_removes_food_and_restores_health()` |
| `game/tests/all/economy_tests.rs` | Tests for economy functionality. | `economy_config_loads_from_repo_and_validates()`, `price_math_on_real_data_counts_and_matches_all_items()`, `wallet_operations_and_death_penalty_hook()`, `buy_and_sell_moves_money_and_items()` |
| `game/tests/all/farming_tests.rs` | Tests for farming functionality. | `test_grow_crop_through_all_stages_on_real_data()`, `test_watering_rules_drying_and_refill()`, `test_harvest_gives_data_items_bonus_and_seed_back()`, `test_trample_field_vs_camp_plots()` |
| `game/tests/all/flowfield_tests.rs` | Flat ground over a 3 x 3 block of chunks (-64 m .. 192 m), so the 128 m grid around (10, 10) is covered. | `open_ground_points_straight_at_target()`, `wall_between_walker_and_target_is_walked_around()`, `walled_in_target_is_unreachable_from_outside()`, `steep_cliff_separates_the_two_sides()` |
| `game/tests/all/gun_tests.rs` | Flat ground over the origin chunk (x and z in 0..64), so shots fired inside it have ground to hit. | `firing_uses_a_round_and_respects_the_fire_rate()`, `an_empty_magazine_cannot_fire()`, `reload_takes_the_csharp_time_and_fills_from_the_bag()`, `reload_is_partial_when_the_bag_is_short()` |
| `game/tests/all/hints_tests.rs` | Tests for hint config data loading, trigger evaluation, sequential queueing/gap cooldown, and save/load. | `hints_tests_data_loads_and_validates()`, `hints_tests_trigger_shows_hint_once()`, `hints_tests_cooldown_and_ordering()`, `hints_tests_save_load_round_trip()` |
| `game/tests/all/items_tests.rs` | Tests for items functionality. | `every_item_in_items_json_loads()`, `stacking_respects_max_stack_and_returns_leftover()`, `weight_limit_refuses_overflow()`, `remove_and_count_work()` |
| `game/tests/all/keybinds_tests.rs` | Integration tests for key rebinding, conflict detection/swapping, prompts, and settings round-trip. | `keybinds_defaults_match_godot_project()`, `keybinds_rebind_changes_action()`, `keybinds_conflict_detects_clash_and_swaps()`, `keybinds_reset_all_restores_defaults()` |
| `game/tests/all/ladders_tests.rs` | Tests for ladders functionality. | `test_marker_pairing_by_suffix_and_missing_top_skipped()`, `test_climbing_up_reaches_the_deck()`, `test_letting_go_falls()`, `test_climb_down_from_deck()` |
| `game/tests/all/loot_tests.rs` | Tests for loot functionality. | `golden_crate_loot_matches_godot_dump()`, `loot_tables_load_and_validate_against_items()`, `crate_seed_is_deterministic_and_matches_dotnet()`, `seed_42_deterministic_roll_matches_traced_numbers()` |
| `game/tests/all/map_buildings_tests.rs` | Tests for POI building discovery on the map and save round-trips. | `map_buildings_noted_once_when_within_reveal_range()`, `map_buildings_looted_status_update_and_markers()`, `map_buildings_kept_after_save_round_trip()`, `map_buildings_note_nearby_poi_sites_scanning()` |
| `game/tests/all/map_tests.rs` | Tests for map math, compass, marker collection/saving, and explored tiles. | `map_tests_pixel_math_round_trip()`, `map_tests_compass_bearing_cardinal_and_ordinal()`, `map_tests_marker_add_remove_save()`, `map_tests_explored_tiles_reveal_and_save()` |
| `game/tests/all/melee_tests.rs` | Flat ground over the origin chunk, so the sight ray has something under the feet. | `every_melee_weapon_in_the_json_is_a_melee_swing()`, `target_in_front_is_hit_once_at_the_impact_time()`, `target_outside_the_arc_or_out_of_range_is_not_hit()`, `max_targets_limits_the_hits_and_keeps_the_nearest()` |
| `game/tests/all/player_tests.rs` | Tests for player functionality. | `test_walking_forward_moves_at_4_mps()`, `test_sprinting_moves_at_7_mps()`, `test_jumping_leaves_the_ground_and_lands()`, `test_player_cannot_walk_through_box()` |
| `game/tests/all/poi_zombies_tests.rs` | Tests for poi_zombies functionality. | `test_no_spawn_far_away()`, `test_spawn_count_markers_near()`, `test_cap_respected()`, `test_cleared_after_all_die_and_never_respawns()` |
| `game/tests/all/profile_stats_tests.rs` | Integration tests for profile stats counters, movement tracking, and save/load compatibility. | `default_stats_match_godot_initial_values()`, `record_kill_and_on_kill_counts_accurately()`, `record_death_and_day_counters()`, `distance_walked_and_driven_counters()` |
| `game/tests/all/skills_tests.rs` | Tests for skills functionality. | `skills_data_loads_and_matches_registry()`, `xp_to_level_thresholds_traced_from_data()`, `skill_bonuses_scale_with_level()`, `max_level_cap_and_excess_xp()` |
| `game/tests/all/spawner_tests.rs` | Flat ground over a 7x7 chunk area centered on (0, 0), so spawns up to 150+ m land on ground. | `spawning_json_loads_correctly_and_completely()`, `missing_key_in_json_is_an_error_not_a_silent_default()`, `spawns_fill_up_to_the_cap_over_time_and_never_exceed_it()`, `cap_is_higher_at_night_and_scaled_by_difficulty()` |
| `game/tests/all/survival_tests.rs` | Tests for survival functionality. | `survival_data_loads_from_real_file()`, `drain_after_n_game_hours_on_real_data()`, `starving_and_dehydration_damage_when_empty()`, `eating_and_drinking_restores_needs()` |
| `game/tests/all/trees_tests.rs` | Integration tests for tree felling, axe strikes, wood rewards, falling animation, and persistence. | `axe_strike_traces_hits_and_fells_tree()`, `bare_hands_does_not_chop_tree()`, `non_axe_weapons_do_not_chop_tree()`, `wood_count_range_and_inventory_addition()` |
| `game/tests/all/vehicles_tests.rs` | Tests for vehicles functionality. | `test_config_loads_completely()`, `test_parked_car_settles_at_equilibrium_ride_height()`, `test_throttle_moves_forward_and_burns_fuel()`, `test_steering_turns_the_car()` |
| `game/tests/all/zombie_brain_tests.rs` | Zombie brain tests (R3-04). | `an_idle_zombie_with_the_player_in_view_chases_toward_them()`, `a_zombie_in_bite_range_bites_once_after_the_windup()`, `a_heard_noise_sends_an_idle_zombie_to_investigate_it()`, `a_zombie_that_loses_the_player_goes_to_the_last_seen_spot_then_gives_up()` |
| `game/tests/all/zombie_extras_tests.rs` | Tests for zombie extras (R3-09): scream, eat, bang, crawler, steering, and stuck escape. | `a_first_sighting_can_trigger_a_scream_and_the_scream_alerts_a_nearby_idle_zombie()`, `a_zombie_near_a_corpse_with_no_target_eats_it()`, `a_player_standing_on_a_high_box_unreachable_gets_banged_for()`, `a_leg_hit_with_enough_damage_makes_a_crawler_that_moves_slower()` |
| `game/tests/all/zombie_tests.rs` | Flat ground over the origin chunk, so the sight ray has something under the feet. | `every_zombie_type_in_the_json_loads()`, `a_missing_key_is_an_error_not_a_silent_default()`, `target_in_front_in_range_with_clear_view_is_seen()`, `target_out_of_sight_range_is_not_seen()` |
| `game/tests/all/zombie_voices_tests.rs` | Integration tests for zombie vocalizations (ports of scripts/Audio/ZombieVoices.cs). | `test_nothing_beyond_the_range()`, `test_chasing_zombie_in_range_groans_after_traced_time()`, `test_cap_holds_with_many_zombies()`, `test_dead_zombies_are_silent()` |

### Crate `rotland`

Game executable and wiring: integrates engine, world, physics, game, humanoid, save, and audio into the playable game. (69 files)

| File | What it does | Key pub items |
|---|---|---|
| `rotland/examples/hud_view.rs` | Stand-alone example rendering and previewing the egui game HUD layout and elements in isolation. | `main()` |
| `rotland/examples/humanoid_view.rs` | Stand-alone example displaying and animating the humanoid character model with clip selection. | `main()` |
| `rotland/examples/pose_check.rs` | CLI validation tool checking humanoid skeleton bone transforms, rest poses, and marker coordinates. | `main()` |
| `rotland/src/bench.rs` | The fixed walk of Godot's forest25 benchmark (BenchmarkPath): forward 5 s, turn 90 degrees over 1 s, repeat. | `SIDE_SECONDS`, `TURN_SECONDS`, `WALK_SPEED`, `yaw_at()` |
| `rotland/src/bubbles.rs` | In-world speech bubble rendering over the 3D view using egui (port of scripts/Settlement/CampChatBubble.cs). | `MAX_BUBBLE_WIDTH`, `BUBBLE_FONT_SIZE`, `BUBBLE_PADDING`, `BUBBLE_Y_OFFSET`, `world_to_screen()`, `draw_bubbles()` |
| `rotland/src/camp.rs` | The camp's systems around the player: wallet, bank, trader stock, skills, bounties and the world's difficulty. | `Camp` |
| `rotland/src/camp_npcs.rs` | The camp's people on screen: the trader, banker, guards and survivors standing at their posts, and the residents walking their daily routine (game::camp). | `CampNpcs` |
| `rotland/src/camp_ui/action.rs` | Actions triggered by user interactions with camp screens. | `CampAction` |
| `rotland/src/camp_ui/draw.rs` | Central draw dispatcher and layout for camp screens (egui 0.37). | `PANEL_WIDTH`, `PANEL_HEIGHT`, `parse_hex_color()`, `draw()` |
| `rotland/src/camp_ui/draw_bank.rs` | egui 0.37 rendering for the Banker screen. | `draw_bank()` |
| `rotland/src/camp_ui/draw_bounties.rs` | egui 0.37 rendering for the Bounty Board screen and on-HUD Bounty Tracker. | `draw_bounties()`, `draw_bounty_tracker()` |
| `rotland/src/camp_ui/draw_crafting.rs` | egui 0.37 rendering for the Crafting panel. | `draw_crafting()` |
| `rotland/src/camp_ui/draw_shop.rs` | egui 0.37 rendering for the Trader Shop screen. | `draw_shop()` |
| `rotland/src/camp_ui/mod.rs` | Helper methods here (count steppers, view getters) are used by the tests and by screens still to come. | `CampAction`, `draw`, `draw_bounty_tracker`, `CampScreen`, `CampUiState`, `CampView` |
| `rotland/src/camp_ui/screen.rs` | Camp screen identifier enum. | `CampScreen` |
| `rotland/src/camp_ui/state.rs` | UI state for camp screens (tab switching, selected row, count steppers, status). | `CampUiState` |
| `rotland/src/camp_ui/tests.rs` | Unit tests for camp facility UI state transitions, bank transfers, trade transactions, and crafting. | `test_tab_switching()`, `test_selected_row()`, `test_count_stepper_basic()`, `test_count_stepper_clamp_to_stock()` |
| `rotland/src/camp_ui/view.rs` | Read-only view struct providing data to camp screens. | `CampView` |
| `rotland/src/campfire_mesh.rs` | The campfire's look, built in code like Godot's CampfireVisuals: a ring of grey stones, two crossed logs, and an orange/yellow flame cone that shows while the fire burns. | `CampfireMeshes` |
| `rotland/src/car_lights.rs` | Car headlights: road-tilted off-road beams from the vehicle headlight markers. | `HEADLIGHT_RANGE`, `HEADLIGHT_CONE_DEG`, `HEADLIGHT_ENERGY`, `HEADLIGHT_COLOR`, `DEFAULT_HEADLIGHT_L`, `DEFAULT_HEADLIGHT_R` (+1 more) |
| `rotland/src/car_visuals.rs` | Car marker loading, CCW emissive light lenses, and pooled exhaust/damage smoke and ember effects. | `CarMarkers`, `SmokeKind` (+ more) |
| `rotland/src/car_solid.rs` | Static collision box management for parked cars. | `default_car_box()`, `ParkedCarSolids` |
| `rotland/src/combat.rs` | The player's side of the fight: health, the melee swing on the left mouse button, being bitten, dying and coming back at the camp. | `Combat`, `CombatEvents`, `MeleeWorld` |
| `rotland/src/crates.rs` | Loot crates and world items: placement and player interaction. | `CRATE_MODEL`, `INTERACT_RANGE`, `HALF_CONE_DEGREES`, `DROP_DISTANCE`, `DROP_LIFT`, `SEARCH_CRATE_PROMPT` (+8 more) |
| `rotland/src/display.rs` | Display settings logic: FOV scaling/application and building engine DisplayRequests. | `DEFAULT_CAMERA_FOV`, `base_fov()`, `apply_fov()`, `display_request()`, `DisplayRequest` |
| `rotland/src/dog_view.rs` | The player's dog (and strays) on screen: game::dogs moves them, this draws them like the animals (one instanced batch per model, each dog with its clip clock) and scales puppies. | `DogView` |
| `rotland/src/fp_weapon.rs` | First-person weapon viewmodel matrix calculation, sway bob/kick, and wall-clip pull. | `FIRST_PERSON_MOUNT`, `FIRST_PERSON_REST_POSE_DEG`, `BOB_AMPLITUDE_X`, `BOB_AMPLITUDE_Y`, `BOB_SPEED`, `GUN_KICK_OFFSET` (+8 more) |
| `rotland/src/hub_ui/action.rs` | Actions triggered by user interactions with the player hub and related screens. | `DogCommand`, `HubAction` |
| `rotland/src/hub_ui/draw.rs` | Central draw dispatcher and layout for player hub and modal screens. | `PANEL_WIDTH`, `PANEL_HEIGHT`, `parse_hex_color()`, `draw()`, `draw_overlays` |
| `rotland/src/hub_ui/draw_bag.rs` | egui 0.37 rendering for the Bag (inventory) tab in the player hub. | `OVERWEIGHT_RATIO`, `OVERWEIGHT_COLOR`, `NORMAL_WEIGHT_COLOR`, `SLOT_SIZE`, `GRID_COLUMNS`, `draw_bag()` |
| `rotland/src/hub_ui/draw_bag_details.rs` | egui 0.37 rendering for the Details subpanel in the Bag tab (InventoryDetailsPanel.cs). | `draw_details_panel()` |
| `rotland/src/hub_ui/draw_bounties.rs` | egui 0.37 rendering for the Bounties tab in the player hub. | `draw_bounties()` |
| `rotland/src/hub_ui/draw_dog.rs` | egui 0.37 rendering for dog interaction screens: DogMenu, DogPackScreen, and DogNameDialog. | `DOG_MENU_WIDTH`, `DOG_MENU_HEIGHT`, `DOG_PACK_WIDTH`, `DOG_PACK_HEIGHT`, `DOG_DIALOG_WIDTH`, `DOG_DIALOG_HEIGHT` (+3 more) |
| `rotland/src/hub_ui/draw_overlays.rs` | egui 0.37 rendering for LevelUpBanner and XpToastStack HUD overlays. | `TOAST_RIGHT_MARGIN`, `TOAST_BOTTOM_MARGIN`, `TOAST_LINE_HEIGHT`, `draw_overlays()` |
| `rotland/src/hub_ui/draw_profile.rs` | egui 0.37 rendering for the Profile tab in the player hub. | `PORTRAIT_WIDTH`, `PORTRAIT_HEIGHT`, `draw_profile()` |
| `rotland/src/hub_ui/draw_skills.rs` | egui 0.37 rendering for the Skills tab in the player hub. | `draw_skills()` |
| `rotland/src/hub_ui/draw_trunk.rs` | egui 0.37 rendering for the Car Trunk storage screen (TrunkScreen.cs). | `TRUNK_PANEL_WIDTH`, `TRUNK_PANEL_HEIGHT`, `draw_trunk()` |
| `rotland/src/hub_ui/mod.rs` | Helper methods and fields here are used by screens, HUD overlays, and tests. | `DogCommand`, `HubAction`, `draw`, `draw_overlays`, `HubScreen`, `HubTab` (+3 more) |
| `rotland/src/hub_ui/screen.rs` | Tab and screen identifiers for player hub and overlay screens. | `HubTab`, `HubScreen` |
| `rotland/src/hub_ui/state.rs` | UI state for player hub and HUD overlays. | `LEVEL_UP_BANNER_DURATION`, `XP_TOAST_LIFETIME`, `XP_TOAST_MERGE_WINDOW`, `LevelUpBannerState`, `XpToast`, `HubUiState` |
| `rotland/src/hub_ui/tests.rs` | Unit tests for player hub menu tab navigation, inventory dragging, and bounty claiming. | `test_tab_order_and_wrap()`, `test_switch_tab_resets_drag_and_status()`, `test_selected_slot_and_moves()`, `test_drag_and_drop_state()` |
| `rotland/src/hub_ui/view.rs` | Read-only view structures providing data to hub screens. | `DogView`, `HubView` |
| `rotland/src/hud.rs` | Player HUD: health, stamina, crosshair, hotbar, compass, clock, FPS, ammo, and death overlay. | `HotbarSlot`, `HudState`, `draw()`, `draw_hint()` |
| `rotland/src/hud/vehicle.rs` | Bottom-right driving dashboard: speed, gear, RPM, fuel/hull bars, status pills, and alerts. | `VehicleHudInfo`, `gear_label()`, `draw_vehicle_hud()` |
| `rotland/src/keys_ui.rs` | egui 0.37 controls / key rebinding panel. | `KeysUiState`, `draw_keys()` |
| `rotland/src/light.rs` | The light of the day, ported from Godot's DayNightPalette (sun path, sun colour, sky colours). | `flashlight()`, `at_hour()` |
| `rotland/src/main.rs` | Rotland (Rust). R0: the real world with real trees and 25 zombies, for the go/no-go speed test. Options: --scene=forest\|camp... | `PlayerPose` |
| `rotland/src/map_ui/action.rs` | User actions resulting from interacting with the map UI. | `MapAction` |
| `rotland/src/map_ui/draw.rs` | Drawing routines for the full-screen map page (egui 0.37). | `draw_map()` |
| `rotland/src/map_ui/draw_icons.rs` | Icon rendering primitives for the map screen: camp, building, bounty, death, waypoint, and player arrow. | `draw_camp_icon()`, `draw_building_icon()`, `draw_bounty_icon()`, `draw_death_icon()`, `draw_waypoint_icon()`, `draw_player_icon()` |
| `rotland/src/map_ui/mod.rs` | Full-screen map page (egui 0.37). | `MapAction`, `draw_map`, `MapUiState`, `MapView` |
| `rotland/src/map_ui/state.rs` | Interactive UI state for the map page: camera centre, zoom level, and cached texture. | `MapUiState` |
| `rotland/src/map_ui/tests.rs` | Unit tests for map UI zoom clamping, panning bounds, and player center positioning. | `map_ui_state_zoom_clamps()`, `map_ui_state_pan_and_centre()` |
| `rotland/src/map_ui/view.rs` | Read-only view snapshot passed from the gameplay layer to the map UI renderer. | `MapView` |
| `rotland/src/menus.rs` | Pure menu state machine and transitions for Main, SlotPicker, Settings, and Paused screens. | `SlotState`, `SlotSummary`, `SlotMode`, `Screen`, `MenuAction`, `Menus` |
| `rotland/src/menus/menus_draw.rs` | egui 0.37 rendering for Main and Paused screens, and central draw dispatcher. | `draw()`, `format_play_time`, `format_saved_at` |
| `rotland/src/menus/menus_draw/menus_draw_settings.rs` | egui 0.37 rendering for the Settings menu (graphics preset, sensitivity, FOV, scale, audio, FPS). | `draw_settings()` |
| `rotland/src/menus/menus_draw/menus_draw_slots.rs` | egui 0.37 rendering for SlotPicker screen and delete confirmation dialog. | `format_play_time()`, `format_saved_at()`, `draw_slot_picker()`, `draw_delete_confirm()` |
| `rotland/src/menus/menus_tests.rs` | Unit tests for main menu navigation, save slot selection, and settings persistence. | `test_escape_transitions()`, `test_continue_disabled_with_all_slots_empty()`, `test_delete_confirm_flow()`, `test_settings_back_navigation()` |
| `rotland/src/play.rs` | The play-mode systems that sit around the player: the backpack and hotbar (what is in hand), the zombie spawner, and the HUD numbers. | `PlayState`, `HandChange` |
| `rotland/src/player_visual.rs` | Player visual representation: rigged body mesh, humanoid animation state, and static weapon model attached to the right hand. | `PlayerVisual` |
| `rotland/src/poi_links.rs` | Glue linking world POIs to gameplay systems (DogPoiSite list and map building notes). | `build_dog_poi_site()`, `build_dog_poi_sites()`, `note_nearby_pois()` |
| `rotland/src/props.rs` | Placed models (trees, rocks, bushes, grass, buildings) drawn as instances, with Godot's distance LODs and shadow rules (ScatterConfig.RangeFor, ChunkProps.ShouldCastShadow). | `PropModel`, `Placed`, `PropLibrary` |
| `rotland/src/session.rs` | Save files <-> the running game. | `Restore`, `seed_of()`, `restore()`, `restore_play()`, `Snapshot`, `capture()` (+1 more) |
| `rotland/src/settings.rs` | Game settings persistence (mouse sensitivity, invert Y, FOV, render scale, vsync, audio, FPS, preset). | `MIN_SENSITIVITY`, `MAX_SENSITIVITY`, `MIN_FOV`, `MAX_FOV`, `MIN_RENDER_SCALE`, `MAX_RENDER_SCALE` (+2 more) |
| `rotland/src/shooting.rs` | Player shooting and gun handling (port of scripts/Combat/GunHandler.cs and WeaponHolder.cs). | `HEAD_HEIGHT_OFFSET`, `HEAD_RADIUS`, `THIRD_PERSON_AIM_FOV`, `FIRST_PERSON_AIM_FOV`, `ShootInput`, `ShootEvents` (+3 more) |
| `rotland/src/streaming.rs` | Loads the world around the player and drops it behind them, like Godot's ChunkManager: chunks within LOAD_RADIUS are built on worker threads (terrain mesh, props, buildings), a few are uploaded to the GPU per frame so there is never a hitch, and chunks beyond UNLOAD_RADIUS are freed. | `LOAD_RADIUS`, `UNLOAD_RADIUS`, `LoadedChunk`, `TreeSlot`, `Streamer` |
| `rotland/src/terrain.rs` | Ground chunks from the world generator, turned into vertices for the GPU. | `chunk_vertices()`, `chunk_of()` |
| `rotland/src/trunk_play.rs` | Car trunk gameplay interactions and inventory transfers. | `INTERACT_RANGE`, `HALF_CONE_DEGREES`, `DEFAULT_TRUNK_LOCAL`, `OPEN_TRUNK_PROMPT`, `open_trunk_prompt()`, `trunk_in_front()` (+2 more) |
| `rotland/src/wildlife.rs` | Wild animals on screen (game::animals): deer, rabbits and boars that graze, flee and (boars) charge. | `FIRST_ANIMAL_ID`, `Wildlife`, `Prey` |
| `rotland/src/zombies.rs` | The live zombies: each has a brain (game::zombie), health, a body moved by the physics mover, and an animated look. | `BODY_RADIUS`, `Zombie`, `Zombies`, `ZombieFrame` |

## 3. Data Files (`data/*.json`)

All JSON data files located in `data/` and the Rust files that load or parse them:

| Data File | Loaded By | Purpose & Description |
|---|---|---|
| `animals.json` | `game/src/animals/config.rs`, `rotland/src/main.rs` | Animal species configuration: health, speeds, senses, flee thresholds, loot tables, and meat yields. |
| `bounties.json` | `game/src/bounties/config.rs`, `rotland/src/camp.rs` | Bounty contract templates, kill/scavenge targets, time limits, and reward payouts. |
| `camp_life.json` | `game/src/camp/config.rs`, `game/src/camp/director.rs` | Camp resident schedules, NPC roles, wander waypoints, gossip dialogues, and camp behaviors. |
| `difficulty.json` | `game/src/difficulty/config.rs`, `rotland/src/camp.rs` | Difficulty presets (Easy, Normal, Hardcore, Nightmare) configuring damage multipliers and death rules. |
| `dog.json` | `game/src/dogs/config.rs`, `rotland/src/main.rs` | Companion and stray dog parameters: hunger, obedience, growth stages, combat damage, and tricks. |
| `economy.json` | `game/src/economy/config.rs`, `rotland/src/camp.rs` | Trader pricing curves, buy/sell markups, restock intervals, cash caps, and initial trader inventories. |
| `farming.json` | `game/src/farming/config.rs`, `rotland/src/camp.rs` | Crop growth timings, watering thresholds, weed growth rates, harvest yields, and farming XP. |
| `graphics_presets.json` | *(not loaded in Rust yet; Godot reference)* | Visual presets (low, medium, high) specifying shadow distance, render scale, and fog toggles. |
| `hints.json` | `game/src/hints/config.rs`, `rotland/src/play.rs` | Contextual tutorial hint triggers, display cooldowns, priority ordering, and dismissal rules. |
| `items.json` | `game/src/items/registry.rs`, `rotland/src/play.rs`, `rotland/src/trunk_play.rs` | Item catalog: item IDs, names, stack limits, weights, categories, icons, and durability stats. |
| `loot_tables.json` | `game/src/loot/table.rs`, `rotland/src/play.rs` | Loot generation tables for containers, crates, car trunks, building cabinets, and enemy drops. |
| `poi.json` | `world/src/poi.rs`, `world/src/generator.rs`, `game/src/poi_zombies/manager.rs` | Point-of-Interest building blueprints: layout footprints, room dimensions, door anchors, and spawn tags. |
| `recipes.json` | `game/src/crafting/registry.rs`, `rotland/src/camp.rs` | Crafting recipes: required ingredients, crafting station (hand/campfire), output counts, and crafting times. |
| `scatter.json` | `world/src/scatter.rs`, `world/src/generator.rs`, `rotland/src/props.rs` | Natural scatter rules: tree, boulder, bush, and grass model densities and biome clustering. |
| `settlements.json` | `world/src/settlement.rs`, `world/src/generator.rs`, `game/src/camp/director.rs`, `game/src/camp_guards/config.rs` | Camp settlement blueprints: fence perimeters, gate locations, NPC posts, and ground pad leveling. |
| `skills.json` | `game/src/skills/registry.rs`, `rotland/src/camp.rs` | Skill tree perks: combat, survival, crafting, farming, agility levels, XP thresholds, and bonuses. |
| `sounds.json` | `audio/src/library.rs`, `audio/src/lib.rs` | Audio library catalog mapping sound IDs to audio files, playback modes (2D/3D), volumes, and pitch variance. |
| `spawning.json` | `game/src/spawner_config.rs`, `game/src/spawner.rs`, `game/src/poi_zombies/manager.rs`, `rotland/src/play.rs` | Zombie spawning density caps, distance brackets, day/night wave frequencies, and herd spawn weights. |
| `start_inventory.json` | `game/src/items/start.rs`, `rotland/src/main.rs` | Default starter loadouts: starting clothing, basic weapons, rations, and starting ammo allocations. |
| `survival.json` | `game/src/survival/config.rs`, `rotland/src/play.rs` | Survival rates: hunger decay, thirst decay, health regen multipliers, starvation penalties, and well-fed buffs. |
| `vehicles.json` | `game/src/vehicles/config.rs`, `rotland/src/main.rs`, `rotland/src/trunk_play.rs` | Vehicle vehicle dynamics: top speeds, acceleration, fuel capacities, trunk slot counts, and health pools. |
| `weapons.json` | `game/src/weapons.rs`, `rotland/src/play.rs`, `rotland/src/combat.rs`, `rotland/src/shooting.rs`, `humanoid/src/weapon.rs` | Weapon definitions: base damage, fire rates, ammo types, recoil, clip animations, and hand hold offsets. |
| `zombies.json` | `game/src/zombie/defs.rs`, `rotland/src/main.rs`, `rotland/src/zombies.rs` | Zombie archetype parameters: walker, runner, spitter, crawler speeds, health, bite damage, and detection ranges. |

## 4. Command-Line Options

Options parsed by the engine (`engine/src/cli.rs`) and the game (`rotland/src/main.rs` via `EngineArgs::value` / `EngineArgs::flag`):

| Option | Read By | What it does |
|---|---|---|
| `--shot=PATH.png` | `engine/src/cli.rs`, `engine/src/app.rs` | Saves a screenshot PNG of frame `--shot-frame` plus `PATH.json` stats, then quits. |
| `--shot-frame=N` | `engine/src/cli.rs`, `engine/src/app.rs` | Frame number captured by `--shot` (default 120). |
| `--frames=N` | `engine/src/cli.rs`, `engine/src/app.rs` | Terminates game run after exactly N frames. |
| `--seconds=S` | `engine/src/cli.rs`, `engine/src/app.rs` | Terminates game run after S elapsed seconds. |
| `--warmup=S` | `engine/src/cli.rs`, `engine/src/app.rs` | Excludes frames in the first S seconds from stats calculations (default 2.0s). |
| `--stats=PATH.json` | `engine/src/cli.rs`, `engine/src/app.rs` | Writes frame-time percentiles and game stats to JSON file upon exit. |
| `--size=WxH` | `engine/src/cli.rs`, `engine/src/app.rs` | Window resolution in pixels (default 1920x1080). |
| `--scale=F` | `engine/src/cli.rs`, `engine/src/app.rs` | Internal rendering scale factor from 0.5 to 1.0 (default 1.0). |
| `--vsync` | `engine/src/cli.rs`, `engine/src/app.rs` | Enables vertical sync capped to monitor refresh rate (default: uncapped). |
| `--scene=SCENE` | `rotland/src/main.rs:704` | Selects active scene: `forest` (scripted benchmark walk), `camp`, or `play` (interactive mode). |
| `--seed=N` | `rotland/src/main.rs:700` | World generator random seed integer (default 42). |
| `--hour=H` | `rotland/src/main.rs:708` | Sets time of day from 0.0 to 24.0 and freezes in-game clock. |
| `--free` | `rotland/src/main.rs:712` | Enables free fly camera mode controlled by WASD and mouse look. |
| `--save-dir=PATH` | `rotland/src/main.rs:716` | Custom directory path for save slots and settings (default `~/.local/share/rotland-rust`). |
| `--menu` | `rotland/src/main.rs:723` | Starts directly on the main title menu. |
| `--load=N` | `rotland/src/main.rs:727` | Loads and continues Rust save slot N (1-3). |
| `--import-godot=N` | `rotland/src/main.rs:732` | Imports Godot save slot N (read-only) into Rust slot N and launches it. |
| `--slot=N` | `rotland/src/main.rs:737` | Sets the active save slot number for saving operations. |
| `--save-at=S` | `rotland/src/main.rs:743` | Triggers an automatic save to active slot after S seconds elapsed. |
| `--yaw=DEG` | `rotland/src/main.rs:750` | Sets starting camera and player horizontal facing yaw angle in degrees. |
| `--give=ITEMS` | `rotland/src/main.rs:794` | Adds items to player backpack at startup (e.g. `--give=hoe,seeds_wheat:5`). |
| `--animals=LIST` | `rotland/src/main.rs:756` | Spawns test entities 9m ahead (e.g. `deer,boar,rabbit`; supports `campfire`, `dog`, `drive`). |
| `--zombies=N` | `rotland/src/main.rs:767` | Overrides initial zombie spawn count. |
| `--hub=TAB` | `rotland/src/main.rs:773` | Starts with player hub opened to tab: `bag`, `skills`, `bounties`, `profile`, or `map`. |
| `--camp-screen=FACILITY` | `rotland/src/main.rs:777` | Opens camp facility screen on startup: `trader`, `crafting`, `farm`, or `guards`. |
| `--chop` | `rotland/src/main.rs:781` | Teleports player next to nearest tree with axe equipped. |
| `--autoattack` | `rotland/src/main.rs:785` | Continuously triggers weapon attack swings automatically. |
| `--autowalk` | `rotland/src/main.rs:789` | Forces continuous forward movement / driving / ladder climbing. |
| `--die-at=S` | `rotland/src/main.rs:798` | Kills player at elapsed time S seconds to test death penalty and respawn. |
| `--first-person` | `rotland/src/main.rs:804` | Starts in first-person camera mode with weapon viewmodel visible. |
| `--poi` | `rotland/src/main.rs:810` | Teleports player 20m from nearest POI building to trigger POI zombie waking. |
| `--climb` | `rotland/src/main.rs:816` | Teleports player next to nearest ladder and grabs onto it. |
| `--mute` | `rotland/src/main.rs:822` | Mutes audio playback (sound triggers are still counted in stats). |
| `--flashlight` | `rotland/src/main.rs:826` | Starts run with player flashlight turned on. |

## 5. Stats Keys in JSON (`game` object)

When running with `--stats=PATH.json` or `--shot=PATH.png`, `engine/src/app.rs:347` inserts `game.stats()` into the output JSON under `"game"`. All keys are produced in `rotland/src/main.rs:2172-2277` in `R0::stats()`:

| Key | Produced In | Description & Source |
|---|---|---|
| `drawCalls` | `rotland/src/main.rs:2174` | Total opaque and alpha mesh draw calls in the last frame (`r.last.draws`). |
| `triangles` | `rotland/src/main.rs:2175` | Total geometry triangles rendered in the forward pass (`r.last.triangles`). |
| `shadowDrawCalls` | `rotland/src/main.rs:2176` | Draw calls executed in the directional sun shadow pass (`r.last.shadow_draws`). |
| `shadowTriangles` | `rotland/src/main.rs:2177` | Triangles rendered in the directional sun shadow pass (`r.last.shadow_triangles`). |
| `colliders` | `rotland/src/main.rs:2179` | Active Rapier3D collider count in PhysicsWorld (`self.physics.collider_count()`). |
| `playerPosition` | `rotland/src/main.rs:2181` | Current player coordinates string formatted as `"{x:.2} {y:.2} {z:.2}"`. |
| `groundCheck` | `rotland/src/main.rs:2185` | Sanity validation comparing physics ground raycast height vs procedural generator height. |
| `scene` | `rotland/src/main.rs:2189` | Active scene name string (`"forest"`, `"camp"`, or `"play"`). |
| `soundsPlayed` | `rotland/src/main.rs:2192` | Total sound effects played through audio mixer (`audio.played`). |
| `zombieVoices` | `rotland/src/main.rs:2193` | Count of zombie vocalization sounds triggered (`self.voices_played`). |
| `poisCleared` | `rotland/src/main.rs:2194` | Number of POI sites cleared of zombies (`self.pois_cleared`). |
| `poiZombiesAwake` | `rotland/src/main.rs:2196` | Count of POI zombies currently awake and active (`pz.awake_count()`). |
| `poisVisited` | `rotland/src/main.rs:2197` | Count of distinct POI sites visited by the player (`pz.visited_ids().len()`). |
| `nearestZombie` | `rotland/src/main.rs:2201` | Distance in meters to the nearest living zombie (`"{near:.1}"`). |
| `lastSound` | `rotland/src/main.rs:2203` | Sound ID string of the most recently played sound effect. |
| `soundLive` | `rotland/src/main.rs:2204` | Boolean indicating whether the audio output stream is currently operational. |
| `seed` | `rotland/src/main.rs:2206` | Procedural generation seed integer (`self.seed`). |
| `groundChunks` | `rotland/src/main.rs:2208` | Count of active loaded terrain chunks in Streamer (`st.chunks.len()`). |
| `placedModels` | `rotland/src/main.rs:2210` | Total placed static prop instances across all active chunks. |
| `zombies` | `rotland/src/main.rs:2219` | Total zombies spawned and managed in Zombies struct (`z.list.len()`). |
| `zombiesAlive` | `rotland/src/main.rs:2220` | Count of currently living zombies (`z.alive()`). |
| `ladders` | `rotland/src/main.rs:2223` | Total ladder interaction segments across active chunks. |
| `climbing` | `rotland/src/main.rs:2225` | Boolean indicating whether player character is currently mounted on a ladder. |
| `campfires` | `rotland/src/main.rs:2227` | Total player-placed and camp campfires (`play.camp.campfires.campfires().len()`). |
| `campfiresLit` | `rotland/src/main.rs:2252` | Count of campfires that are currently burning (`f.is_lit()`). |
| `dogPosition` | `rotland/src/main.rs:2229` | Coordinates of the companion dog formatted as `"{x:.2} {y:.2} {z:.2}"`. |
| `dogClip` | `rotland/src/main.rs:2230` | Current skeletal animation clip name playing on the companion dog. |
| `campCar` | `rotland/src/main.rs:2234` | Camp vehicle status formatted as `"{x:.2} {y:.2} {z:.2} {speed}kmh fuel {fuel:.1}"`. |
| `mapChunks` | `rotland/src/main.rs:2236` | Count of discovered map exploration chunks (`play.map.explore.chunk_count()`). |
| `hint` | `rotland/src/main.rs:2237` | Currently displayed tutorial hint message text. |
| `felledTrees` | `rotland/src/main.rs:2238` | Count of trees felled by the player (`self.trees.fallen.len()`). |
| `guardShots` | `rotland/src/main.rs:2239` | Total bullets fired by camp settlement guards (`camp_guards.total_shots_fired()`). |
| `parkedSolids` | `rotland/src/main.rs:2240` | Count of parked vehicle rigid body colliders registered (`self.car_solids.count()`). |
| `mapBuildings` | `rotland/src/main.rs:2241` | Discovered building markers registered on the map (`play.map.found.len()`). |
| `strayDogs` | `rotland/src/main.rs:2242` | Count of un-tamed stray dogs active in the world. |
| `profile` | `rotland/src/main.rs:2244` | Player career statistics formatted as `"kills {k} headshots {h} deaths {d} distance {m}"`. |
| `bubbles` | `rotland/src/main.rs:2248` | Count of active NPC speech dialogue bubbles visible on screen. |
| `campChats` | `rotland/src/main.rs:2251` | Count of camp NPC gossip chat dialogue events triggered. |
| `animals` | `rotland/src/main.rs:2255` | Total living wildlife animals (`w.sim.live_count()`). |
| `animalSpecies` | `rotland/src/main.rs:2257` | Comma-separated string of species IDs for all living wildlife. |
| `playerHealth` | `rotland/src/main.rs:2260` | Player current health float formatted as `"{hp:.0}"`. |
| `kills` | `rotland/src/main.rs:2261` | Total zombie kills in current session (`combat.kills`). |
| `shots` | `rotland/src/main.rs:2262` | Total shots fired by the player in current session (`combat.shots`). |
| `ammoBag` | `rotland/src/main.rs:2264` | Pistol ammo count currently carried in player inventory backpack. |
| `ammoMag` | `rotland/src/main.rs:2265` | Bullets remaining in equipped weapon magazine. |
| `playerDead` | `rotland/src/main.rs:2267` | Boolean flag indicating whether the player is dead. |
| `playerSpeed` | `rotland/src/main.rs:2271` | Player horizontal ground speed magnitude formatted as `"{spd:.2}"`. |
| `playerOnFloor` | `rotland/src/main.rs:2274` | Boolean flag from CharacterMover indicating player is grounded. |
| `playerCrouching` | `rotland/src/main.rs:2275` | Boolean flag indicating player is currently in crouching stance. |
| `playerSprinting` | `rotland/src/main.rs:2276` | Boolean flag indicating player is sprinting. |
| `playerStamina` | `rotland/src/main.rs:2278` | Player stamina bar float formatted as `"{stamina:.1}"`. |

## 6. "Where to Add Things" Cheat Sheet

- **Pure Gameplay Logic**:
  - Add modules to `game/src/<topic>/` (e.g. `game/src/fishing/`, `game/src/traps/`).
  - Write unit tests in `game/tests/<topic>_tests.rs` (pure logic, no GPU or window needed).

- **Procedural World Generation**:
  - Terrain heightmaps, biomes, flora scatter, or POI building blueprints -> `world/` (e.g. `world/src/generator.rs`, `world/src/poi.rs`).
  - Verify generator determinism with golden tests in `world/tests/`.

- **Wiring & Visual Presentation**:
  - Connect gameplay systems to the engine in the matching `rotland/src/app/frame_*.rs` step (see section 1) accompanied by a small focused helper `rotland/src/<topic>.rs` (e.g. `rotland/src/wildlife.rs`, `rotland/src/camp_npcs.rs`).
  - Add egui interface panels under `rotland/src/<feature>_ui/` (e.g. `camp_ui`, `hub_ui`, `map_ui`).

- **Rendering & GPU Pipelines**:
  - Low-level wgpu pipelines, vertex layouts, bind groups, shadow passes, shaders -> `engine/src/render/` (e.g. `lit.rs`, `shadow.rs`, `pipelines.rs`).

- **3D Models & Animations**:
  - Vehicle geometry in `tools/blender/vehicles/car_parts.py` defines the multi-part 4x4 SUV; `tools/blender/vehicles/build_car.py` exports three body/door/tailgate colour variants and the shared wheel `.glb`.
  - Author or edit models with Blender empties/markers and collision boxes in `tools/blender`.
  - Re-bake assets into `assets/models.pack` using the baker tool (`cargo run --release -p baker`).
