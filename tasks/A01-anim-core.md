# A01 (Neon): animation core, the foundation for a deep character animation system

Files: humanoid/Cargo.toml, humanoid/src/lib.rs, humanoid/src/animator.rs, humanoid/src/blend.rs, humanoid/src/action.rs, humanoid/src/anim_config.rs, humanoid/src/mask.rs, humanoid/src/pose_buffer.rs, humanoid/src/layers.rs, humanoid/src/full_body.rs, humanoid/src/clip_events.rs, humanoid/src/core_tests.rs, engine/src/anim.rs, data/animation.json, tasks/A01-anim-core.md
Read: humanoid/Cargo.toml, humanoid/src/lib.rs, humanoid/src/animator.rs, humanoid/src/blend.rs, humanoid/src/action.rs, humanoid/src/ik.rs, humanoid/src/swing.rs, engine/src/anim.rs
API: humanoid/src/weapon.rs, rotland/src/player_visual.rs, pack/src/skeleton.rs

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 1 of 20 that build a deep character animation system: weapon stances, aiming, directional movement, foot placement, ladders, getting in cars, wounds, many deaths, and later finger bones. **This job builds only the foundation. The game must look and play exactly the same after it**, except that transitions may be slightly smoother. Do not add new clips or new gameplay.

Output complete files for everything you change. You may edit only the paths in `Files:`. You may add the existing workspace crates `serde` and `serde_json` to `humanoid/Cargo.toml` (written as `serde.workspace = true`, `serde_json.workspace = true`). No other new crates. No sounds. Keep files around 300 lines. The target is a weak laptop with integrated graphics.

## What exists today
`HumanoidAnimator` blends idle/walk/run, jump, crouch, an upper-body aim clip and one upper-body one-shot action, with a procedural two-handed swing and left-hand IK. It has one big `recompute_skin`, allocates a new `Vec` for every sampled clip, and has no general idea of layers, masks or full-body moves.

## Keep working (hard requirement)
- Every existing public item of the `humanoid` crate keeps its name and signature, including all the `pub use` lines in `lib.rs`, and every method on `HumanoidAnimator`. `rotland/` must compile without any edit.
- Existing tests in `humanoid/src/lib.rs` and `humanoid/tests/two_hand_swing_tests.rs` must pass unchanged.
- `Rig::local_pose` keeps working. The `melee_2h` procedural swing and `apply_two_hand_left_grip_ik` behave the same. LOD update throttling and the `dirty` skin cache behave the same.
- Locomotion numbers and `ACTION_FADE_IN`, `ACTION_FADE_OUT`, `BLEND_RATE` stay the same by default, and the existing constants remain exported. `data/animation.json` holds the same values, so changing the file changes the feel.
- The game must still work if `data/animation.json` is missing or broken: use built-in defaults and `log::warn!` once. `HumanoidAnimator::new` keeps its signature and loads config through a cached `OnceLock`, using `paths::repo_path("data/animation.json")`. Tests must be able to use `AnimConfig::default()` directly without files.

## New pieces (exact names)

### `engine/src/anim.rs`
Add `pub fn local_pose_into(&self, clip: &Clip, time: f32, out: &mut [(Vec3, Quat, Vec3)])` (same result as `local_pose`, no allocation) and `pub fn rest_pose(&self) -> &[(Vec3, Quat, Vec3)]`. `local_pose` should call the new function. Nothing else changes.

### `mask.rs`
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaskId { Full, Upper, Lower, Spine, Arms, LeftArm, RightArm, Head }
pub struct BoneMasks { /* one weight per joint per mask */ }
impl BoneMasks {
    pub fn new(joint_names: &[&str]) -> Self;
    pub fn weights(&self, id: MaskId) -> &[f32];
}
```
Weights are 0.0..=1.0 so that later jobs can feather a mask across the spine. `Upper` must match `UPPER_BONES` exactly so the look does not change. Unknown bone names get weight 0, except in `Full`. Must work on a skeleton missing some bones.

### `pose_buffer.rs`
```rust
pub type LocalTransform = (Vec3, Quat, Vec3);
pub struct PosePool { /* reusable pose buffers */ }
impl PosePool {
    pub fn new(joint_count: usize, buffers: usize) -> Self;
    pub fn resize(&mut self, joint_count: usize);
}
pub fn blend_pose_masked(base: &mut [LocalTransform], over: &[LocalTransform], weight: f32, mask: &[f32]);
pub fn add_pose_masked(base: &mut [LocalTransform], delta: &[LocalTransform], rest: &[LocalTransform], weight: f32, mask: &[f32]);
```
`blend_pose_masked` must use the shortest quaternion path, normalize, and match the current `blend_local_poses` result when mask weights are 1. Additive is for later breathing, recoil and flinches.

### `layers.rs`
A small ordered list of blend layers replacing the fixed sequence in `recompute_skin`:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerId { Locomotion, Air, Crouch, Stance, Aim, Action, Additive, FullBody }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerBlend { Override, Additive }
pub struct AnimLayer { pub id: LayerId, pub mask: MaskId, pub blend: LayerBlend, pub weight: f32, pub target: f32, pub rate: f32, /* clip + time state */ }
pub struct LayerStack { /* fixed order, no allocation per frame */ }
impl LayerStack {
    pub fn new(cfg: &AnimConfig) -> Self;
    pub fn get(&self, id: LayerId) -> &AnimLayer;
    pub fn get_mut(&mut self, id: LayerId) -> &mut AnimLayer;
    pub fn set_clip(&mut self, id: LayerId, clip: Option<&str>);
    pub fn advance(&mut self, dt: f32);
}
```
Clip names must not allocate each frame (only when changed). `Stance` and `Additive` start empty and are used by later jobs.

