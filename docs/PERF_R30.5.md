# R30.5 Performance Report (Game FPS)

Hardware: Intel i5-10310U (4c/8t), Intel UHD Graphics (CML GT2, Vulkan Mesa 26.0.8), 14 GB RAM, 1920x986.

## Before (main @ bb131e7, release build)

Measured via `tools/perf_scenes.sh` (30s per scene, 2s warmup, `--save-dir=/tmp/r30.5-saves`):

| Scene | Avg FPS | 1% Low FPS | Worst Frame (ms) | p95 (ms) | Draws (Shadow) |
|---|---:|---:|---:|---:|---|
| camp | 120.3 | 45.5 | 100.0 | 11.8 | 56 (27) |
| forest | 73.4 | 24.8 | 100.0 | 23.4 | 75 (21) |
| city | 83.4 | 20.3 | 100.0 | 17.4 | 60 (34) |
| night | 141.4 | 64.3 | 105.0 | 9.7 | 63 (0) |
| zombies | 90.0 | 35.5 | 100.0 | 16.0 | 61 (22) |

### CPU vs GPU diagnosis (`forest` scene, 20s)
- Full-res (`--scale=1.0`): **73.4 avg FPS**, **24.8 1% low**, **23.4 ms p95**.
- Half-res (`--scale=0.5`): **131.0 avg FPS**, **70.7 1% low**, **9.31 ms p95**.
- **Result:** Heavily **GPU-bound** (fragment fill-rate, foliage overdraw, shadow map rendering/sampling).
- **Night vs Day (`city` at `-384,1792`):** `night` (0 shadow draws) runs at **141.4 FPS / 64.3 1% low** vs `city` (34 shadow draws) at **83.4 FPS / 20.3 1% low**.
- **Spikes (`--spikes` on `forest`):** `perf record` blocked by `kernel.perf_event_paranoid = 4`; built-in `--spikes` shows `surface acquire` stalls of **23.7–44.4 ms** (Mailbox + frame latency 2 + uncapped FPS on Intel UHD Mesa), plus frame 1 `render submit` (38.9 ms) and `physics` (24.7 ms).

## Optimisation Experiments (Kept)
1. **Swapchain frame latency (`engine/src/gpu.rs`, `engine/src/app.rs`):** Set `desired_maximum_frame_latency = 1` (was `2`) and removed redundant per-frame `Vec` allocation in `App::frame`. Kept.
2. **Shadow bandwidth & PCF (`engine/src/render/shadow.rs`, `lit.wgsl`, `lit.rs`):** Switched `SHADOW_FORMAT` from `Depth32Float` to `Depth16Unorm`, shadow map size `2048 -> 1024`, `sun_shadow()` from 4 `textureSampleCompareLevel` calls (16 taps) to 1 hardware 2x2 PCF tap, skipped shadow lookup when `sun_dir.w <= 0.001` (night), and reused scratch `Vec`s in `LitRenderer::draw`. Kept.
3. **Foliage Early-Z & render_frame allocations (`rotland/src/props.rs`, `rotland/src/app/render_frame.rs`):** Disabled dither-fade (`fade_band = 0.0`, no `discard`) on ground props (`grass_`, `flowers_`, `fern`, `mushrooms`) so Early-Z stays active on Intel UHD, recycled batch `Vec` capacities across frames (`BatchScratch`), and removed per-car `String`/`Vec` allocations in `render_frame`. Kept.
4. **Physics compound chunk colliders & static-vs-static filter (`physics/src/world.rs`, `rotland/src/streaming.rs`):** Filtered static `GROUP_WORLD` colliders to collide only with `GROUP_CHARACTER`, and combined non-tree static props per chunk into one `SharedShape::compound` collider. Kept.

## After (r30.5, release build)

| Scene | Avg FPS | 1% Low FPS | Worst Frame (ms) | p95 (ms) | Draws (Shadow) |
|---|---:|---:|---:|---:|---|
| camp | 151.4 | 77.6 | 100.0 | 8.0 | 56 (27) |
| forest | 103.4 | 55.1 | 100.0 | 13.9 | 75 (21) |
| city | 107.8 | 53.3 | 142.3 | 12.5 | 62 (30) |
| night | 155.7 | 89.0 | 100.0 | 8.3 | 62 (0) |
| zombies | 121.3 | 51.6 | 100.0 | 13.3 | 62 (21) |

- **`play` profile (`forest`, 15s):** **114.0 avg FPS**, **66.5 1% low**, **10.23 ms p95** (within target parity of `release`).
