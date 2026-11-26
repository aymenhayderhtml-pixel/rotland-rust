# Performance log

Frame-rate results for each speed round, newest first. The target is at least 30 FPS at 1080p on the owner's laptop (Intel i5-10310U, Intel UHD graphics), with 60 FPS as the goal.

How the numbers are made:
- Scene runs come from `tools/perf_scenes.sh` (the play build, `target/play/rotland`), one scene at a time, after a warm-up.
- "Avg FPS" is the average frame rate. "1% Low FPS" is the average of the slowest 1% of frames (a stutter measure).
- Older rounds (R30, R30.5) used the release build and other scene lengths; the table notes say which.

## R37 (2026-11-26, play build, 20 s per scene)

| Scene | Avg FPS | 1% Low FPS |
|---|---:|---:|
| camp | 108.1 | 71.1 |
| forest | 74.7 | 44.1 |
| city | 68.4 | 16.8 |
| night | 93.0 | 23.8 |
| zombies | 86.0 | 43.6 |

## R35 (all-out detail, `tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland`)

### Before (R34 baseline)

| Scene | Avg FPS | 1% Low FPS | Worst Frame (ms) | p95 (ms) | Draws (Shadow) |
|---|---|---|---|---|---|
| camp | 137.6 | 33.7 | 136.8 | 9.9 | 58 (24) |
| forest | 106.3 | 45.0 | 165.8 | 12.9 | 71 (19) |
| city | 103.4 | 31.7 | 241.7 | 15.3 | 61 (26) |
| night | 127.3 | 37.9 | 158.4 | 10.9 | 60 (0) |
| zombies | 93.8 | 32.7 | 106.4 | 16.6 | 51 (18) |
| desert | 162.6 | 44.6 | 138.9 | 10.2 | 39 (14) |
| snow | 133.4 | 31.3 | 148.8 | 13.0 | 48 (18) |
| swamp | 141.8 | 42.4 | 144.6 | 11.8 | 49 (20) |
| mountains | 131.5 | 44.2 | 164.9 | 12.0 | 45 (19) |
| town_desert | 125.0 | 42.4 | 138.7 | 12.6 | 50 (20) |
| town_snow | 115.5 | 43.8 | 249.3 | 13.9 | 47 (23) |

### After (R35 all-out detail, tuned `data/ground_detail.json`)

| Scene | Avg FPS | 1% Low FPS | Worst Frame (ms) | p95 (ms) | Draws (Shadow) |
|---|---|---|---|---|---|
| camp | 123.4 | 47.6 | 115.9 | 12.1 | 60 (24) |
| forest | 86.9 | 34.1 | 155.7 | 15.7 | 71 (19) |
| city | 97.7 | 34.2 | 166.9 | 16.2 | 62 (26) |
| night | 128.0 | 56.4 | 154.2 | 11.4 | 61 (0) |
| zombies | 68.7 | 25.7 | 109.4 | 21.3 | 52 (18) |
| desert | 121.7 | 34.6 | 216.5 | 14.2 | 40 (14) |
| snow | 113.1 | 36.3 | 148.7 | 15.7 | 46 (19) |
| swamp | 93.0 | 27.7 | 172.3 | 18.7 | 52 (21) |
| mountains | 97.9 | 29.1 | 194.3 | 16.9 | 48 (19) |
| town_desert | 107.7 | 37.9 | 186.0 | 15.4 | 65 (33) |
| town_snow | 75.3 | 26.5 | 233.2 | 21.9 | 59 (33) |

## R34 (forest and culling speed work, `target/play/rotland`, 20 s per scene)

### Before (Phase 0 baseline)

| Scene | Avg FPS | 1% Low FPS | Worst Frame (ms) | p95 (ms) | Draws (Shadow) |
|---|---|---|---|---|---|
| camp | 132.7 | 52.8 | 100.0 | 11.7 | 56 (27) |
| forest | 86.7 | 25.6 | 100.0 | 20.7 | 75 (21) |
| city | 100.5 | 24.9 | 153.7 | 16.6 | 61 (31) |
| night | 146.2 | 68.9 | 214.4 | 9.4 | 61 (0) |
| zombies | 102.0 | 37.8 | 103.9 | 14.7 | 53 (22) |
| desert | 169.8 | 58.2 | 120.2 | 8.9 | 37 (14) |
| snow | 139.6 | 40.5 | 130.4 | 11.2 | 43 (18) |
| swamp | 154.5 | 47.6 | 169.1 | 10.1 | 49 (21) |
| mountains | 143.8 | 47.6 | 136.6 | 10.2 | 46 (20) |
| town_desert | 140.9 | 53.3 | 128.1 | 11.0 | 51 (22) |
| town_snow | 120.8 | 45.2 | 133.6 | 12.8 | 48 (23) |

