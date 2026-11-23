# A04 (Neon): directional movement, leaning, starts, stops and turning

Files: humanoid/src/motion.rs, humanoid/src/motion_tests.rs, data/motion.json, tools/blender/characters/poses_move.py, tasks/A04-directional-movement.md, humanoid/src/animator.rs, humanoid/src/lib.rs, rotland/src/app/state.rs, rotland/src/app/render_frame.rs, tools/blender/characters/poses.py, assets/models/characters/human_player.glb
Read: humanoid/src/blend.rs, humanoid/src/animator.rs, humanoid/src/pose_buffer.rs, tools/blender/characters/poses.py, tools/blender/characters/poses_locomotion.py
API: humanoid/src/mask.rs, humanoid/src/layers.rs, humanoid/src/aim.rs

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 4 of 20 of a deep character animation system. Done so far: layers and masks, weapon stances, shouldered aiming, recoil and reloads.

**The problem:** the legs only know forward idle, walk and run. While aiming or in first person, the body faces the camera, so walking sideways or backward makes the legs walk forward and the feet slide. The body also never leans, starts, stops or turns; it just rotates like a statue.

**How the player moves (do not change it):** when not aiming, the body turns toward the movement direction. When aiming or in first person, the body snaps to the camera yaw and can move in any direction. Walk blend point is 4 m/s and run is 7 m/s. Model forward is `(-sin(yaw), 0, -cos(yaw))`.

**After this job:** real strafing and backward walking with feet that match the ground, a lean into turns and speed changes, a small settle when stopping, and small foot steps when turning on the spot. The upper body keeps its weapon pose.

## Output rules (important)
- Output **complete files only for**: `humanoid/src/motion.rs`, `humanoid/src/motion_tests.rs`, `data/motion.json`, `tools/blender/characters/poses_move.py`.
- **Do not output** `animator.rs`, `lib.rs`, `state.rs`, `render_frame.rs` or `poses.py`. In `=== NOTES ===`, give exact edits: file, exact existing lines to find (copied exactly, unique in that file), and exact replacement. A helper applies them literally. Keep edits to `animator.rs` as small as possible, because it is already too long: all new logic belongs in `motion.rs`.
- **Keep the whole answer under 50,000 characters.** Compact tests with shared helpers. Always finish with `=== NOTES ===`.
- No new crates. No sounds. No gameplay or speed changes. No foot IK yet (job 5). Files around 300 lines. No allocation per frame.

## New clips: `tools/blender/characters/poses_move.py`
Same keyframe format as `poses_locomotion.py`; export `CLIPS`. **Legs and hips only** (`hips`, `thigh_*`, `shin_*`, `foot_*`, `toe_*`). All loop.
- `move_walk_back` (1.0 s), `move_run_back` (0.7 s)
- `move_strafe_l`, `move_strafe_r` (1.0 s): side steps, feet do not cross, hips sway a little
- `move_run_strafe_l`, `move_run_strafe_r` (0.7 s)
- `move_turn_l`, `move_turn_r` (0.6 s): small on-the-spot stepping

**Phase rule:** at time 0 of every walk-family and run-family clip the legs must be in the same phase as `walk` and `run` (right and left contact at the same fraction of the cycle), so that blending directions never makes the legs scissor. Make strafes readable, and write `_r` as a true mirror of `_l` with a small helper in Python.

## `data/motion.json`
```json
{
  "version": 1,
  "directionRate": 10.0,
  "strafeSpeedScale": 0.85,
  "backSpeedScale": 0.8,
  "minMoveSpeed": 0.25,
  "leanMaxDeg": 9.0,
  "leanPerRadPerSec": 3.5,
  "leanRate": 8.0,
  "accelLeanDeg": 6.0,
  "accelLeanPerMps2": 0.35,
  "stopSettleDeg": 3.0,
  "turnInPlaceDegPerSec": 45.0,
  "turnStepRate": 8.0,
  "maxYawRateRad": 8.0
}
```

