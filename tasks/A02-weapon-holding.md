# A02 (Neon): the player really holds every weapon

Files: humanoid/src/stance.rs, humanoid/src/stance_tests.rs, data/stances.json, tools/blender/characters/poses_stance.py, tasks/A02-weapon-holding.md, humanoid/src/animator.rs, humanoid/src/lib.rs, rotland/src/player_visual.rs, rotland/src/app/frame_actors.rs, tools/blender/characters/poses.py, assets/models/characters/human_player.glb
Read: humanoid/src/animator.rs, humanoid/src/weapon.rs, humanoid/src/layers.rs, humanoid/src/pose_buffer.rs, tools/blender/characters/poses.py, tools/blender/characters/poses_shared.py, tools/blender/characters/poses_combat.py
API: humanoid/src/ik.rs, humanoid/src/mask.rs, humanoid/src/anim_config.rs
Data: data/weapons.json, data/animation.json

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 2 of 20 of a deep character animation system. Job 1 added layers, masks, full-body clips and events.

**The problem to solve:** guns and tools look like they float. The gun is attached to a hand that keeps doing the empty-handed walk swing. `pistol_aim` and `rifle_aim` exist, but nothing calls them, and the left hand does not hold two-handed weapons except during a swing.

**After this job:** every weapon has a believable carried pose while standing, walking and running, the legs keep walking, two-handed weapons have both hands on them, and holding right mouse raises guns and the bow to an aim pose. Firing, recoil, reload, and looking up/down are job 3. Do not do them here.

## Output rules (important, your last answer in this project was cut off)
- Output **complete files only for these 4 new files**: `humanoid/src/stance.rs`, `humanoid/src/stance_tests.rs`, `data/stances.json`, `tools/blender/characters/poses_stance.py`.
- **Do not output** the existing files (`animator.rs`, `lib.rs`, `player_visual.rs`, `frame_actors.rs`, `poses.py`). Instead, in `=== NOTES ===`, give exact edits for them: for each edit, the file, the exact existing lines to find, and the exact replacement. A helper applies them literally, so the "find" text must be copied exactly from the file shown to you and be unique in that file. For `frame_actors.rs`, which you cannot see, use the one edit given below.
- **Keep the whole answer under 50,000 characters.** Make tests compact with shared helpers. Always finish with `=== NOTES ===`.
- No new crates. No sounds. No new gameplay. Files around 300 lines. Weak laptop: no allocation per frame.

## Weapon classes
`pistol` (pistol), `rifle` (rifle, ak47, smg), `shotgun`, `melee_1h` (knife, dagger, machete, pipe), `melee_2h` (bat, axe, fire_axe, hammer, crowbar, katana), `spear`, `bow`, `rod` (fishing_rod), and none (empty hands).

Weapon ids in `set_weapon` are **model names**, which are the same as the ids above. An unknown id must fall back safely: no stance, keep old behaviour.

## `data/stances.json`
```json
{
  "version": 1,
  "readyBlendRate": 10.0,
  "aimBlendRate": 14.0,
  "ikBlendRate": 14.0,
  "classes": {
    "rifle": {
      "readyClip": "stance_rifle_ready",
      "aimClip": "rifle_aim",
      "leftHandGrip": true,
      "gripWhenAiming": true,
      "readyHold": { "pos": [0, 0, 0], "rot": [0, 0, 0] },
      "aimHold": { "pos": [0, 0, 0], "rot": [0, 0, 0] }
    }
  },
  "weapons": { "rifle": "rifle" },
  "overrides": { "shotgun": { "readyHold": { "pos": [0, 0, 0], "rot": [0, 0, 0] } } }
}
```
Fill in all classes and all weapons. `overrides` may replace `readyHold`, `aimHold` or `offHandGrip` for one weapon. Weapon holds are starting values: Claude will tune them from screenshots by editing this JSON without rebuilding, so everything that affects where a weapon sits must be in this file. Melee weapons and the rod have no `aimClip`. Keep the current look of how the bat sits as a starting point: its existing hold comes from `weapon_hold_for`.

