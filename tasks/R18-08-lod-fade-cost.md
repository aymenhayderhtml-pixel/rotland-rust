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

