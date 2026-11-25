# R30-03 Performance & Stutter Investigation

## Before measurements
Command:
```bash
flock /tmp/rotland-heavy.lock target/release/rotland --save-dir=/tmp/r30-saves --autowalk --seconds=40 --spikes=/tmp/r30-spikes-before.txt --stats
```
Summary:
- `ROTLAND DONE frames=4700 avgFps=117.7 p95FrameMs=14.54 worst1PercentFps=31.9`
- GPU: `Intel(R) UHD Graphics (CML GT2) (Vulkan, Mesa 26.0.8-1ubuntu0.3)`, present modes: `[Mailbox, Fifo]`
- Total logged spike rows (`total_ms > 20 ms`): **148**
- Frames > 33 ms: **26**

Phase breakdown across the 148 spike rows:
| Phase | Max (ms) | Rows > 10 ms | Notes |
|---|---:|---:|---|
| `surface acquire` | 69.05 | 76 | `gpu.surface.get_current_texture()` blocking on Mailbox + `ControlFlow::Poll` + `desired_maximum_frame_latency = 2` on Intel UHD Mesa |
| `render submit` | 44.58 | 2 | Frame 1 (44.58 ms) & Frame 6 (17.38 ms) — initial pipeline/mesh upload |
| `physics` | 39.23 | 1 | Frame 462 (39.23 ms) — `update_sim` in `rotland/src/app/frame_sim.rs` |
| `world streaming` | 25.56 | 3 | Frames 3022 (25.56 ms), 3023 (19.88 ms), 3021 (15.76 ms) |
| `ui/egui` | 20.37 | 2 | Frame 1 (20.37 ms) & Frame 2754 (20.26 ms) |
| `GPU wait/present` | 6.61 | 0 | — |
| `render` | 3.22 | 0 | — |

Top 10 worst frames (before):
1. frame 1: `total=100.00ms` (`rsub=44.58`, `ui=20.37`, `gwait=6.61`, `chunk=49`, `mesh=12`)
2. frame 6: `total=89.36ms` (`rsub=17.38`)
3. frame 5: `total=74.89ms` (`acq=69.05`)
4. frame 2: `total=73.39ms` (`work=4.86`, `since_last_ms=73.39`)
5. frame 3021: `total=72.09ms` (`stream=15.76`, `ui=5.22`, `phys=4.04`)
6. frame 3020: `total=71.93ms` (`acq=66.26`)
7. frame 3210: `total=57.83ms` (`work=7.32`, `since_last_ms=57.83`)
8. frame 3209: `total=57.60ms` (`acq=49.28`)
9. frame 3240: `total=56.29ms` (`work=6.96`, `since_last_ms=56.29`)
10. frame 3239: `total=56.21ms` (`acq=50.14`)

## Root causes identified
1. **Streaming broad-phase thrashing (`physics/src/world.rs`, `rotland/src/streaming.rs`):**
   `PhysicsWorld::add_ground_chunk`, `add_static`, and `remove` each called `self.update_broad_phase()` (`rapier3d::pipeline::PhysicsWorld::detect_collisions`) on every single collider. Uploading 3 chunks in one frame (`MAX_APPLIED_PER_FRAME = 3`) triggered 100–300+ full broad-phase rebuilds in a single frame, even though `Streamer::update` already calls `physics.update_broad_phase()` once at the end.
   - **Fix:** Added `add_ground_chunk_deferred`, `add_static_deferred`, `add_static_tagged_deferred`, and `remove_deferred` to `PhysicsWorld`, updated `Streamer` to use them during chunk load/unload (keeping the single `physics.update_broad_phase()` at the end of `Streamer::update`), and lowered `MAX_APPLIED_PER_FRAME` from `3` to `1` to spread chunk uploads across frames.
2. **Surface acquire stalls (`surface acquire` = 76 of 148 spikes, up to 69.05 ms):**
   With `ControlFlow::Poll` (`engine/src/app.rs`) and `PresentMode::Mailbox` + `desired_maximum_frame_latency = 2` (`engine/src/gpu.rs`, not owned by this task), the CPU submits frames at ~118 FPS while Intel UHD Mesa's swapchain/compositor periodically blocks in `get_current_texture()`. Left unchanged to avoid altering uncapped frame pacing or touching unowned `engine/src/gpu.rs`.
3. **Frame 462 `physics` spike (39.23 ms):**
   Recorded by `PhaseScope::new("physics")` around `update_sim` in `rotland/src/app/frame_sim.rs` (not owned by this task), with `new chunk built = 0`.

## After measurements
Command:
```bash
flock /tmp/rotland-heavy.lock target/release/rotland --save-dir=/tmp/r30-saves --autowalk --seconds=40 --spikes=/tmp/r30-spikes-after.txt --stats
```
- `ROTLAND DONE frames=5408 avgFps=138.4 p95FrameMs=10.29 worst1PercentFps=54.2`
- Total logged spike rows (`total_ms > 20 ms`): **39** (down from 148)
- Frames > 33 ms: **10** (down from 26)
- Worst frame ms: **152.54 ms** (frame 1 startup upload); worst post-startup frame: **64.68 ms** / **64.33 ms** (frame 393 `physics=61.28 ms`), `world streaming` max **2.80 ms** (down from 25.56 ms)
