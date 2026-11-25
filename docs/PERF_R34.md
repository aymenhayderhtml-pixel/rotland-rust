# R34 Performance Log

## Before (Phase 0 baseline, `target/play/rotland`, 20s per scene)

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

## Changes

1. **Sun-frustum & distance culling for prop shadows (`rotland/src/props.rs`, `rotland/src/app/render_frame.rs`):**
   - Passed the sun's orthographic `Frustum` into `PropLibrary::batches` (and `None` at night when `sun_energy <= 0.01`), culling all shadow-casting props outside the 60x60 m shadow box (including behind the camera) instead of submitting a full ~58 m sphere around the camera.
   - Capped small-prop (`!building && lods.len() < 3`) shadow distance at `35.0` m (`SMALL_PROP_SHADOW_DIST`).
   - Switched 3-LOD tree shadow casters beyond `15.0` m (`TREE_SHADOW_LOD1_DIST`) to `LOD1` instead of full-detail `LOD0`.
2. **Tighter tree LOD switch distances & early distance-squared cull (`rotland/src/props.rs`):**
   - Early-out `dist_sq >= max_range_sq` before `sqrt()`, `lod_fade()`, and AABB frustum tests across the ~6,200 placed props per frame.
   - Tightened 3-LOD tree switch distances to `35.0` m (`TREE_PERF_LOD1`) and `75.0` m (`TREE_PERF_LOD2`) while keeping full `170.0` m max visibility so the forest stays just as dense.
3. **Fragment shadow early-out beyond shadow distance (`engine/src/render/lit.wgsl`):**
   - Returned `1.0` in `sun_shadow()` when `dist >= scene.shadow.w` before computing `light_view_proj` and `textureSampleCompareLevel`.
4. **Zero-allocation skinned batch splitting (`engine/src/render/lit.rs`):**
   - Replaced per-frame `split_batches()` heap allocations in `LitRenderer::draw()` with reused `normal_ranges` / `fade_ranges` scratch buffers.

## After (post-gate, `target/play/rotland`, 20s per scene)

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
