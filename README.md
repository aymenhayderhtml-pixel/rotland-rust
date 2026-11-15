# Rotland in Rust (custom engine on wgpu)

Plan: `../docs/RUST_PLAN.md`. The Godot game in the parent folder stays the reference until parity.

## Commands (always through the shared lock: `flock /tmp/rotland-heavy.lock <command>`)
```bash
cargo build                         # fast dev build (target/debug/rotland)
cargo build --release               # the build the owner plays and benchmarks use
cargo run --release -p baker        # bake all models into rust/assets/models.pack
cargo test                          # all unit and golden tests (no window)
cargo clippy --all-targets          # must have no warnings in new code
```

## Seeing and measuring the game (made for AI workers)
Every run can be scripted, photographed and measured. Output lines start with `ROTLAND` so they are easy to grep.

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

Example: `flock /tmp/rotland-heavy.lock target/debug/rotland --shot=/tmp/x/forest.png` then open the PNG.

## Layout
| Crate | What |
|---|---|
| `engine` | window, input, GPU, frame image + blit (render scale), screenshots, stats, command line |
| `rotland` | the game program |
| `pack` | model pack binary format, data types and serialization (postcard) |
| `baker` | tool: .glb models -> compact GPU-ready binary pack (rust/assets/models.pack) |
| `world` | generator, ground paint, chunks, scatter, POIs, settlements (pure, testable) |
| `physics` | rapier3d collision (terrain, static props/buildings) and kinematic character mover |
| `game` | player controller, camera rig, input actions, gameplay logic (pure, testable) |
| `humanoid` | humanoid animation state machine, locomotion blend, one-shot action layering, weapon attachment |
More crates (`ui`) are added by the phases in the plan.

## Controls (play scene)
- WASD: Move (camera-relative)
- Shift: Sprint (forward only, drains stamina)
- Space: Jump (drains stamina)
- C: Crouch
- V: Toggle 1st / 3rd person camera
- Mouse: Look (yaw / pitch)
- Mouse wheel: Zoom camera (3rd person)
- Right click: Aim (zoom in, narrower FOV)
- Esc: Release mouse cursor (click window to capture again)

## Rules for workers
- Read `../AGENTS.md`, then this file, then only the files your task names.
- Port behaviour from the C# file the task names; keep the same numbers and the same data files (`../data`).
- Pure logic goes in a crate with `cargo test` tests; compare with golden files from the Godot version where the task gives them.
- After any visual change: take a `--shot` and look at it. After any speed change: a `--stats` run before and after, same options.
- Small files (aim under 300 lines), comments say *why*. `cargo build`, `cargo test` and `cargo clippy` must pass.
- Each checkout uses its **own** `target/` build folder (do not set CARGO_TARGET_DIR to another checkout's): a shared build folder reused programs built in deleted checkouts.
- Find project files at run time with `paths::repo_path("data/...")`, never with `env!("CARGO_MANIFEST_DIR")` (that bakes in the folder the program was built in).