## `humanoid/src/motion.rs`
```rust
pub struct MotionConfig { /* serde camelCase */ }
impl MotionConfig { pub fn from_json(text: &str) -> Result<Self, serde_json::Error>; pub fn load_shared() -> &'static MotionConfig; }
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DirectionWeights { pub forward: f32, pub back: f32, pub left: f32, pub right: f32 }
pub fn direction_weights(local_move: Vec2) -> DirectionWeights; // sums to 1 for a nonzero input
/// World velocity to body-local: x = right, y = forward.
pub fn local_velocity(velocity: Vec3, yaw: f32) -> Vec2;
pub struct MotionJoints { /* hips, spine, chest indices, Option each */ }
impl MotionJoints { pub fn from_names(names: &[&str]) -> Self; }
pub struct MotionState { /* smoothed direction, lean, turn, previous yaw and speed */ }
impl MotionState {
    pub fn new() -> Self;
    /// Call once per rendered frame. Until this is called the state has no effect.
    pub fn set_input(&mut self, velocity: Vec3, yaw: f32);
    pub fn tick(&mut self, dt: f32, on_floor: bool);
    pub fn weights(&self) -> DirectionWeights;
    pub fn directional_amount(&self) -> f32; // 0 when moving straight forward or idle
    pub fn turn_amount(&self) -> f32; // -1..1, nonzero only when turning on the spot
    pub fn lean_roll_deg(&self) -> f32;
    pub fn lean_pitch_deg(&self) -> f32;
    pub fn tempo_scale(&self) -> f32; // stride correction for back and side speed
    /// Blends the directional and turn clips over the lower body in `local`.
    pub fn apply_legs(&self, rig: &Rig, clips: &MotionClips<'_>, time_loco: f32, run_fraction: f32, lower_mask: &[f32], local: &mut [LocalTransform], scratch: &mut [LocalTransform]);
    /// Adds the lean on hips, spine and chest.
    pub fn apply_lean(&self, joints: &MotionJoints, local: &mut [LocalTransform]);
}
pub struct MotionClips<'a> { /* Option<&Clip> for each of the 8 clips */ }
impl<'a> MotionClips<'a> { pub fn from_map(clips: &'a std::collections::HashMap<String, Clip>) -> Self; }
```
Rules:
- Forward motion must leave the existing walk and run untouched, so with `directional_amount` 0 and no turn, `apply_legs` does nothing.
- Handle yaw wrapping at +/- PI. A body snap (yaw jump larger than what `maxYawRateRad` allows in one frame, as when aiming starts) must not cause a lean or a turn step.
- Lean only on the floor; it fades in the air. Lean direction: leans **into** the turn, forward when speeding up, back a little when slowing down, and `stopSettleDeg` is a small overshoot when stopping.
- Turn in place only when nearly stopped and yaw rate is above `turnInPlaceDegPerSec`.
- A missing clip is safe: that direction falls back to the existing leg pose.
- Frame-rate independent, finite at dt 0 and 0.1, identical positions and zero velocity safe.

## Exact edits to give in the notes
1. `poses.py`: merge the clips into `PLAYER_ONLY_CLIPS`.
2. `lib.rs`: `pub mod motion;`, `#[cfg(test)] mod motion_tests;`, exports.
3. `animator.rs`: add `motion: MotionState` and `motion_joints`; add `pub fn set_move_velocity(&mut self, velocity: Vec3, yaw: f32)`; tick it in `update`, including under the LOD throttle; multiply the locomotion tempo by `tempo_scale()`. In `recompute_skin`, after the base locomotion sample and before air, crouch, stance and aim, call `apply_legs`; apply the lean after aim and recoil and before left-hand IK. Use an existing pool buffer as the scratch. **The golden test from job 1 must still pass**, because it never calls `set_move_velocity`.
4. `rotland/src/app/state.rs`, find exactly `    pub horizontal_speed: f32,` and add after it `    pub velocity: Vec3,`. Find exactly `            horizontal_speed: p.horizontal_speed(),` and add after it `            velocity: p.velocity,`.
5. `rotland/src/app/render_frame.rs`, find exactly:
   `        // The player's yaw already uses the model's facing (checked in a screenshot: no extra turn)`
   and add before it:
   `        pv.animator.set_move_velocity(pose.velocity, pose.yaw);`

## Tests: `humanoid/src/motion_tests.rs`
Compact, hand-made skeleton and clips: `data/motion.json` parses; all 8 clips are defined in `poses_move.py` (read as text); `direction_weights` for 8 directions sum to 1; `local_velocity` for facing north and east using the yaw convention (facing east is yaw `-PI/2`); straight forward has no effect on the pose; yaw wrap; a snap causes no lean; lean sign into a left and right turn; lean fades in the air; turn only when stopped; missing clips safe; frame-rate independence; no NaN at dt 0 and zero velocity.

Every new number in Rust code needs a source tag on its line (`// data: motion.json key` or `// new: DECISIONS 2026-11-23`).

## Worker notes

## Neon notes

Summary

  - Added data/motion.json, humanoid/src/motion.rs,
    humanoid/src/motion_tests.rs, and tools/blender/characters/poses_move.py.
  - MotionState does nothing until set_input is called, keeping the job 1 golden
    test unchanged.
  - poses_move.py defines all 8 looping lower-body clips matched to walk/run leg
    contact phase at t = 0.0, with _r clips generated via _mirror_clip
    (mirroring L/R bones and Y/Z rotations + X hip sway, shifted half a cycle so
    right contact stays at 0.0).
  - Missing API note: humanoid/src/lib.rs, MaskId variants in
    humanoid/src/mask.rs, and pack::Skeleton/pack::Clip/engine::anim::Rig
    constructors were not included in the Read/API sections. motion_tests.rs
    therefore exercises the full apply_legs blending logic through
    MotionState::apply_legs_sampled with hand-made poses (zero invented APIs),
    and the animator.rs edit assumes MaskId::Lower exists in
    humanoid/src/mask.rs (verify variant name if different).

