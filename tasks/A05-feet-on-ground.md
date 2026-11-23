# A05 (Neon): feet that really stand on the ground

Files: humanoid/src/foot_ik.rs, humanoid/src/foot_ik_tests.rs, data/foot_ik.json, tasks/A05-feet-on-ground.md, humanoid/src/animator.rs, humanoid/src/lib.rs, rotland/src/player_visual.rs, rotland/src/app/render_frame.rs
Read: humanoid/src/ik.rs, humanoid/src/pose_buffer.rs, humanoid/src/animator.rs, rotland/src/player_visual.rs
API: humanoid/src/motion.rs, humanoid/src/mask.rs, engine/src/anim.rs, physics/src/types.rs

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 5 of 20 of a deep character animation system. Done so far: layers and masks, weapon stances, aiming and recoil, directional leg movement.

**The problem:** the legs are animated for perfectly flat ground. On hills, stairs, docks and curbs one foot floats in the air or sinks into the ground.

**After this job:** each foot is placed on the real surface and tilts to match it, the hips drop as needed so that both feet reach, and all of this fades smoothly. On flat ground the pose must look the same as today.

## Output rules (important)
- Output **complete files only for**: `humanoid/src/foot_ik.rs`, `humanoid/src/foot_ik_tests.rs`, `data/foot_ik.json`.
- **Do not output** `animator.rs`, `lib.rs`, `player_visual.rs` or `render_frame.rs`. In `=== NOTES ===`, give **every** exact edit: file, exact existing lines to find (copied exactly, unique in that file), and the complete replacement. A helper applies them literally. Keep `animator.rs` edits as small as possible, because it is already too long.
- **Keep the whole answer under 45,000 characters, and write the edits completely.** The last job's notes were cut off in the middle of an edit. Keep tests compact so that the notes always fit, and finish with `=== NOTES ===` content ending in the line `END OF EDITS`.
- No new crates. No sounds. No gameplay, collision or player height changes. No allocation per frame. At most 2 ground rays per frame for the player.

## `data/foot_ik.json`
```json
{
  "version": 1,
  "enabled": true,
  "rayUp": 0.6,
  "rayDown": 0.9,
  "footHeight": 0.07,
  "maxFootLift": 0.45,
  "maxHipDrop": 0.32,
  "maxFootTiltDeg": 28,
  "footRate": 16.0,
  "hipRate": 10.0,
  "weightRate": 8.0,
  "swingLiftThreshold": 0.07,
  "maxSpeedFullWeight": 4.0,
  "minWeightAtRun": 0.35
}
```

