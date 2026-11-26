# Code map

Where things live in the Rust workspace (`/home/aymen/Documents/rotland-rust`). Read this once, then open only the files your task needs. Folder names below were checked against the real layout on 2026-11-26.

## 1. Boot and runtime

- **Entry point:** `rotland/src/main.rs` (`fn main`). It starts the engine with `engine::run(R0 { .. }, ..)`. `R0` implements the engine's `Game` trait.
- **Engine loop** (`engine/src/app.rs`): parses command-line flags, opens the window, sets up the GPU (`engine/src/gpu.rs`), then each frame calls `Game::update`, `Game::render`, and `Game::ui`. It also handles screenshots and the stats JSON (`--stats`, `--shot`).
- **Frame steps** (`rotland/src/app/`): `frame_input.rs` (keys), `frame_sim*.rs` (player, zombies, actors, physics), `frame_world.rs`, `frame_camp.rs`, `frame_cars.rs`, `frame_ui.rs`, `frame_audio.rs`, and `render_frame.rs` (drawing). `startup.rs` and `start_world.rs` set up a new game or load a save. `stats.rs` builds the stats JSON. `state.rs` holds the state groups.
- **Runtime paths:** always find files with `paths::repo_path("data/...")` (`paths/src/lib.rs`). Never use `env!("CARGO_MANIFEST_DIR")`.
- **Saves:** default folder `~/.local/share/rotland-rust` (change with `--save-dir=`). Save format in `save/src/`.
- **Models:** `assets/models.pack` is made from the `.glb` files by `cargo run --release -p baker`.

## 2. Crate dependency order

Lowest first. A crate may only use crates above it.

1. `paths`: finds project folders.
2. `pack`: binary model format.
3. `baker`: turns `.glb` files into `assets/models.pack`.
4. `world`: pure procedural world (terrain, biomes, towns, roads, rivers, lakes, city, POIs, weather).
5. `physics`: rapier3d physics and the character mover.
6. `save`: save file format and slots.
7. `audio`: sound library and mixer (kira).
8. `engine`: wgpu renderer, winit app, egui.
9. `humanoid`: skeleton animation, poses, grips, weapon holds.
10. `game_core`: items, loot, skills, weapons, hints, difficulty.
11. `game_zombie`: zombies, flow fields, hordes, director, spawner.
12. `game_camp`: camp, guards, upgrades, guild, economy, bounties, building, farming, crafting, gunsmith, map.
13. `game_player`: player controller, survival, ladders, fishing, trees, vehicles, downed, guns feel.
14. `game_world`: melee, gun fire, projectiles, grenades, animals, dogs.
15. `game`: thin facade that re-exports the `game_*` crates and adds `camera_rig.rs`, `input.rs`, `keybinds/`.
16. `rotland_ui`: all egui screens (HUD, menus, camp, hub, map, gunsmith).
17. `rotland`: the game executable. It wires everything together.

## 3. Folder map

### Engine and data crates

| Folder | What is inside |
|---|---|
| `engine/src/` | `app.rs` (loop), `gpu.rs`, `cli.rs` (flags), `input.rs`, `stats.rs`, `timing.rs`, `anim.rs`, `ui.rs` |
| `engine/src/render/` | `lit.rs` (main lit pass), `shadow.rs`, `pipelines.rs`, `skinned.rs`, `sky.rs`, `water.rs`, `weather.rs`, `camera.rs`, `frustum.rs`, `mesh.rs` |
| `world/src/` | `generator*.rs` (chunk generation), `biome/`, `city/`, `lake/`, `rivers/`, `roads/`, `towns/`, `camp_layout/`, `weather/`, `scatter.rs`, `poi.rs`, `settlement.rs`, `daynight.rs` |
| `physics/src/` | `world.rs` (colliders, broad phase), `mover.rs` (character mover) |
| `humanoid/src/` | `animator.rs`, `layers.rs`, `full_body.rs`, `stance.rs`, `aim.rs`, `swing.rs`, `foot_ik.rs`, `grip.rs`, `swim.rs`, `ladder_anim.rs`, `car_anim.rs`, `wound_anim.rs` (each file has a `_tests.rs` sibling where tested) |
| `audio/src/` | `library.rs` (sound catalog), `mix.rs` |
| `save/src/` | `file.rs`, `slots.rs`, `systems/*.rs` (one file per saved topic) |
| `pack/src/`, `baker/src/` | model pack format; baker steps (`bake.rs`, `mesh.rs`, `skeleton.rs`, `budget.rs`) |

### Game logic crates (pure, no GPU)