Exact edits to apply

1. tools/blender/characters/poses.py

Find:

from poses_locomotion import CLIPS as _LOCOMOTION
from poses_shared import ARM_DOWN_OFFSET  # noqa: F401  (re-exported for build_characters)
from poses_stance import CLIPS as _STANCE
from poses_zombie import CLIPS as _ZOMBIE

CLIPS = {**_LOCOMOTION, **_ACTIONS, **_COMBAT, **_ZOMBIE, **_CAMP}
# Clips only the player gets (V12-04 gun clips + A02 weapon stances + A03 aim/reload). Looked up through ALL_CLIPS, never CLIPS alone.
PLAYER_ONLY_CLIPS = {**_PLAYER_ONLY, **_STANCE, **_AIM}
ALL_CLIPS = {**CLIPS, **PLAYER_ONLY_CLIPS}

Replace with:

from poses_locomotion import CLIPS as _LOCOMOTION
from poses_move import CLIPS as _MOVE
from poses_shared import ARM_DOWN_OFFSET  # noqa: F401  (re-exported for build_characters)
from poses_stance import CLIPS as _STANCE
from poses_zombie import CLIPS as _ZOMBIE

CLIPS = {**_LOCOMOTION, **_ACTIONS, **_COMBAT, **_ZOMBIE, **_CAMP}
# Clips only the player gets (V12-04 gun clips + A02 weapon stances + A03 aim/reload + A04 directional movement). Looked up through ALL_CLIPS, never CLIPS alone.
PLAYER_ONLY_CLIPS = {**_PLAYER_ONLY, **_STANCE, **_AIM, **_MOVE}
ALL_CLIPS = {**CLIPS, **PLAYER_ONLY_CLIPS}

2. humanoid/src/lib.rs (not shown in Read; add alongside existing modules/re-exports)

Add module declarations:

pub mod motion;
#[cfg(test)]
mod motion_tests;

Add exports:

pub use motion::{
    DirectionWeights, MotionClips, MotionConfig, MotionJoints, MotionState, direction_weights,
    local_velocity,
};

3. humanoid/src/animator.rs

Edit 3a — imports: Find:

use crate::blend::{
    lod_interval, sample_locomotion_into, tempo_for, Locomotion, LocomotionClips, RUN_BLEND,
};

Replace with:

use crate::blend::{
    locomotion_weights, lod_interval, sample_locomotion_into, tempo_for, Locomotion,
    LocomotionClips, RUN_BLEND,
};

Edit 3b — motion import: Find:

use crate::mask::{BoneMasks, MaskId};
use crate::pose_buffer::{add_pose_masked, blend_pose_masked, PosePool};

Replace with:

use crate::mask::{BoneMasks, MaskId};
use crate::motion::{MotionClips, MotionJoints, MotionState};
use crate::pose_buffer::{add_pose_masked, blend_pose_masked, PosePool};

Edit 3c — struct fields: Find:

    aim: AimController,
    aim_joints: AimJoints,
    blend_pos: f32,

Replace with:

    aim: AimController,
    aim_joints: AimJoints,
    motion: MotionState,
    motion_joints: MotionJoints,
    blend_pos: f32,

Edit 3d — resolve joints in with_config: Find:

        let masks = BoneMasks::new(&joint_names);
        let aim_joints = AimJoints::from_names(&joint_names);

Replace with:

        let masks = BoneMasks::new(&joint_names);
        let aim_joints = AimJoints::from_names(&joint_names);
        let motion_joints = MotionJoints::from_names(&joint_names);

Edit 3e — initialize fields in with_config: Find:

            aim: AimController::new(),
            aim_joints,
            blend_pos: 0.0,

Replace

## Gate report
- date: 2026-11-23
- base: main
- files changed (24): assets/models/buildings/settlement.glb, assets/models/characters/human_player.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/motion.json, humanoid/src/animator.rs, humanoid/src/lib.rs, humanoid/src/motion.rs, humanoid/src/motion_tests.rs, tasks/A04-directional-movement.md, tools/blender/buildings/settlement.py, tools/blender/characters/poses_move.py, tools/blender/characters/poses.py, tools/blender/vehicles/car_parts.py
- tests passed: 0
- clippy: FAIL
- test: SKIP
- checks: SKIP
- gate: FAIL (clippy)