## `humanoid/src/foot_ik.rs`
```rust
pub struct FootIkConfig { /* serde camelCase */ }
impl FootIkConfig { pub fn from_json(text: &str) -> Result<Self, serde_json::Error>; pub fn load_shared() -> &'static FootIkConfig; }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundHit { pub point: Vec3, pub normal: Vec3 } // world space
pub struct FootJoints { /* hips, thigh, shin, foot for L and R, Option */ }
impl FootJoints { pub fn from_names(names: &[&str]) -> Option<Self>; }
pub struct FootIkState { /* smoothed offsets, normals, hip drop, weight, last probe points */ }
impl FootIkState {
    pub fn new() -> Self;
    /// World XZ points to probe next, from the feet of the last solved pose. Returns None when disabled or never solved.
    pub fn probe_points(&self) -> Option<[Vec3; 2]>;
    /// Stores this frame's ground under each foot, plus the character transform.
    pub fn set_ground(&mut self, left: Option<GroundHit>, right: Option<GroundHit>, char_world: Mat4);
    pub fn set_active(&mut self, active: bool); // false in air, on ladders, in cars, swimming, dead
    pub fn tick(&mut self, dt: f32, speed: f32);
    pub fn weight(&self) -> f32;
    /// Moves the hips and solves both legs in `local`. Runs after all clip layers and leans.
    pub fn apply(&mut self, joints: &FootJoints, parents: &[Option<usize>], order: &[usize], local: &mut [LocalTransform], globals: &mut [Mat4]);
}
```
Rules:
- Convert ground points and normals to model space with the inverse of `char_world`.
- For each foot: compare the animated ankle with the target height (`ground + footHeight`). A foot that is lifted above the flat-ground height by more than `swingLiftThreshold` is mid-swing, so **keep its lift** and only add the ground offset. Never push a foot under the ground.
- The hips drop by the amount needed for the lower foot to reach, up to `maxHipDrop`. Hips never rise. Moving the hips must move the whole upper body and held weapon naturally.
- Solve legs with the existing `solve_two_bone_ik` (you can see its signature), pole forward so knees bend forward. Keep the existing foot rotation and add only the ground tilt, limited to `maxFootTiltDeg`. A too-far target must clamp and stay finite.
- Offsets are smoothed with exponentials, frame-rate independent. A teleport (the character moved more than 2 m between frames) resets smoothing with no leg snap.
- The foot weight falls with speed from 1 at `maxSpeedFullWeight` to `minWeightAtRun` at full run.
- With a missing hit, that foot fades to its animated pose.
- On flat ground at the character's height, the pose must be unchanged within 1 mm. **If `set_ground` is never called, `apply` must do nothing at all**, so the job 1 golden test still passes.

## Exact edits to give in the notes
1. `lib.rs`: `pub mod foot_ik;`, `#[cfg(test)] mod foot_ik_tests;`, exports.
2. `animator.rs`: add `foot_ik: FootIkState`, `foot_joints: Option<FootJoints>`, and a reused `Vec<Mat4>` scratch sized at construction. Add `pub fn foot_ik_mut(&mut self) -> &mut FootIkState`. Tick it in `update`, with speed `blend_pos`. Active = on floor, not dying, not held pose, no full-body clip. In `recompute_skin`, run `apply` just before the left-hand IK stage, so the off hand follows when the hips move.
3. `player_visual.rs`: add
   ```rust
   pub fn update_foot_ground(&mut self, pos: Vec3, yaw: f32, ground: impl Fn(Vec3) -> Option<humanoid::foot_ik::GroundHit>)
   ```
   which builds `char_world` the same way as `pose()`, takes `probe_points()`, runs `ground` for each, and calls `set_ground`. On the first frame, use the character position for both.
4. `render_frame.rs`, find exactly:
   `        pv.animator.set_move_velocity(pose.velocity, pose.yaw);`
   and replace with:
   ```rust
        pv.animator.set_move_velocity(pose.velocity, pose.yaw);
        if pose.on_floor && !pose.first_person {
            let physics = &world.physics;
            let (up, down) = {
                let c = humanoid::foot_ik::FootIkConfig::load_shared();
                (c.ray_up, c.ray_down)
            };
            pv.update_foot_ground(pose.position, pose.yaw, |p| {
                physics
                    .raycast(Vec3::new(p.x, pose.position.y + up, p.z), Vec3::NEG_Y, up + down)
                    .map(|h| humanoid::foot_ik::GroundHit { point: h.point, normal: h.normal })
            });
        }
   ```
   If `Vec3` is not imported there, the helper will fix it.

## Tests: `humanoid/src/foot_ik_tests.rs`
Compact, with a hand-made leg skeleton: `data/foot_ik.json` parses; never-called state does nothing; flat ground leaves feet within 1 mm; one raised foot reaches the step and the hips stay; one lowered foot makes the hips drop within `maxHipDrop`; mid-swing foot keeps its lift; slope tilts the foot and stays within the limit; target out of reach stays finite; missing hit fades out; inactive fades out; teleport resets; frame-rate independence; no NaN at dt 0.

Every new number in Rust code needs a source tag on its line (`// data: foot_ik.json key` or `// new: DECISIONS 2026-11-23`).

## Worker notes
