# Rust migration plan (2026-11-15)

**Owner's call:** move Rotland from Godot/C# to **Rust with our own engine on wgpu** (no Bevy, no other game engine), tuned to the limit for **one PC**: the owner's laptop (i5-10310U, 4 cores / 8 threads, Intel UHD CML GT2 on Vulkan 1.4 / Mesa, 14 GB RAM). Claude (Opus) orchestrates; **Gemini 3.8 Flash** workers port systems; **Haiku** workers build, test and check.

## Ground rules
1. **The Godot game stays untouched and playable** until the Rust version matches it (parity checklist, phase R7). Shared `data/*.json` and `assets/` are read by both; don't change them in a way that breaks Godot.
2. **Same world, same rules.** The Rust version reads the same JSON data and the same models, and uses the same noise library (FastNoiseLite has an official Rust port), so the same seed gives the same world. Ports are checked against **golden files** dumped from the Godot version (heights, scatter, loot rolls, damage numbers).
3. **Every phase ends with something the owner can run**, a benchmark against Godot on the same view, and all tests green.
4. **Measure plugged in**, other apps closed, alternating old/new runs.
5. **One heavy job at a time** (`flock /tmp/rotland-heavy.lock`): Rust builds are as heavy as Godot runs. All worktrees share one cargo target folder.

## Engine design (what makes it fast on this laptop)
- **Libraries, not an engine:** `winit` (window, input), `wgpu` (GPU, Vulkan backend), `glam` (math), `rapier3d` (collision, character and car physics), `kira` (sound), `egui` (menus, HUD), `serde_json` (the existing data files), `fastnoise-lite` (terrain), `rayon` (use all 8 threads), `gltf` (only in the asset baker).
- **Baked assets ("hardcoded objects"):** a baker tool turns every .glb (still made by the Blender scripts) into one compact binary pack that loads straight into GPU memory. No .glb parsing in the game.
- **Baked light ("hardcoded light"):** soft shadows (ambient occlusion) painted into vertex colours at bake time; the live light is one cheap shader for everything: sun + sky ambient + fog. Live sun shadows only near the player; far characters get a cheap round shadow.
- **Few, big draws:** one palette texture array for all models, one pipeline for all static models, GPU instancing for repeated models, static objects merged per chunk, distance LOD, frustum culling on all threads.
- **Render scale + our own FSR 1 upscale** (Godot's Mobile renderer could not use FSR; ours can).
- **Simple data, no ECS:** each system keeps its things in plain arrays (`Vec<Zombie>` and so on), updated in parallel where it pays. Rigid skinning (one bone per vertex) on the GPU.
- **Zombie pathing:** a flow field around the player instead of a navmesh per zombie (one search serves the whole horde).
- **Headless mode** (no window) for tests; **bench mode** with fixed camera paths, writing the same JSON as the Godot benchmark.

## Code layout (`rust/`, a cargo workspace)
| Crate | What |
|---|---|
| `engine` | window, input, renderer, asset pack loading, audio, physics glue, frame timing |
| `baker` | tool: .glb → asset pack (meshes, palette, skeletons, clips, markers, colliders) |
| `world` | generator, ground paint, chunks, scatter, POIs, settlements (pure, testable) |
| `game` | player, zombies, combat, items, economy, camp, animals, farming, dog, car, saves |
| `ui` | HUD, hub screens, menus, settings (egui) |
| `rotland` | the game program (also `--headless`, `--bench=`, `--snapshot=`) |

## Phases
| # | What | Done when |
|---|---|---|
| **R0** ✅ | **Go / no-go test.** Window + renderer, real terrain from the real generator, real trees from the asset pack, 25 skinned zombies playing their walk clip, the forest benchmark camera path | Same view as Godot's `forest25`, **clearly faster** (target 1.5x or more). If not, stop and rethink before months go in |
| R1 | World: chunk streaming on worker threads, scatter + LOD, day/night, fog, shadows, POI buildings, camp layout, static collision | world snapshots match Godot's; golden tests pass |
| R2 | Player: controller, first/third-person camera, key bindings, humanoid + clips, weapon in hand | walk around the real world at full speed |
| R3 | Zombies + combat: AI states, flow field, pooling and far-zombie throttles, melee, guns, health, death and respawn | `forest25` benchmark with real AI beats Godot |
| R4 | Items, UI, saves: inventory, hotbar, loot, HUD, hub tabs, menus, settings, 3 save slots (import the Godot saves) | a full loot-and-sell loop |
| R5 | Camp: NPCs, trader, bank, bounties, crafting, campfire, harvesting, hints, difficulty | camp life matches Godot |
| R6 | Life: animals + hunting, farming, dog, car, map, ladders, all sounds | every Godot feature present |
| R7 | Parity + switch: checklist from CODEMAP, balance, Linux build, owner playtest; then the Godot version is archived | owner OK |

## Team
- **Opus (orchestrator):** architecture, the renderer core and other tricky core code, task specs, reviews, merges, benchmarks, owner updates.
- **Gemini 3.8 Flash** (T3 `delegate_task`, worktree): ports one system per task, with the C# files as the reference and golden files as the check.
- **Haiku** (`Agent`): builds, `cargo test` / `clippy`, golden-file dumps from Godot, checking ports against the C#, small fixes.
- At most 3 workers at once; builds and benchmarks go through the heavy lock.

## Size (honest)
About 38,000 lines of C# game code and 364 tests. R0 is about a week of work; full parity is a matter of months. The Godot game stays playable the whole time.

## R0 result (2026-11-15): GO
Godot `forest25` against the Rust forest scene (same seed, same densest-forest spot, same 20 m square walk, third-person camera 4 m back, 25 zombies), alternating runs, laptop plugged in, other apps closed, 20 s measured after a 3 s warm-up:

| Run | Godot (75 % render scale) | Rust (75 %) |
|---|---|---|
| 1 | 36.7 FPS (p95 38.0 ms) | 121.4 FPS (p95 10.0 ms) |
| 2 | 38.5 FPS (p95 36.2 ms) | 120.2 FPS (p95 9.9 ms) |
| 3 | 33.9 FPS (p95 47.6 ms) | 122.0 FPS (p95 9.9 ms) |

Rust at full resolution: 99.7 FPS (p95 12.1 ms). About **3.3x** Godot at the same render scale.
Not yet equal: the Rust zombies walk straight at the player (no AI, no physics), there is no player model or HUD. Those come in R2/R3 and the benchmark is re-run then.
