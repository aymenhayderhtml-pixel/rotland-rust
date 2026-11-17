# Rotland

A solo, offline, 3D open-world zombie survival game built in Rust with its own engine (`wgpu`, `winit`, `egui`, `rapier3d` physics, and `kira` audio). You start in a walled camp, venture out into the world to loot houses and crates, sell what you find to the trader, and fight to get back before nightfall. Survive as long as you can: battle zombies with guns and melee weapons, hunt wildlife, farm crops, cook, drive a car, keep a companion dog, and level up your skills.

## Play it

To build and run the game from this folder:

```bash
cargo build --release
target/release/rotland
```

- Add `--menu` to start directly on the main menu: `target/release/rotland --menu`.
- Rebuild model assets: `cargo run --release -p baker` compiles all `.glb` models from `assets/models/` into `assets/models.pack`. This is required on a fresh clone because `assets/models.pack` is not tracked in git.
- Saves are stored in `~/.local/share/rotland-rust/`.
- Import Godot saves: add `--import-godot=N` (where `N` is slot 1–3) to copy Godot slot `N` (read-only) into Rust slot `N` and play it.

## Controls

These are the default controls confirmed from code. Most keys can be changed in Settings > Controls. (Settings also lists save/load, arrow-key look, tab switching and "dog go home" keys; those are not wired into the game yet.)

### On foot
| Key / Mouse | Action |
|---|---|
| W, A, S, D | Move (camera-relative) |
| Mouse | Look around (yaw / pitch) |
| Shift | Sprint (forward only, drains stamina) |
| Space | Jump (drains stamina) |
| C | Crouch |
| Left Click | Attack: swing melee weapon, or shoot (hold for automatic weapons) |
| Right Click (hold) | Aim (zooms in, narrower FOV; uses scope on rifle) |
| E | Interact / use (loot crates, talk to NPCs, butcher animals, tend farm beds, climb ladders, enter cars) |
| R | Reload weapon |
| 1 to 6 | Hotbar slots (select weapon or use consumable) |
| V | Toggle 1st / 3rd person camera |
| Mouse wheel | Zoom camera in/out (3rd person) |
| Flashlight | Use the flashlight item from hotbar (1–6) or bag to toggle beam |
| Esc | Pause menu / release mouse cursor (click window to capture again) |

### Player Hub & Screens
| Key | Screen / Action |
|---|---|
| Tab | Bag / Inventory (crafting recipes below item slots) |
| L | Skills |
| J | Bounties |
| P | Profile |
| M | World Map (drag to pan, wheel to zoom, right-click to place waypoint) |

### Driving
| Key | Action |
|---|---|
| W / S | Throttle / brake and reverse |
| A / D | Steer |
| Space | Handbrake |
| H | Horn |
| L | Toggle headlights on/off |
| E | Exit vehicle |

### Companion Dog
| Key | Action |
|---|---|
| G | Stay / follow toggle |
| E (on dog) | Open dog menu (inventory pack, feed, stay/follow) |

---

## Commands (always through the shared lock: `flock /tmp/rotland-heavy.lock <command>`)

```bash
cargo build                         # fast dev build (target/debug/rotland)
cargo build --release               # the build the owner plays and benchmarks use
cargo run --release -p baker        # bake all models into assets/models.pack
cargo test                          # all unit and golden tests (no window)
cargo clippy --all-targets          # must have no warnings in new code
```

## Seeing and measuring the game (made for AI workers)

Every run can be scripted, photographed, and measured. Output lines start with `ROTLAND` so they are easy to grep.

