# R12-02 stutter: before and after (release build, Intel UHD, 30 s runs)

Power: on AC (`AC Mains` online = 1), battery charging.
Setup: `--save-dir=/tmp/r12s --seconds=30`, `--autowalk` for the walk run. Each run is one process, one at a time under `flock /tmp/rotland-heavy.lock`.

## Numbers (stats JSON keys: avgFps, p99FrameMs, worst1PercentFps)

| Build | Run | avg FPS | p99 ms | worst 1% FPS |
|---|---|---|---|---|
| main (unchanged) | camp | 94.6 | 29.3 | 26.2 |
| main (unchanged) | walk | 127.1 | 19.9 | 32.6 |
| R12-02 worker code (spike logger only) | camp run 1 | 111.5 | 22.4 | 31.8 |
| R12-02 worker code (spike logger only) | camp run 2 | 126.1 | 16.2 | 36.4 |
| R12-02 worker code (spike logger only) | walk run 1 | 104.4 | 28.3 | 27.3 |
| R12-02 worker code (spike logger only) | walk run 2 | 131.7 | 21.1 | 35.1 |

Target (release): p99 under 20 ms and worst 1% over 50 FPS, at camp and walk. **Not reached.**
The same code gave p99 22.4 and 16.2 at camp on two runs, so run-to-run noise (about 6 ms on p99) is bigger than the before/after gap. No change in this round is a proven improvement.

Experiment (reverted): applying 1 built chunk per frame instead of 3 (`MAX_APPLIED_PER_FRAME` in `rotland/src/streaming.rs`) gave camp p99 26.9 ms and walk p99 27.6 ms. No gain, so chunk uploads are not the main cause.

## Spike causes (from `--spikes` logs, frames over 20 ms)

- Startup (first ~200 frames): chunk builds (49 chunks, 12 meshes) land in `render submit` and `ui/egui` (up to 100 ms frame at frame 1).
- Steady state (after frame 200, about 60-70% of spikes in camp and walk): 25-40 ms of each slow frame is not inside any timed phase. Probe timers added in `engine/src/app.rs` showed `game update (all)` and `surface acquire` are both about 0 ms in those frames, so the time is spent between frames (window system, compositor, or CPU contention). Slow frames come in pairs.
- Another `rotland` process was running outside the lock during these tests (PID 167118, not started by this round). That may explain part of the stalls. Re-run with nothing else running before judging the next fix.
- Summed phases over all spikes: camp = render submit 166 ms, physics 85 ms, ui/egui 98 ms, world streaming 77 ms. Walk = render submit 182 ms, ui/egui 178 ms, world streaming 188 ms, physics 150 ms.

## Not done / next step

- Find what blocks between frames (try a wall-clock timer around `event_loop` waiting and `request_redraw`, and compare with the compositor). Only then pick a fix.
- Physics `detect_collisions` runs over the whole scene on every frame where chunks change (`physics/src/world.rs`). It is the next candidate if the walk spikes stay in `physics`.

## 2026-11-22: cause found and fixed (zombie flow field)
- Probe timers at Ashford core (seed 42, `--at=-384,1792`, release): the zombie flow field rebuild (`game/src/flowfield.rs`, every 0.5 s) took **10 ms median, 15 ms max**, one frame every half second: the regular hitch the owner saw. Scan of 16k cells (all from the HashMap cache) ~3 ms + heap Dijkstra ~6 ms.
- Fix: the scan copies cells still inside the previous grid (no hash lookups), the search uses a bucket ring (Dial) instead of a heap, and the search is skipped when the grid did not move and nothing changed. Result: **0.13 ms median standing still, 2.5 ms median / 6 ms p95 walking** (new cells still need rays).
- The spike log was misleading: `total_ms` mixed this frame's work with the gap since the previous frame, so each slow frame showed twice ("pairs"). It now writes `work_ms`, `since_last_ms`, `game update (all)` and `surface acquire` separately.
- What's left in slow frames: `surface acquire` (the CPU waiting for the GPU, i.e. GPU-bound frames of 15-30 ms). Measured with Firefox decoding video in the background (shares the integrated GPU), so re-measure on a quiet laptop.
- Also seen: the zombie loop costs ~3 ms for 21 zombies at the core (move_and_slide rays); a steady cost, not a spike.
