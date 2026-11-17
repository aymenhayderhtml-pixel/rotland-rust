# AGENTS.md — Rules for every AI working on Rotland

Read this file first. Every AI agent working on this repo follows it.

**File map:** Read `docs/CODEMAP.md` once to understand the layout and modules. After that, open **only** the files your task needs. Don't browse unrelated parts of the repo.

## The project in one line
**Rotland** is a solo, offline, 3D open-world zombie survival game built with a custom wgpu engine in Rust.

## The owner
The owner is not a programmer and does not want to learn to code. All code is written by AI.
- Explain results in short, simple English with no technical jargon.
- Never ask the owner to edit code by hand. Give exact commands or simple instructions.

## Tech stack
| Component | Details |
|---|---|
| Language & Edition | **Rust 2024** cargo workspace (`Cargo.toml`, resolver = "3") |
| Crates | `engine` (wgpu, winit, egui), `physics` (rapier3d), `game` (core gameplay logic), `world` (terrain, biomes, chunks, POIs), `pack` (binary asset formats), `baker` (.glb -> pack), `save` (save format & Godot import), `audio` (kira), `humanoid` (rigging & animation state machine), `paths` (runtime repo paths), `rotland` (main executable wiring everything together) |
| 3D models | `.glb` files made by Blender 5.3 Python scripts in `tools/blender/`, then baked into `assets/models.pack` via `cargo run --release -p baker` |
| Game data | Data files in `data/*.json` (items, recipes, zombies, loot tables, hints, etc.) |
| Toolchain & Linker | `mold` linker, `target-cpu=native` configured in `.cargo/config.toml` |
| Version control | git |

## Target hardware (always design for this)
The owner's laptop: Intel i5-10310U, **integrated Intel UHD graphics**, 14 GB RAM.
- **Minimum 30 FPS at 1080p** (hard requirement); 60 FPS is the goal.
- Low-poly stylized look: cheap shaders, no heavy post-processing, keep effects light.
- Keep zombies cheap: flow fields, pooling/reuse, throttled updates when distant.

## Code rules
1. **Small files (~300 lines).** One topic per module.
2. **Data-driven.** Numbers and stats belong in `data/*.json`, not hardcoded in Rust source.
3. **Pure, testable logic.** Put core logic in `game/` and `world/` crates; `rotland/` wires them into the game loop.
4. **Unit tests for every feature.** Pure logic crates must have thorough unit/golden tests.
5. **Clean clippy.** Code must be clean with no warnings under `cargo clippy --all-targets`.
6. **Path discovery.** Always find files using `paths::repo_path("data/...")`, never `env!("CARGO_MANIFEST_DIR")` (which bakes the build-time directory into the binary).
7. **Godot coordinate conventions.** Maintain parity with Godot conventions: yaw forward = `(-sin(yaw), -cos(yaw))` (in 3D: `(-sin yaw, 0, -cos yaw)`). Facing east (+X) is yaw `-PI/2`.
8. **Port, don't invent.** Port behavior and exact numbers from the Godot C# file named in the task. Never invent formulas, values, or numbers.

## Safety rules
- **Save directory protection:** NEVER write to the owner's Godot saves (`~/.local/share/godot/app_userdata/Rotland/saves`) or real Rust saves (`~/.local/share/rotland-rust`) in tests or automated runs. Always pass `--save-dir=<scratch folder>`.
- **Never `pkill -f`:** Never kill processes blindly.
- **Single window & flock lock:** Only one game window at a time. Run heavy commands through `flock /tmp/rotland-heavy.lock <cmd>`.
- **Muted runs:** Scripted/automated runs (`--shot`, `--seconds`, `--frames`, `--stats`) are muted.
- **The ground must stay green:** Terrain rendering and grass palette must stay green and intact across changes.

## Worker task rules
- You get one task file from `tasks/`. Work **only** on that task. Do not edit unrelated files.
- **Workers do NOT build or test.** Do not run `cargo build`, `cargo test`, `cargo clippy`, or the game. Claude merges each round and runs the single unified build and test pass.
- Fill in `## Worker notes` in your task file: list files changed, how to test, wiring notes for Claude, and known issues.
- **In a git worktree:** Commit once to your own task branch when done. Never push to remote, never touch `main`.
- **In the shared main checkout:** Never run `git commit`.
- Do not edit `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, or `docs/` files unless your task specifically instructs you to.
- If something is ambiguous or unclear, write the question under `## Worker notes` instead of guessing.

## Useful commands
Run heavy commands under the shared lock (`flock /tmp/rotland-heavy.lock`):
```bash
cargo build                         # fast dev build (target/debug/rotland)
cargo build --release               # the release build the owner plays and benchmarks use
cargo run --release -p baker        # bake all models into assets/models.pack
cargo test                          # run all unit and golden tests (headless, no window)
cargo clippy --all-targets          # lint all targets (must have 0 warnings)

# Screenshot and verification run (always pass --save-dir)
flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch_saves --shot=/tmp/x/forest.png
```