### After (post-gate)

| Scene | Avg FPS | 1% Low FPS | Worst Frame (ms) | p95 (ms) | Draws (Shadow) |
|---|---|---|---|---|---|
| camp | 137.6 | 33.7 | 136.8 | 9.9 | 58 (24) |
| forest | 106.3 | 45.0 | 165.8 | 12.9 | 71 (19) |
| city | 103.4 | 31.7 | 241.7 | 15.3 | 61 (26) |
| night | 127.3 | 37.9 | 158.4 | 10.9 | 60 (0) |
| zombies | 93.8 | 32.7 | 106.4 | 16.6 | 51 (18) |
| desert | 162.6 | 44.6 | 138.9 | 10.2 | 39 (14) |
| snow | 133.4 | 31.3 | 148.8 | 13.0 | 48 (18) |
| swamp | 141.8 | 42.4 | 144.6 | 11.8 | 49 (20) |
| mountains | 131.5 | 44.2 | 164.9 | 12.0 | 45 (19) |
| town_desert | 125.0 | 42.4 | 138.7 | 12.6 | 50 (20) |
| town_snow | 115.5 | 43.8 | 249.3 | 13.9 | 47 (23) |

## R30.5 (speed round, release build, 30 s per scene; play profile checked separately)

Hardware: Intel i5-10310U, Intel UHD Graphics (Vulkan, Mesa), 14 GB RAM, 1920x986.

### Before (main, release build)

| Scene | Avg FPS | 1% Low FPS | Worst Frame (ms) | p95 (ms) | Draws (Shadow) |
|---|---:|---:|---:|---:|---|
| camp | 120.3 | 45.5 | 100.0 | 11.8 | 56 (27) |
| forest | 73.4 | 24.8 | 100.0 | 23.4 | 75 (21) |
| city | 83.4 | 20.3 | 100.0 | 17.4 | 60 (34) |
| night | 141.4 | 64.3 | 105.0 | 9.7 | 63 (0) |
| zombies | 90.0 | 35.5 | 100.0 | 16.0 | 61 (22) |

Diagnosis: the forest scene is GPU-bound. At half resolution (`--scale=0.5`) it went from 73.4 to 131.0 avg FPS (1% low 24.8 to 70.7). Night (no shadow draws) runs much faster than city in the same spot.

### After (release build)

| Scene | Avg FPS | 1% Low FPS | Worst Frame (ms) | p95 (ms) | Draws (Shadow) |
|---|---:|---:|---:|---:|---|
| camp | 151.4 | 77.6 | 100.0 | 8.0 | 56 (27) |
| forest | 103.4 | 55.1 | 100.0 | 13.9 | 75 (21) |
| city | 107.8 | 53.3 | 142.3 | 12.5 | 62 (30) |
| night | 155.7 | 89.0 | 100.0 | 8.3 | 62 (0) |
| zombies | 121.3 | 51.6 | 100.0 | 13.3 | 62 (21) |

Play profile (`forest`, 15 s): 114.0 avg FPS, 66.5 1% low, 10.23 ms p95.

## R30 (stutter investigation, release build, 40 s autowalk run)

Measured with the built-in `--spikes` log and the `--stats` summary.

| Measure | Before | After |
|---|---:|---:|
| Avg FPS | 117.7 | 138.4 |
| Worst 1% FPS | 31.9 | 54.2 |
| p95 frame (ms) | 14.54 | 10.29 |
| Frames over 20 ms | 148 | 39 |
| Frames over 33 ms | 26 | 10 |
| Max "world streaming" phase (ms) | 25.56 | 2.80 |

Main causes fixed: the physics broad phase was rebuilt once per collider during chunk loading (now deferred to one rebuild per frame), and chunk uploads were spread out (1 per frame). The remaining slow frames came from swapchain acquire stalls on the Intel driver, which were left alone.