### `full_body.rs`
Later jobs use this for climbing, car entry, deaths and vaults.
```rust
pub struct FullBodyOptions { pub speed: f32, pub looping: bool, pub hold_last: bool, pub fade_in: f32, pub fade_out: f32 }
impl Default for FullBodyOptions { /* from config */ }
pub struct FullBodyState { /* clip, time, phase, crossfade from the previous full-body clip */ }
```
Add to `HumanoidAnimator`: `pub fn play_full_body(&mut self, clip: &str, opts: FullBodyOptions) -> bool` (false when the clip does not exist), `pub fn stop_full_body(&mut self, fade_out: Option<f32>)`, `pub fn is_full_body_active(&self) -> bool`, `pub fn full_body_progress(&self) -> f32` (0..1). Full-body plays above all other layers and fades smoothly in and out, and from one full-body clip to another. `hold_pose`, `release_pose`, `play_die` and `reset_alive` keep their behaviour; `play_die` should now fade in with the configured `dieFadeIn` (0.15) instead of snapping.

### `clip_events.rs`
Timed markers so later jobs can do things like "the magazine comes out" or "foot touches the ground".
```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ClipEvent { pub clip: String, pub name: String, pub time: f32 }
pub struct ClipEventTable { /* per clip, sorted by time, from data */ }
```
Add `pub fn drain_events(&mut self, out: &mut Vec<ClipEvent>)` to `HumanoidAnimator`. Events fire exactly once when an action or full-body clip passes them, including a large `dt`, a clip with speed other than 1, and each wrap of a looping clip. Must not lose events while LOD throttling is on. Events for clips that do not exist are ignored. No allocation in the update when a clip has no events. The buffer must have a fixed limit so that it cannot grow forever if nobody drains it.

### `anim_config.rs` and `data/animation.json`
```rust
pub struct AnimConfig { /* serde camelCase */ }
impl Default for AnimConfig { /* same values as today */ }
impl AnimConfig { pub fn from_json(text: &str) -> Result<Self, serde_json::Error>; pub fn load_shared() -> &'static AnimConfig; }
```
JSON keys: `version` (1), `blendRate` 8.0, `actionFadeIn` 0.05, `actionFadeOut` 0.15, `dieFadeIn` 0.15, `fullBodyFadeIn` 0.15, `fullBodyFadeOut` 0.2, `events` `{}`. Every key is optional, so future versions stay compatible.

### `animator.rs`
Rewrite `recompute_skin` as a clear pipeline using the pieces above: base locomotion → air → crouch → stance → aim → action or swing → additive → left-hand IK → full body → skin. It must not allocate during normal frames, apart from the first use. If `ik.rs` or `swing.rs` need a `Vec`-style slice, pass slices. Split the file so `animator.rs` stays near 300 lines. `ik.rs` and `swing.rs` are not in `Files:`, so do not change them; call them as they are. If their signature forces an allocation, keep that small allocation only while they are active and say so in the notes.

## Tests: `humanoid/src/core_tests.rs` (added with `#[cfg(test)] mod core_tests;`)
Build small skeletons and clips by hand; no asset files. Cover:
- `local_pose_into` equals `local_pose`.
- Masks: `Upper` equals `UPPER_BONES`; missing bones are safe.
- `blend_pose_masked` at weights 0, 1 and 0.5; shortest path with a negated quaternion; matches `blend_local_poses`.
- Additive with rest delta changes nothing.
- A golden test: for several speeds, air, crouch, aim and action states, the new pipeline's skin equals the old math within 1e-4 (write the old math as a small helper in the test).
- Full body: fade in, hold last, finishing, crossfade between two clips, missing clip returns false, `stop_full_body`.
- `play_die` fade; `reset_alive` clears it.
- Events: once each, large `dt`, speed 2, looping wraps, LOD throttle, the buffer limit.
- Config: defaults match the constants; empty JSON `{}` loads; broken JSON gives an error; `data/animation.json` in the repo parses.
- No NaN for `dt` of 0, 0.001 and 0.5.

Every new number in Rust code needs a source tag on its line (`// data: animation.json key`, `// godot: File.cs:line` if it is an existing ported value, or `// new: DECISIONS 2026-11-23`).

## Notes to write
In `=== NOTES ===` list anything not done, any allocation that remains in the frame path and why, and what A02 (weapon stances) should call.

## Worker notes