| Option | What it does |
|---|---|
| `--shot=PATH.png` | saves frame `--shot-frame` (default 120) as a PNG plus `PATH.json` (frame times, GPU, sizes, the game's own numbers), then quits. Prints `ROTLAND SHOT PATH` |
| `--seconds=S` / `--frames=N` | ends the run after S seconds / N frames |
| `--stats=PATH.json` | writes frame-time stats (avgFps, p50/p95/p99 frame ms, worst 1 % FPS) at the end. Prints `ROTLAND STATS PATH` |
| `--warmup=S` | frames in the first S seconds are not counted (default 2) |
| `--size=WxH` | window size (default 1920x1080) |
| `--scale=F` | render scale 0.5 to 1.0 |
| `--vsync` | cap to the screen rate (default: uncapped) |
| F12 (while playing) | saves `shots/shot_<time>.png` + `.json` |

Every run ends with `ROTLAND DONE frames=… avgFps=… p95FrameMs=… worst1PercentFps=…`.  
Game options (scenes, camera spots) are read by the game from the same command line (`EngineArgs::value("--scene")`).

### Save and menu options (play scene)

| Option | What it does |
|---|---|
| `--save-dir=PATH` | use this folder for save slots and settings. **Tests and screenshots must always pass it**; without it the game uses the real folder `~/.local/share/rotland-rust/` |
| `--menu` | start on the main menu |
| `--load=N` | continue Rust slot N (1-3) |
| `--import-godot=N` | copy Godot slot N (read only, never changed) into Rust slot N and play it |
| `--save-at=S` | save into the active slot after S seconds (tests) |
| `--yaw=DEG` | start the player and camera facing this way (screenshots) |
| `--give=hoe,seeds_wheat:5` | put these items in the backpack at the start (tests) |
| `--animals=deer,boar,rabbit` | put these animals 9 m in front of the player at the start (screenshots); `campfire` in the list lays a burning campfire 4 m ahead, `dog` gives the player a grown dog, `drive` seats the player in the camp's car (with `--autowalk` it drives off). Stats: `dogPosition`, `dogClip`, `campCar`, `mapChunks`, `hint`. The stats JSON has `animals` and `animalSpecies` |
| `--hub=bag\|skills\|bounties\|profile\|map` | start with the player hub open on that tab (screenshots) |
| `--chop` | after a second, stand at the nearest tree with the axe in hand (with `--autoattack` it is felled after 4 swings). Stats: `felledTrees`, `guardShots`, `parkedSolids`, `mapBuildings`, `strayDogs`, `profile`, `bubbles`, `campChats` |
| `--first-person` | start in the first-person camera (screenshots of the weapon in view). Stats: `zombieVoices`, `nearestZombie` |
| `--poi` | after a second, stand 20 m from the nearest building (wakes its zombies). Stats: `poisVisited`, `poiZombiesAwake`, `poisCleared` |
| `--climb` | after a second, stand at the nearest ladder and grab it (with `--autowalk` the player climbs up). Stats: `ladders`, `climbing`, `campfires`, `campfiresLit` |
| `--mute` | no sound. Runs with `--shot`, `--seconds`, `--frames` or `--stats` are always muted; the JSON stats still count `soundsPlayed` / `lastSound` |
| `cargo run -p save --example check_godot` | reads the owner's Godot slots (read only) and prints what was understood |

Example: `flock /tmp/rotland-heavy.lock target/debug/rotland --shot=/tmp/x/forest.png` then open the PNG.

## Repo layout

| Folder / Path | Description |
|---|---|
| `engine` | Rotland's custom engine: window (`winit`), input, GPU (`wgpu` on Vulkan), UI (`egui`), frame image, screenshots, and runtime stats |
| `rotland` | The main game program wiring engine, physics, world, audio, and gameplay logic together |
| `pack` | Model pack binary format, data types, and serialization (`postcard`) for GPU-ready models |
| `baker` | Asset baker tool converting Blender `.glb` models into compact binary packs (`assets/models.pack`) |
| `world` | Procedural world generator, ground paint, terrain chunks, biomes, day/night cycle, scatter, POIs, and settlements (pure, testable) |
| `physics` | Rapier3D collision (terrain, static props, buildings) and kinematic character mover |
| `game` | Core gameplay logic: player controller, camera rig, input actions, stamina, health, weapons, economy, crafting, and vehicles (pure, testable) |
| `humanoid` | Humanoid animation state machine, locomotion blend, one-shot action layering, and right-hand weapon attachment |
| `save` | Save file format, slots management, player/world state serialization, and read-only Godot save importer |
| `audio` | Positional 3D and stereo sound effects, day/night ambient loops, and volume management powered by `kira` |
| `paths` | Runtime project root discovery (`Cargo.lock` + `data/`), avoiding build-time hardcoded paths |
| `data/` | Game data definitions in JSON format (`data/*.json`: items, recipes, weapons, vehicles, zombies, loot tables, hints, etc.) |
| `assets/` | Raw assets: `models/` (.glb files), `audio/` (sound effects, music), `icons/`, `fonts/`, `textures/`, and the baked `models.pack` |
| `golden/` | Deterministic golden reference files (`golden/world.json`) for parity verification against Godot |
| `tools/blender/` | Blender 5.3 model generation scripts; run via `blender -b --python tools/blender/<script>.py` |
| `tools/audio/synth.py` | Procedural sound synthesizer generating retro game sound effects |
| `docs/` | Project documentation and architecture specs (`RUST_PLAN.md`, `RUST_PARITY.md`, `RUST_QUEUE.md`, `JESTER.md`, `GAME_DESIGN.md`, `DECISIONS.md`, `ART_PIPELINE.md`, `*_CONTRACT.md`, `CODEMAP.md`) |
| `tasks/` | Development task specifications and notes from previous implementation slices |

## Reference

The original Godot C# version is frozen at [https://github.com/aymenhayderhtml-pixel/rotland](https://github.com/aymenhayderhtml-pixel/rotland). It serves solely as the behavioral reference for porting. See `docs/RUST_PARITY.md` for current parity status and what features remain to be implemented.

## Rules for workers

- **Read `AGENTS.md` first**, then this file, then only the files your task specifically names.
- **Workers do not build or run tests.** Do not run `cargo build`, `cargo test`, `cargo clippy`, or the game. Write clean code and unit tests, report, and let the orchestrator merge and execute unified builds and tests.
- **Port behaviour from the Godot C# file named in the task.** Keep the same numbers, algorithms, and data files (`data/*.json`). The frozen Godot repository is located at `../zombi servival` on the owner's machine (read-only).
- **Find project files at run time with `paths::repo_path("data/...")`**, never with `env!("CARGO_MANIFEST_DIR")` (which bakes the build directory into the binary). The repo root is located dynamically by the presence of `Cargo.lock` and `data/`.
- **Pure logic goes in a crate with unit tests.** Compare with golden files from the Godot version where provided.
- **Always pass `--save-dir` for test and screenshot runs.** Never write to the real save folder (`~/.local/share/rotland-rust/`) or Godot saves.
- **Run heavy commands through the shared lock:** `flock /tmp/rotland-heavy.lock <command>`.
- **Keep files small (aim under 300 lines)**, with comments explaining *why*. All code must pass `cargo build`, `cargo test`, and `cargo clippy --all-targets` cleanly.