## `humanoid/src/stance.rs`
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StanceClass { Pistol, Rifle, Shotgun, Melee1H, Melee2H, Spear, Bow, Rod }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoldTransform { pub pos: Vec3, pub rot_deg: Vec3 }
#[derive(Clone, Debug, PartialEq)]
pub struct StanceDef {
    pub class: StanceClass,
    pub ready_clip: String,
    pub aim_clip: Option<String>,
    pub left_hand_grip: bool,
    pub grip_when_aiming: bool,
    pub ready_hold: HoldTransform,
    pub aim_hold: HoldTransform,
    pub off_hand_grip: Option<Vec3>,
}
pub struct StanceTable { /* parsed data */ }
impl StanceTable {
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error>;
    /// Cached with OnceLock from paths::repo_path("data/stances.json"). If missing or broken, log::warn once and use an empty table.
    pub fn load_shared() -> &'static StanceTable;
    pub fn for_weapon(&self, weapon: &str) -> Option<&StanceDef>;
    pub fn ready_rate(&self) -> f32;
    pub fn aim_rate(&self) -> f32;
    pub fn ik_rate(&self) -> f32;
}
pub struct StanceController { /* current def, weights */ }
impl StanceController {
    pub fn new() -> Self;
    pub fn set_weapon(&mut self, def: Option<&StanceDef>);
    pub fn set_aiming(&mut self, aiming: bool);
    pub fn is_aiming(&self) -> bool;
    pub fn tick(&mut self, dt: f32, rates: (f32, f32, f32), action_active: bool);
    pub fn ready_clip(&self) -> Option<&str>;
    pub fn aim_clip(&self) -> Option<&str>;
    pub fn ready_weight(&self) -> f32;
    pub fn aim_weight(&self) -> f32;
    pub fn left_ik_weight(&self) -> f32;
    pub fn hold(&self) -> HoldTransform; // blended ready to aim by aim_weight
    pub fn off_hand_grip(&self) -> Option<Vec3>;
}
```
Rules:
- `off_hand_grip`: use the JSON value, otherwise `crate::weapon::off_hand_grip_for(id)`.
- Aiming only has an effect for classes with an `aim_clip`.
- When the weapon changes, the new stance fades in and the aim resets, with no snap and no leftover clip from the old weapon.
- While a melee action plays, `left_ik_weight` must follow the old behaviour for the two-handed swing (the existing IK stage already handles `melee_2h`), so the swing must look exactly as it does now.
- Weights stay in 0..=1, are frame-rate independent, and are safe at dt 0.

## New clips: `tools/blender/characters/poses_stance.py`
Same keyframe format as `poses_combat.py`; export `CLIPS`. Upper body only (no leg or hip keys). Each loops over 2.0 s with a very small breathing motion on chest and arms so that it isn't frozen. Arm Z is relative to `ARM_DOWN_OFFSET` as in `poses_shared.py`.
- `stance_pistol_ready`: low ready, gun pointing down and forward, right elbow slightly bent, left arm relaxed near it.
- `stance_rifle_ready`: rifle across the body, muzzle low to the left, right hand at the hip or lower chest, left hand forward supporting the barrel.
- `stance_shotgun_ready`: like the rifle, slightly lower and heavier.
- `stance_melee1h_ready`: weapon low at the side, elbow bent, relaxed and ready.
- `stance_melee2h_ready`: both hands on the handle, weapon resting up toward the right shoulder.
- `stance_spear_ready`: both hands, spear held diagonal, point forward and up.
- `stance_bow_ready`: bow held low in front in the hand, other arm relaxed.
- `stance_rod_ready`: rod held forward at about 45 degrees up, with the right hand near the waist and the left hand supporting.

## Exact edits you must describe in the notes
1. `poses.py`: import these clips and merge them into `PLAYER_ONLY_CLIPS` (NPCs get them in a later job).
2. `lib.rs`: add `pub mod stance;`, `#[cfg(test)] mod stance_tests;` and the exports.
3. `animator.rs`:
   - Add a `stance: StanceController` field.
   - `set_weapon` also calls the controller with `StanceTable::load_shared().for_weapon(name)`.
   - Add `pub fn set_aiming(&mut self, aiming: bool)`, and `pub fn weapon_hold(&self) -> Option<HoldTransform>` (None when no stance, so old behaviour is used).
   - In `update`, tick the controller and drive `LayerId::Stance` and `LayerId::Aim` clip and weight from it. Use `MaskId::Upper`. `set_aim_pose` must keep working for callers that use it directly: the stance only drives the Aim layer while it has a stance with an aim clip.
   - In the IK stage, run the existing `apply_two_hand_left_grip_ik` also when `left_ik_weight > 0`, using the stance hold and off-hand grip. To fade it, apply the IK to a scratch pose and blend it with `blend_pose_masked` using `MaskId::LeftArm`, without allocating a new pose per frame. Full weight during `melee_2h` as today.
4. `player_visual.rs` `pose()`: when `animator.weapon_hold()` is `Some`, call `weapon_world_matrix(char_world, &self.rig, &bones[base..], self.hand_joint, hold.pos, hold.rot_deg)`; otherwise keep `weapon_world_matrix_named`. Mind the borrow of `self.animator`: read the hold before the closure.
5. `frame_actors.rs`: find this exact line:
   `        ui.right_held = aiming;`
   and add after it:
   `        if let Some(pv) = actors.player_visual.as_mut() { pv.animator.set_aiming(aiming); }`

## Tests: `humanoid/src/stance_tests.rs`
Compact, using a hand-made skeleton. Cover: the real `data/stances.json` parses; every weapon in `data/weapons.json` plus `fishing_rod` has a stance; every `readyClip` name is defined in `poses_stance.py` (read the file as text); unknown weapon is safe; aim is ignored for melee; weights fade and stay in range; frame-rate independence (one 0.1 s step vs ten 0.01 s steps within a small tolerance); weapon swap resets aim; `hold()` blends ready to aim; overrides replace only their field; no NaN at dt 0.

Every new number in Rust code needs a source tag on its line (`// data: stances.json key` or `// new: DECISIONS 2026-11-23`).

## Worker notes
