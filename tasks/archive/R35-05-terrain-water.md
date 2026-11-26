# R35-05: terrain colour detail, shore foam, river flow

Owns: world/src/terrain_paint.rs (outer biomes + a light core variation layer), engine/src/render/water.wgsl, engine/src/render/water.rs
Files: world/src/terrain_paint.rs, engine/src/render/water.wgsl, engine/src/render/water.rs
Read only: these files, rotland/src/river_view.rs, rotland/src/water_view.rs, world/tests/golden.rs (to see what it checks).

- Terrain: add patchy colour variation (two noise scales) and darker damp ground near water, lighter dry patches in desert, blue shade in snow hollows. The core may get variation ONLY if golden.rs does not check colours; if it does, outer biomes only. Grass must stay green.
- Water: a thin white foam line where water meets the shore (from depth difference, cheap), and a flowing ripple pattern on rivers along their direction (pass flow dir per vertex or use mesh UV). Lakes: gentle ripple only.
- Report anything the lead must wire.

## Worker notes
- **Files changed:**
  - `world/src/terrain_paint.rs`:
    - `world/tests/golden.rs` checks triangle `r.color` in the core, so `core_color` (`sample.outer <= 0.0`) was left untouched.
    - Added two-scale patch noise (`patch_broad` at 0.007, `patch_fine` at 0.042) for outer-biome colour and brightness variation (`alt_mix` + `±8%` patch brightness).
    - Added lighter dry patches in `Desert` (`DESERT_DRY_COLOR`), blue shade in `Snow` hollows (`tri_y < local_avg` lerping toward `SNOW_SHADOW`), and richer damp riverbanks in `paint_outer_features` (`damp = blend.powf(0.75)` darkening the underlying colour).
    - Added unit tests `desert_dry_patches_brighten_and_snow_hollows_tint_blue` and `outer_plains_grass_stays_green_with_patch_variation`.
  - `engine/src/render/water.wgsl` and `engine/src/render/water.rs`:
    - Added `WaterVertex { position: [f32; 3], flow: [f32; 2] }` and `WaterRenderer::draw_all_with_flow` (keeping `draw_all` compatible with `&[Vec3]` as still water).
    - Added `DepthCopy` in `WaterRenderer` to copy `target.depth_texture` and bind it at `@group(0) @binding(1)` (`depth_tex: texture_depth_2d`), plus `camera.near` in `lighting_params.w`.
    - In `water.wgsl`: directional downstream ripple when `length(i.flow) > 0.01` (rivers), gentle isotropic ripple when `flow == 0` (lakes/patches), and thin white shore foam from reverse-Z depth difference (`0.0..0.45 m`).
    - Added unit test `water_vertex_stores_position_and_flow`.
- **Shared-file / lead wiring needed:**
  1. `engine/src/target.rs`: Add `pub depth_texture: wgpu::Texture` to `FrameTarget` with `usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC`, and create `depth = depth_texture.create_view(&Default::default())`.
  2. `engine/src/render/mod.rs`: Export `WaterVertex` in `pub use water::{WaterPatch, WaterRenderer, WaterStyle, WaterVertex};`.
  3. `world/src/rivers/mesh.rs` & `world/src/rivers/mod.rs`: Store `flow_dirs: Vec<Vec2>` (using `fwd = vertex_tangent(&river.points, idx)` already computed in `push_cross_section`) and expose `water_mesh_with_flow_near(&self, center: Vec2, radius: f32) -> Vec<(Vec3, Vec2)>` (or `WaterVertex`).
  4. `rotland/src/river_view.rs` & `rotland/src/water_view.rs`: Pass `Vec<WaterVertex>` for rivers and call `renderer.draw_all_with_flow(...)` for the river mesh.
- **How to see it:** Run `--biome=desert`, `--biome=snow`, and `--river=0` shots and check terrain patchiness, snow hollow shading, shore foam along lake/river edges, and downstream ripples on rivers.
- **Not run:** `cargo`, `tools/gate.sh`, `blender`, and the game were not run per subagent rules.
