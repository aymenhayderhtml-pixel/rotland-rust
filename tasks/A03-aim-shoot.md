# A03 (Neon): aiming, shooting, recoil and reloading on the body

Files: humanoid/src/aim.rs, humanoid/src/aim_tests.rs, data/aim.json, data/stances.json, tools/blender/characters/poses_aim.py, tasks/A03-aim-shoot.md, humanoid/src/animator.rs, humanoid/src/lib.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_world.rs, tools/blender/characters/poses.py, assets/models/characters/human_player.glb
Read: humanoid/src/animator.rs, humanoid/src/stance.rs, humanoid/src/pose_buffer.rs, tools/blender/characters/poses.py, tools/blender/characters/poses_shared.py, tools/blender/characters/poses_stance.py
API: humanoid/src/ik.rs, humanoid/src/mask.rs, humanoid/src/layers.rs, humanoid/src/weapon.rs
Data: data/stances.json, data/weapons.json

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 3 of 20 of a deep character animation system. Job 1 added layers, masks and events. Job 2 gave every weapon a carry stance and a right-mouse aim pose.

**Problems seen in the game now:**
1. When aiming a rifle, the right arm is almost straight, so the stock floats in front of the shoulder and the left hand cannot reach the front of the gun. It stops by the receiver.
2. The body does not follow the camera: looking up or down while aiming leaves the gun level.
3. Firing has no body reaction, and reloading shows nothing.

**After this job:** guns are shouldered with the left hand on the fore-end, the upper body aims where the camera looks, every shot kicks the body by weapon, and reloads play for exactly the real reload time.

## Output rules (important)
- Output **complete files only for**: `humanoid/src/aim.rs`, `humanoid/src/aim_tests.rs`, `data/aim.json`, `data/stances.json`, `tools/blender/characters/poses_aim.py`.
- **Do not output** `animator.rs`, `lib.rs`, `frame_actors.rs`, `frame_world.rs` or `poses.py`. In `=== NOTES ===`, give exact edits: file, exact existing lines to find (copied exactly, unique in that file), and exact replacement. A helper applies them literally.
- **Keep the whole answer under 50,000 characters.** Compact tests with shared helpers. Always finish with `=== NOTES ===`.
- No new crates. No sounds. No muzzle flash work. No change to shooting rules, damage or timing. Files around 300 lines. No allocation per frame.

## New clips: `tools/blender/characters/poses_aim.py`
Same keyframe format as the other pose files; export `CLIPS`. Upper body only.
- `aim_rifle_shoulder` (loop, 2 s, tiny breathing): torso bladed slightly (chest turned so the left shoulder leads), right upper arm forward and out with the **elbow clearly bent** so the right hand is close to the right side of the chest and the stock sits at the shoulder; left arm reaching forward under the gun. Head slightly down toward the sights.
- `aim_shotgun_shoulder`: same family, more forward lean.
- `aim_pistol_two_hand`: both arms forward in a triangle, slight elbow bend, hands together.
- `aim_bow_draw`: left arm straight forward holding the bow, right hand drawn to the cheek. **The bow is held in the right hand in this game**, so mirror this as needed: right arm extended forward with the bow, left hand drawn back to the cheek.
- `reload_pistol` (1.0 s, not looping): gun tilts in, left hand down to the belt, back up, slaps the grip.
- `reload_rifle` (1.0 s): left hand pulls the magazine down, goes to the belt, inserts, then pulls the bolt.
- `reload_shotgun` (1.0 s): gun tilted up, left hand repeats a shell-loading motion 3 times.
Keep the hand angle so that the weapon stays level when the forearm is bent: add a `hand_R` key that cancels the forearm's bend. Today a level barrel comes from `aimHold.rot = [0,0,0]` with a level forearm.

## `data/stances.json`
Same file with: rifle class `aimClip` = `aim_rifle_shoulder`; shotgun = `aim_shotgun_shoulder`; pistol = `aim_pistol_two_hand`; bow = `aim_bow_draw`. Keep all current values otherwise, especially these tuned from screenshots: pistol readyHold rot `[-35,0,0]`, pistol aimHold rot `[0,0,0]`, rifle readyHold rot `[-30,0,0]`, shotgun readyHold rot `[-35,0,0]` and its override. Add `reloadClip` to pistol (`reload_pistol`), rifle (`reload_rifle`) and shotgun (`reload_shotgun`) only if you read it through `aim.json` instead of `StanceDef`. **Prefer putting reload clips in `aim.json`**, so `stance.rs` needs no change.

## `data/aim.json`
```json
{
  "version": 1,
  "maxPitchUpDeg": 55, "maxPitchDownDeg": 50,
  "pitchBones": { "spine": 0.2, "chest": 0.35, "neck": 0.2, "head": 0.25 },
  "carryPitchFraction": 0.25,
  "pitchRate": 18.0,
  "classes": {
    "pistol": { "kickDeg": 7, "kickBack": 0.018, "stiffness": 170, "damping": 20, "reloadClip": "reload_pistol" },
    "rifle":  { "kickDeg": 4, "kickBack": 0.025, "stiffness": 190, "damping": 22, "reloadClip": "reload_rifle" },
    "shotgun":{ "kickDeg": 13, "kickBack": 0.05, "stiffness": 120, "damping": 15, "reloadClip": "reload_shotgun" },
    "bow":    { "kickDeg": 1.5, "kickBack": 0.005, "stiffness": 150, "damping": 20 }
  },
  "maxKickDeg": 22
}
```

