# A10 (Neon): walking up and down stairs and steep slopes

Files: humanoid/src/incline.rs, humanoid/src/incline_tests.rs, data/incline.json, tasks/A10-stairs-slopes.md, humanoid/src/animator.rs, humanoid/src/lib.rs, rotland/src/app/render_frame.rs
Read: humanoid/src/animator.rs, humanoid/src/swim.rs, humanoid/src/layers.rs, tools/blender/characters/poses_climb.py
API: humanoid/src/mask.rs, humanoid/src/foot_ik.rs, humanoid/src/posture.rs

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 10 of 20 of a deep character animation system. Done so far: layers, stances, aiming, directional legs, foot IK, jump/land (`airborne.rs`), crouch/sprint/tired (`posture.rs`), swimming/wading (`swim.rs`), ladders (`ladder_anim.rs`). Copy the structure of `swim.rs` (config with `from_json` + `load_shared`, a small state with a `driven` flag, helpers that write into the layers so `animator.rs` gets only a few lines).

**The problem:** going up stairs or a steep hill, the legs play the flat walk and the feet are only pulled up by foot IK; going down looks like gliding. There are already two clips for this in the player model (in `poses_climb.py`): `stairs_up` (loop 0.7 s: high knee lift, slight forward lean) and `stairs_down` (loop 0.7 s: careful steps, lean back, knees bend on landing).

**After this job:** when the player walks or runs on the ground and the height changes steeply (stairs, steep slopes), the legs blend toward `stairs_up` or `stairs_down` (lower body only), stronger the steeper it is, with the stride tempo following the speed. Flat ground and gentle slopes look exactly as today. Animation only: **no movement or physics change.**

## Output rules (important)
- Output **complete files only for**: `humanoid/src/incline.rs`, `humanoid/src/incline_tests.rs`, `data/incline.json`.
- **Do not output** the other files. In `=== NOTES ===`, give **every** edit for `animator.rs` and `lib.rs`: file, exact existing lines to find (copied exactly, unique in that file), and the complete replacement. A helper applies them literally. `animator.rs` is 863 lines: put the logic in `incline.rs`, add only a few lines there.
- The game wiring edit is fixed below; repeat it in NOTES unchanged.
- **Keep the whole answer under 30,000 characters.** `incline.rs` under 250 lines, tests under 180 lines. Do not stop until the final line `END OF EDITS`.
- No new crates. No sounds. No allocation per frame. No new clips (use the two that exist).

## `data/incline.json`
```json
{
  "version": 1,
  "minGrade": 0.25,
  "fullGrade": 0.6,
  "minSpeed": 0.5,
  "gradeSmoothing": 8.0,
  "weightRate": 6.0,
  "maxWeight": 0.85,
  "teleportDistance": 1.5,
  "tempoRefSpeed": 3.0
}
```
Grade = height change / horizontal distance (0.25 is about 14 degrees; ordinary stairs are about 0.6-0.7).

## `humanoid/src/incline.rs`
```rust
pub struct InclineConfig { /* serde camelCase */ }
impl InclineConfig { pub fn from_json(text: &str) -> Result<Self, serde_json::Error>; pub fn load_shared() -> &'static InclineConfig; }
pub struct InclineState { /* driven, last position, smoothed grade, up/down weights */ }
impl InclineState {
    pub fn new() -> Self;
    /// The character's world position and whether it is on the ground this frame.
    pub fn set_input(&mut self, position: Vec3, on_floor: bool);
    pub fn tick(&mut self, dt: f32);
    pub fn grade(&self) -> f32; // smoothed, + up, - down
    pub fn up_weight(&self) -> f32; // 0..maxWeight
    pub fn down_weight(&self) -> f32; // 0..maxWeight
    pub fn tempo(&self, speed: f32) -> f32; // clip playback speed
}
```
Rules:
- Grade comes from the change of position between ticks (height over horizontal distance), only while on the floor and moving faster than `minSpeed`; it is smoothed exponentially with `gradeSmoothing` (frame-rate independent) so single bumps and stair-step jitter do not flicker. Off the floor or standing still, weights fade to 0 at `weightRate`.
- Weight ramps from 0 at `minGrade` to `maxWeight` at `fullGrade`. Only one of up/down is non-zero at a time.
- A position jump larger than `teleportDistance` resets without a spike. Finite at dt 0 and 0.1.
- Not driven = everything 0 (the job 1 golden test must still pass).

## How the animator uses it (you write these edits)
- Field `incline: InclineState`; `pub fn set_incline(&mut self, position: Vec3, on_floor: bool)`.
- Blend `stairs_up` / `stairs_down` on the **legs only** (Lower mask), with tempo from `tempo()`, on a layer that is free in this situation, and only when not on a ladder, not swimming, not wading, not in the air and no full-body clip is active. Read `animator.rs` to see which layer the others use (`Air` is used by air, swim, wade and ladder; `Crouch`, `Aim` and `Additive` by posture). Explain your choice in one line. If no layer is truly free, give the minimal edit to add one `LayerId` in `layers.rs` as a NOTES edit and say why.
- Crouch walking and sprinting win over the stairs blend (do not apply while `posture` drives crouch walk or sprint).
- Foot IK keeps running on top (it runs after the layers), so the feet still land on each step.

## Game wiring (fixed; repeat it in NOTES exactly)
`rotland/src/app/render_frame.rs`: find `            pv.animator.set_ladder(pose.climbing, pose.on_rungs, pose.position.y);` and replace with:
```rust
            pv.animator.set_ladder(pose.climbing, pose.on_rungs, pose.position.y);
            pv.animator.set_incline(pose.position, pose.on_floor && !pose.swimming && !pose.climbing);
```

## Tests: `humanoid/src/incline_tests.rs`
Compact: data parses; `stairs_up` and `stairs_down` exist in `poses_climb.py` (read as text); not driven = zero; flat walking stays zero; walking up a 0.6 grade reaches maxWeight on up only; walking down gives down only; a single 0.2 m bump on flat ground stays below 0.1; standing still and leaving the floor fade to 0; teleport no spike; same result at 30 and 120 fps within a small tolerance; no NaN at dt 0 and 0.1.

Every new number in Rust code needs a source tag on its line (`// data: incline.json key` or `// new: DECISIONS 2026-11-24`).

## Worker notes