| Folder | What is inside |
|---|---|
| `game_core/src/` | `items/` (registry, inventory, hotbar), `loot/`, `skills/`, `weapons/` (weapon data, attachments, wear), `hints/`, `difficulty/`, `health.rs`, `damage.rs` |
| `game_zombie/src/` | `zombie/` (brain, attack, senses, specials: brute, crawler, screamer, biome), `horde/`, `director/`, `flowfield*.rs`, `spawner*.rs`, `poi_zombies/`, `zombie_voices/` |
| `game_camp/src/` | `camp/` (residents, planner, chat), `camp_guards/`, `camp_upgrades/`, `guild/` (board, companions, shop, recruits), `economy/`, `bounties/`, `building/` (base build and siege), `farming/`, `crafting/`, `campfire/`, `gunsmith/` (buy, sell, repair, upgrade), `map/`, `rescue.rs` |
| `game_player/src/` | `player.rs`, `survival/` (hunger, thirst, temperature), `ladders/`, `vehicles/` (car, fuel, damage), `trees/`, `fishing/`, `swimming.rs`, `downed.rs`, `vault.rs`, `gun_feel.rs`, `melee_combo.rs` |
| `game_world/src/` | `melee.rs`, `guns_fire.rs`, `grenade.rs`, `projectiles/`, `animals/` (AI, herds, hunting, spawner), `dogs/` (companion and strays) |
| `game/src/` | `camera_rig.rs`, `input.rs`, `keybinds/` (only glue; logic lives in the `game_*` crates) |

### Game app crates

| Folder | What is inside |
|---|---|
| `rotland/src/` | `main.rs` (entry and `R0`), `app/` (frame steps, see section 1), `zombies/` (glue and drawing), `build_mode/`, `dev_panel/`, `play.rs`, `combat*.rs`, `shooting.rs`, `streaming.rs` (chunk loading), `props.rs`, `terrain.rs`, `water_view.rs`, `river_view.rs`, `wildlife.rs`, `gunsmith_play.rs`, `guild_play.rs`, `rescue_play.rs`, and `*_view.rs` / `*_play.rs` files that draw or wire one feature |
| `rotland_ui/src/` | `hud/` (one file per widget), `camp_ui/`, `hub_ui/` (bag, skills, bounties, profile), `map_ui/`, `gunsmith_ui/`, `menus/`, `settings.rs`, `keys_ui.rs`, `icons.rs`, `display.rs` |

Each `*_ui/` folder usually has `state.rs`, `view.rs`, `draw*.rs`, `action.rs`, and a `tests.rs`. Pure layout and state logic is tested there.

## 4. Data files (`data/*.json`)

Game numbers live in `data/`, not in Rust code. Each Rust module reads its own file with `paths::repo_path`. To find which code reads a file, search for its name, for example `grep -rn "weapons.json" game_core rotland`.

Files present now: `action_anim`, `aim`, `airborne`, `animals`, `animation`, `attachments`, `backpack`, `biomes`, `bounties`, `budgets`, `build_parts`, `camp_layout`, `camp_life`, `camp_upgrades`, `car_anim`, `city`, `city_ground`, `companions`, `death_anim`, `difficulty`, `director`, `dog`, `downed`, `economy`, `effects`, `farming`, `fishing`, `foot_ik`, `fp_motion`, `graphics_presets` (not used yet), `grips`, `ground_detail`, `guild`, `gun_feel`, `gunsmith`, `hints`, `hit_react`, `hud`, `incline`, `items`, `ladder_anim`, `lake`, `loot_tables`, `map`, `melee_guard`, `melee_moves`, `minimap`, `motion`, `poi`, `posture`, `recipes`, `rescue`, `rivers`, `roads`, `scatter`, `settlements`, `siege`, `skills`, `sky`, `sounds`, `spawning`, `special_zombies`, `stances`, `start_inventory`, `survival`, `swim_anim`, `swimming`, `team_anim`, `temperature`, `towns`, `vault`, `vehicles`, `weapon_mounts`, `weapon_wear`, `weapons`, `weather`, `world_layout`, `wound_anim`, `zombie_anim`, `zombies`. (All are `.json`.)

## 5. Command-line flags

The engine parses the common flags in `engine/src/cli.rs` (`--shot`, `--shot-frame`, `--frames`, `--seconds`, `--warmup`, `--stats`, `--size`, `--scale`, `--vsync`). The game's own flags are read in `rotland/src/main.rs` and `rotland/src/app/test_flags.rs` (for example `--scene`, `--seed`, `--hour`, `--save-dir`, `--load`, `--at`, `--biome`, `--weather`, `--spawn-zombie`, `--gunsmith`, `--give`). Always pass `--save-dir=<scratch folder>` in tests and screenshot runs.

Stats JSON keys (written with `--stats`) are built in `rotland/src/app/stats.rs`.

## 6. Where to add things

- **New gameplay rule or number:** pure logic in the matching `game_*` crate (for example `game_camp/src/gunsmith/`). Data in `data/*.json`. Unit tests next to the code or in `game/tests/all/` (one test program per crate; add `mod <name>;` to `game/tests/all/main.rs`).
- **New world feature** (terrain, towns, roads, rivers): `world/src/<topic>/`, with golden tests in `world/tests/`.
- **Wiring into the game:** the matching `rotland/src/app/frame_*.rs` step, plus a small helper in `rotland/src/<topic>*.rs`.
- **New screen or menu:** `rotland_ui/src/<name>_ui/` (state, view, draw, action, tests).
- **Rendering:** `engine/src/render/` (shaders are `.wgsl` files in the same folder).
- **Models and animation:** scripts in `tools/blender/` (run with `blender -b --python`), then `cargo run --release -p baker` to rebuild `assets/models.pack`.
