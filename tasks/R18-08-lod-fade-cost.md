# R18-08: make the LOD fade cheap (only fading instances use the dither shader)

Read first: `tasks/R18-common.md`, `engine/src/render/lit.wgsl` + `lit.rs` + `skinned.rs` + `pipelines.rs` + `shadow.rs`, `rotland/src/props.rs` (`lod_fade`, batches).
Files: engine/src/render/*, rotland/src/props.rs, tasks/R18-08-lod-fade-cost.md
Test: lod

R16-07 put a `discard` (screen-door dither) into the main lit fragment shader that EVERY mesh uses. On Intel integrated GPUs any `discard` in a shader turns off early depth testing for the whole pipeline, so all buildings, trees and the ground pay for it even when nothing fades. The city core FPS dropped after that change.
- Make two variants of the instanced lit pipeline: the normal one with NO discard (used for every instance with fade 1.0), and a "fading" one with the dither discard (used only for instances inside a fade band). Batches split by variant; the shader source can be one file with a `const`/override or two entry points (`fs_main` and `fs_main_fade`).
- The ground/terrain and skinned characters never use the fade variant.
- Shadow pass: no discard at all (only the nearer LOD casts, as today).
- Keep the visual result identical. Tests: pure batch-splitting helper (instances with fade 1.0 go to the normal batch, others to the fade batch).
Claude measures FPS before/after at the city core with a quiet laptop.

## Worker notes
- Files changed:
  - `engine/src/render/lit.wgsl`: Extracted lighting logic into `shade_fragment(i: VsOut) -> vec4<f32>`. Split fragment stage into `mesh_fs` (NO discard, preserves early-Z) and `mesh_fs_fade` (dither discard using 4x4 Bayer matrix for fading instances).
  - `engine/src/render/pipelines.rs`: Added `skinned_fade` pipeline to `Pipelines` using `mesh_fs_fade`.
  - `engine/src/render/skinned.rs`: Added `SkinInstance::is_fading` and pure batch-splitting helpers `split_instances` and `split_batches`. Added unit tests for fade detection, instance splitting, and batch splitting.
  - `engine/src/render/mod.rs`: Re-exported `split_batches` and `split_instances`.
  - `engine/src/render/lit.rs`: Updated `LitRenderer::draw` to split instanced batches via `split_batches`, packing instances contiguously into the instance buffer, drawing solid instances with `pipes.skinned` and fading instances with `pipes.skinned_fade`. Factored draw recording into `record_items` and `record_skinned`.
  - `rotland/src/props.rs`: Added unit test `test_lod_batch_splitting` verifying instance classification and batch splitting.
- How to test:
  - `tools/gate.sh --task tasks/R18-08-lod-fade-cost.md` (10 tests pass, filter `lod`).
- Wiring notes for Claude:
  - None needed. `LitRenderer::draw` and `PropLibrary::batches` preserved their exact public signatures, so `render_frame.rs` and examples work unchanged.
- Known issues:
  - None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (7): engine/src/render/lit.rs, engine/src/render/lit.wgsl, engine/src/render/mod.rs, engine/src/render/pipelines.rs, engine/src/render/skinned.rs, rotland/src/props.rs, tasks/R18-08-lod-fade-cost.md
- tests passed: 10
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