## `humanoid/src/aim.rs`
```rust
pub struct AimConfig { /* serde camelCase */ }
impl AimConfig { pub fn from_json(text: &str) -> Result<Self, serde_json::Error>; pub fn load_shared() -> &'static AimConfig; }
pub struct AimController { /* look pitch, recoil spring, reload state */ }
impl AimController {
    pub fn new() -> Self;
    pub fn set_class(&mut self, class: Option<StanceClass>); // weapon changed: stops recoil and reload
    pub fn set_look_pitch(&mut self, radians: f32); // camera pitch, positive looks up
    pub fn fire(&mut self);
    /// Returns the clip to play and the speed that makes it last exactly `seconds`.
    pub fn start_reload(&mut self, seconds: f32, clip_duration: impl Fn(&str) -> Option<f32>) -> Option<(&str, f32)>;
    pub fn cancel_reload(&mut self);
    pub fn is_reloading(&self) -> bool;
    pub fn tick(&mut self, dt: f32);
    /// Applies pitch and recoil to the local pose. `aim_weight` 0..1.
    pub fn apply(&self, joints: &AimJoints, local: &mut [LocalTransform], aim_weight: f32);
    pub fn kick_deg(&self) -> f32;
    pub fn smoothed_pitch(&self) -> f32;
}
pub struct AimJoints { /* indices of spine, chest, neck, head, upper_arm_R; Option each */ }
impl AimJoints { pub fn from_names(names: &[&str]) -> Self; }
```
Rules:
- Pitch is applied around each bone's local X in model space terms, spread by `pitchBones`, clamped to the limits. While aiming use the full pitch; while carrying use `carryPitchFraction`. Blend by `aim_weight`. Check the sign: positive pitch must bend the torso and gun **upward**. The model faces -Z; a negative rotation about X on these bones currently tilts the chest backward in the clips (see `"chest": (-4, ...)` in the aim poses). State the sign you used in the notes.
- Recoil is a damped spring with stable integration (never explodes at dt 0.1, use sub-steps or a closed form). Each shot adds `kickDeg`, limited to `maxKickDeg`. It tilts chest and right arm up and moves the chest back by `kickBack`, using a small amount of spine too. It settles to zero.
- Reload lasts the real `reloadTime` given. It plays as a normal upper-body action through `play_action(clip, speed)`, and left-hand IK is off during it so the left hand can move, then fades back.
- When firing or changing weapon during a reload, the game's reload rules decide. The animation just calls `cancel_reload` when the weapon changes.

## Exact edits to give in the notes
1. `poses.py`: merge the clips into `PLAYER_ONLY_CLIPS`.
2. `lib.rs`: `pub mod aim;`, `#[cfg(test)] mod aim_tests;`, and the exports.
3. `animator.rs`: add an `AimController` and `AimJoints`. Update it in `update` (must also run with LOD throttle). Apply it after the aim and action layers and **before** the left-hand IK, so the IK follows the moved gun. Add `pub fn set_look_pitch(&mut self, radians: f32)`, `pub fn on_gun_fired(&mut self)`, `pub fn on_reload_started(&mut self, seconds: f32)`. `set_weapon` updates the class. During a reload `left_ik_weight` must be treated as 0. The golden test from job 1 must still pass (with no weapon, pitch and kick have no effect).
4. `frame_actors.rs`, find exactly:
   `        if let Some(pv) = actors.player_visual.as_mut() { pv.animator.set_aiming(aiming); }`
   replace with:
   `        if let Some(pv) = actors.player_visual.as_mut() { pv.animator.set_aiming(aiming); pv.animator.set_look_pitch(rig.pitch); }`
5. `frame_world.rs`, find exactly:
   `            c.shots += ev.shots_fired;`
   and add after it:
   ```rust
            if let Some(pv) = actors.player_visual.as_mut() {
                for _ in 0..ev.shots_fired { pv.animator.on_gun_fired(); }
                if ev.reload_started
                    && let Some(gun) = play_state.gun.as_ref()
                {
                    pv.animator.on_reload_started(gun.reload_time);
                }
            }
   ```
   If the field on `WeaponDef` has another name, the helper will fix it.

## Tests: `humanoid/src/aim_tests.rs`
Compact with a hand-made skeleton: `data/aim.json` parses; every gun class in it has a clip defined in `poses_aim.py` (read as text); stance aim clips exist in `poses_aim.py`; pitch clamps; pitch up and down move the chest in opposite directions; carry uses a fraction; recoil rises then returns to zero; many shots stay within `maxKickDeg`; stable at dt 0.1; frame-rate independent within tolerance; reload speed makes the clip last the given seconds; weapon change cancels recoil and reload; no class means no effect; no NaN at dt 0.

Every new number in Rust code needs a source tag on its line (`// data: aim.json key` or `// new: DECISIONS 2026-11-23`).

## Worker notes
