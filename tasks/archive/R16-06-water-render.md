# R16-06: water surface drawing (engine) + a test view

Read first: `tasks/R16-common.md`, `engine/src/render/mod.rs`, `engine/src/render/lit.rs` + `lit.wgsl` (how pipelines, the camera uniform, fog and lighting are set up: match the fog and sun so water blends with the world), `engine/src/render/pipelines.rs`, `rotland/src/app/render_frame.rs` (where the lit pass runs).
Files: engine/src/render/water.rs, engine/src/render/water.wgsl, engine/src/render/mod.rs, rotland/src/water_view.rs, rotland/src/main.rs, rotland/src/app/render_frame.rs, engine/src/cli.rs, tasks/R16-06-water-render.md
Test: water

The lake (another lane is making its layout) needs a water surface. Cheap: integrated Intel GPU, must not cost more than ~1 ms.
- `engine/src/render/water.rs`: `WaterRenderer` with its own pipeline, drawn AFTER the opaque pass with alpha blending, depth test on, depth write off. Input: a list of `WaterPatch { min: Vec2, max: Vec2, level: f32 }` (one quad grid per patch, 4 m cells, made once and cached; patches get culled by the camera frustum like other draws, reuse `frustum.rs`).
- `water.wgsl`: vertex waves (2-3 sines from time, amplitude <= 0.08 m), color = shallow color -> deep color by view angle (fresnel-like, cheap), sun specular highlight, the same distance fog as `lit.wgsl` (copy the fog math). Flat-shaded look to match the low-poly style (derivative normals). Colors and wave numbers are uniforms set from Rust; put the default values in a `WaterStyle` struct with a `Default`.
- Time uniform: seconds since start, passed in each frame.
- Test view: CLI flag `--water-test` (add to `engine/src/cli.rs` like the others). When set, `rotland/src/water_view.rs` adds one 200 x 200 m patch at the camp ground height + 0.3 m, 60 m in front of the spawn, so Claude can screenshot it. Hook the draw into `render_frame.rs` with as few lines as possible.
- Tests: pure parts only (`water_` names): the patch grid has the right vertex count and covers min..max; culling skips a patch behind the camera; `WaterStyle::default()` values are in range.
Claude checks the look with `--water-test --shot`.

## Worker notes
- **Files changed**:
  - `engine/src/render/water.rs`: `WaterPatch`, `WaterStyle`, and `WaterRenderer` with cached 4 m cell quad grids, frustum culling, and separate render pass (depth test on, depth write off, alpha blending).
  - `engine/src/render/water.wgsl`: Vertex waves (3 sine waves, max amplitude <= 0.08 m), view-dependent color transition (shallow to deep via Fresnel), sun specular highlight, derivative flat-shaded normals, distance fog, and filmic tonemap.
  - `engine/src/render/mod.rs`: Re-exports `water` module and `WaterPatch`, `WaterRenderer`, `WaterStyle`.
  - `engine/src/cli.rs`: Added `--water-test` flag support in `EngineArgs`.
  - `rotland/src/water_view.rs`: Water view module implementing `init`, `test_patch` (200x200m at ground + 0.3m, 60m ahead of spawn), and `draw`.
  - `rotland/src/main.rs`: Registers `mod water_view;` and initializes with `water_view::init(args)`.
  - `rotland/src/app/render_frame.rs`: Hooks `water_view::draw` right after opaque pass `renderer.draw`.
- **How to test**:
  - `cargo test water` runs unit tests `water_grid_vertex_count_and_bounds`, `water_culling_skips_patch_behind_camera`, and `water_style_default_in_range`.
  - Visual verification: run with `--water-test --shot=out/water.png` to inspect the water rendering from spawn.
- **Wiring notes for Claude**:
  - Water surface patches are submitted as `WaterPatch { min: Vec2, max: Vec2, level: f32 }`.
  - Call `water_renderer.draw(gpu, target, encoder, camera, light, time, &[patches])` after the opaque pass.
  - `water_view::draw` is already hooked into `render_frame.rs` and will render the test patch whenever `--water-test` is passed.
- **Known issues**:
  - None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (8): engine/src/cli.rs, engine/src/render/mod.rs, engine/src/render/water.rs, engine/src/render/water.wgsl, rotland/src/app/render_frame.rs, rotland/src/main.rs, rotland/src/water_view.rs, tasks/R16-06-water-render.md
- tests passed: 3
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
