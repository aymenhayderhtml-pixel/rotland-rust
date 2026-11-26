# R34-03: speed (after the lead's Phase 0)

Owns: whatever the lead's measurement points to, inside engine/src/render/*, rotland/src/*, world/src/scatter.rs. Tell the lead which files you changed.
Files: engine/src/render/*, rotland/src/*, world/src/scatter.rs, docs/PERF_R34.md

Goal: forest scene back to >= 115 FPS avg (R31 was 121, now ~105) without making the world look emptier. Camp must not get slower.
Read ONLY the files the lead's numbers point to. Typical wins, in order: fewer draw calls for scattered props (instancing/batching per model), shadow casters culled by distance (small props cast no shadow beyond ~40 m), cheaper far trees (LOD). Every tuning number gets `// new: PERF R34`.
Write in `docs/PERF_R34.md` what you changed and why.

## Worker notes
- **Files changed:**
  - `engine/src/render/lit.wgsl`: early-out in `sun_shadow()` when `dist >= scene.shadow.w` (skips `light_view_proj` transform and PCF texture lookup beyond shadow distance).
  - `engine/src/render/lit.rs`: replaced per-frame `split_batches()` heap allocations in `LitRenderer::draw()` with reused `normal_ranges` and `fade_ranges` scratch `Vec`s and `record_skinned_ranges()`.
  - `rotland/src/props.rs`: added `max_range_sq` early-out before `sqrt()`/`lod_fade()`, tightened 3-LOD tree switch distances (`TREE_PERF_LOD1 = 35.0`, `TREE_PERF_LOD2 = 75.0`) while keeping 170 m visibility, added sun-frustum culling (`sun_frustum: Option<&Frustum>`), capped small-prop shadow distance to `35.0` m (`SMALL_PROP_SHADOW_DIST`), and switched 3-LOD tree shadow casters beyond `15.0` m (`TREE_SHADOW_LOD1_DIST`) to `LOD1` via `shadow_lod_for()` (with unit test `test_shadow_lod_for_trees_and_buildings`).
  - `rotland/src/app/render_frame.rs`: moved `lighting` computation before `render.library.batches(...)` and passed `sun_frustum.as_ref()` (only `Some` when `shadows_on` is true).
  - `docs/PERF_R34.md`: appended `## Changes` section.
- **Shared-file lines needed:** None.
- **How to see it:** Run `cargo build --profile play` + `tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland` and `forest` 10s `--stats=/tmp/r34/forest.json`; verify `forest` avg FPS >= 115, `shadowTriangles` drops sharply, and `camp` is not slower.
- **Unsure / not run:** Did not run `cargo`, `tools/gate.sh`, or the game per subagent rules.
