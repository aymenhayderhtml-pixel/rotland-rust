# R16-07: smooth building detail change (dithered LOD cross-fade)

Read first: `tasks/R16-common.md`, `rotland/src/props.rs` (LOD choice by distance, `ends`, `BUILDING_LOD1` = 70 m), `engine/src/render/skinned.rs` (`SkinInstance` has 3 spare `_pad` u32s), `engine/src/render/lit.wgsl` + `lit.rs` (how instances reach the shader).
Files: rotland/src/props.rs, engine/src/render/skinned.rs, engine/src/render/lit.rs, engine/src/render/lit.wgsl, engine/src/render/shadow.rs, tasks/R16-07-lod-fade.md
Test: lod

The owner sees buildings "transform" when they switch from the simple to the detailed model. Make the switch a smooth fade:
- Over a band of `FADE` (10 m) around each LOD distance, draw BOTH LODs; the outgoing one fades out and the incoming fades in, using screen-door dithering (a 4x4 Bayer pattern in the fragment shader, `discard` when the fade value is below the threshold; the two LODs use complementary patterns so there are no holes and no double-dark pixels). No alpha blending (keeps it cheap and depth-correct).
- Carry the fade (0..1, plus one bit for "complementary pattern") in the instance's spare `_pad` field (as f32 bits), default 1.0 = fully visible, so every other model is unchanged.
- Shadows: only the nearer LOD casts the shadow during the fade (no double shadows).
- Do it for buildings and trees (both have LODs); bushes too if it's the same code.
- Keep the per-frame cost tiny (only instances inside a fade band get a second draw).
- Tests (`lod_` names): pure helper `lod_fade(dist, ends, band) -> [(lod, fade, complementary); up to 2]`: outside bands one LOD at 1.0; in the band two LODs whose fades sum to 1; ends of the band match the old hard switch.
Claude checks the look in the game.

## Worker notes
- **Files changed**:
  - `rotland/src/props.rs`: Defined `FADE` (10.0 m), `LodFades`, and pure helper `lod_fade(dist, ends, band)`. Updated `PropLibrary::batches` to calculate LOD fades and push dithered instances for the main camera view when inside a fade band, while preserving single nearer-LOD casting for shadow batches. Added thorough unit tests (`lod_` prefixed).
  - `engine/src/render/skinned.rs`: Updated `SkinInstance` to encode fade and complementary flag in `_pad[0]` (defaulting to 1.0 f32 bits in `SkinInstance::new`), added `SkinInstance::with_fade` and `fade(&self)` accessor, and added attribute 9 (`Float32`) to vertex layout. Added encoding unit test.
  - `engine/src/render/lit.wgsl`: Added `lod_fade` to `SkinIn` (@location 9), `VsOut` (@location 4 with `@interpolate(flat)`), passed from `skinned_vs` (and 1.0 in `mesh_vs`), and implemented 4x4 Bayer screen-door dither discard with complementary patterns in `mesh_fs`.
- **How to test**:
  - `cargo test lod`
- **Wiring notes for Claude**:
  - Already wired into `PropLibrary::batches` in `rotland/src/props.rs` for all placed models (buildings, trees, bushes) that have distance LODs.
  - Standard `SkinInstance::new` calls default to 1.0 (fully visible, bypasses dither logic).
  - Only instances within `switch_dist +/- 5.0` m get a second draw.
  - Shadows only draw the nearer LOD without dithering.
- **Known issues**:
  - None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (4): engine/src/render/lit.wgsl, engine/src/render/skinned.rs, rotland/src/props.rs, tasks/R16-07-lod-fade.md
- tests passed: 7
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
