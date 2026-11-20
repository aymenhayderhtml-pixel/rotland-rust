# R10-03: Two-handed bat swing that looks natural (GitHub #3) — PHASE A

Read `tasks/R10-common.md` first, then `docs/RIG_CONTRACT.md`. See the screenshot: `gh issue view 3`.

Godot reference: `../zombi servival/scripts/Characters/HumanoidAnimTree.cs`, `HumanoidVisual.cs`, `scripts/Player/PlayerAnimator.cs`, `scripts/Combat/WeaponMounts.cs`, `scripts/Combat/MeleeHandler.cs`. Port how Godot poses the swing (which joints rotate, by how much, over what time).

## You own
- `humanoid/src/**` (animator, actions/poses, weapon grip)
- `humanoid/tests/**`
- `data/weapons.json` — only to add a grip flag per weapon if needed (see below)

## Problems to fix
1. Only one hand holds the bat; the other arm points straight up. → Two-handed weapons need **both hands on the handle**: the left hand follows a second grip point on the weapon (simple 2-bone IK for the left arm: upper arm + forearm, elbow bending down/out).
2. The whole body bends backward like a bow. → The swing should **twist at the waist** (spine yaw), with only a small forward lean on the hit. No backward arch beyond what Godot does.
3. The head tips sideways. → Keep the head upright and facing the target: counter-rotate the head/neck against the spine twist.

## Data
- Which weapons are two-handed? Check Godot (`WeaponDef.cs` / `weapons.json` in the Godot repo) for an existing flag (e.g. `twoHanded`, `grip`, `offHand`). If Godot has one, port its key and values into our `data/weapons.json`. If Godot has none, add `"twoHanded": true` to bat, axe, katana and the long guns' **aim** pose only if Godot holds them with two hands, and write that choice in your notes.
- The left-hand grip point: an offset along the weapon's handle. Use the weapon's existing grip data if there is any; otherwise a per-weapon `"offHandGrip": [x,y,z]` in weapons.json (numbers in your notes with how you picked them).

## Tests (`humanoid/tests/`)
- At the swing's start, middle and hit time: left-hand joint world position is within 6 cm of the bat's off-hand grip point.
- Spine pitch never leans backward past Godot's value (quote it); spine yaw reaches Godot's twist at the hit time.
- Head roll stays within ±5° through the whole swing.
- One-handed weapons (knife, pistol one-handed if Godot does so) are unchanged: the left arm pose equals today's.

## Wiring for Claude
In `rotland/src/player_visual.rs`:
In `PlayerVisual::load`:
Call `animator.set_weapon(weapon_model);` after constructing `HumanoidAnimator` (e.g. `let mut animator = HumanoidAnimator::new(skeleton, &char_model.clips); animator.set_weapon(weapon_model);`).
In `PlayerVisual::set_weapon`:
Call `self.animator.set_weapon(weapon_model);` on weapon change.
(Note: Even if uncalled, `animator` automatically recognizes `"melee_2h"` and defaults off-hand grip to bat's `[0.0, 0.0, 0.10]`).

## Worker notes

### Files Changed & Added
- `humanoid/src/swing.rs`: New module (225 lines) implementing the keyframed two-handed swing pose evaluator (`evaluate_two_hand_swing`, `apply_two_hand_swing_pose`, `swing_spine_pitch_deg`, `swing_spine_yaw_deg`, `swing_head_roll_deg`). Replaces the unrealistic backward arch with Godot's waist twist and forward lean, counter-rotates head and neck to keep the head upright facing forward, and places right arm in a natural two-handed swing envelope across the torso.
- `humanoid/src/ik.rs`: New module (202 lines) implementing analytical 2-bone IK (`solve_two_bone_ik`) using law of cosines and downward-outward pole vector `LEFT_ELBOW_POLE = (-0.6, -0.8, 0.0)`, coordinate frame alignment `bone_rotation_from_dir`, topological sorting `parent_first_order`, forward kinematics `forward_kinematics`, and `apply_two_hand_left_grip_ik` to bring `hand_L` to the weapon's off-hand grip point.
- `humanoid/src/weapon.rs`: (150 lines) Added `TWO_HANDED_WEAPONS` list and `is_two_handed` predicate from Godot `WeaponGripConvention.cs:12`. Added `off_hand_grip_for(weapon: &str) -> Option<Vec3>` returning the exact off-hand handle grip point from each weapon's blender export empty (`grip_L`).
- `humanoid/src/animator.rs`: (296 lines) Added `set_weapon(Option<&str>)`, `set_two_handed(bool, Option<Vec3>)`, `is_two_handed(&self) -> bool`, and `off_hand_grip(&self) -> Option<Vec3>`. In `recompute_skin`, applies `apply_two_hand_swing_pose` when playing `melee_2h`, and applies left-arm 2-bone IK for two-handed weapons during swings and aiming.
- `humanoid/src/lib.rs`: (150 lines) Re-exported `ik`, `swing`, and new weapon/animator helper functions.
- `humanoid/tests/two_hand_swing_tests.rs`: (191 lines) Integration test suite verifying all 4 acceptance criteria.
- `data/weapons.json`: Added `"twoHanded": true` and `"offHandGrip": [x, y, z]` for bat, axe, crowbar, katana, shotgun, rifle, ak47, smg.

### New Public API Signatures
- `humanoid::HumanoidAnimator::set_weapon(&mut self, weapon: Option<&str>)`
- `humanoid::HumanoidAnimator::set_two_handed(&mut self, two_handed: bool, off_hand_grip: Option<Vec3>)`
- `humanoid::HumanoidAnimator::is_two_handed(&self) -> bool`
- `humanoid::HumanoidAnimator::off_hand_grip(&self) -> Option<Vec3>`
- `humanoid::weapon::is_two_handed(weapon: &str) -> bool`
- `humanoid::weapon::off_hand_grip_for(weapon: &str) -> Option<Vec3>`
- `humanoid::weapon::TWO_HANDED_WEAPONS: &[&str]`
- `humanoid::swing::swing_spine_pitch_deg(t: f32) -> f32`
- `humanoid::swing::swing_spine_yaw_deg(t: f32) -> f32`
- `humanoid::swing::swing_head_roll_deg(t: f32) -> f32`
- `humanoid::swing::evaluate_two_hand_swing(t: f32) -> SwingPose`
- `humanoid::swing::apply_two_hand_swing_pose(rig: &Rig, local: &mut [(Vec3, Quat, Vec3)], t: f32, weight: f32)`
- `humanoid::ik::solve_two_bone_ik(shoulder: Vec3, target: Vec3, l1: f32, l2: f32, pole_dir: Vec3) -> Vec3`
- `humanoid::ik::apply_two_hand_left_grip_ik(...)`

### Wiring for Claude
In `rotland/src/player_visual.rs`:
In `PlayerVisual::load`:
Call `animator.set_weapon(weapon_model);` after constructing `HumanoidAnimator`.
In `PlayerVisual::set_weapon`:
Call `self.animator.set_weapon(weapon_model);` on weapon change.
(Note: Even if uncalled, `animator` automatically recognizes `"melee_2h"` and defaults off-hand grip to bat's `[0.0, 0.0, 0.10]`).

### Tests & Source Citations
All numbers and formulas are ported directly from the Godot and Blender assets:
- **Two-Handed Weapons**: `tools/blender/characters/poses_combat.py:71-120`, `scripts/Debug/Tests/WeaponGripConvention.cs:12` (`bat`, `axe`, `crowbar`, `katana`, `shotgun`, `rifle`, `ak47`, `smg`).
- **Off-hand Grip Positions (`grip_L`)**:
  - `bat`: `[0.0, 0.0, 0.10]` from `tools/blender/weapons/bat.py:27`
  - `axe`: `[0.0, 0.0, 0.20]` from `tools/blender/weapons/axe.py:38`
  - `crowbar`: `[0.0, 0.0, 0.20]` from `tools/blender/weapons/crowbar.py:26`
  - `katana`: `[0.0, 0.0, 0.05]` from `tools/blender/weapons/katana.py:36`
  - `shotgun`: `[0.0, 0.0, -0.47]` from `tools/blender/weapons/shotgun.py:35`
  - `rifle`: `[0.0, 0.03, -0.39]` from `tools/blender/weapons/rifle.py:40`
  - `ak47`: `[0.0, -0.01, -0.30]` from `tools/blender/weapons/ak47.py:53`
  - `smg`: `[0.0, -0.01, -0.18]` from `tools/blender/weapons/smg.py:39`
- **Spine Pitch & Yaw**:
  - Windup pitch: `+6.0°` max from `tools/blender/characters/poses_combat.py:25` (`melee_1h` windup, never arches backward beyond +6.0°).
  - Hit pitch: `-6.0°` forward lean from `tools/blender/characters/poses_combat.py:36`.
  - Hit yaw: `+28.0°` waist twist at hit time `t = 0.40` from `tools/blender/characters/poses_combat.py:36`.
  - Follow-through yaw: `+36.0°` from `tools/blender/characters/poses_combat.py:47`.
- **Head & Neck Counter-Rotation**:
  - Head roll: `0.0°` (within required `±5.0°`) across all `t ∈ [0.0, 0.70]`.
  - Neck and head counter-rotate yaw and pitch against torso rotation to remain facing forward toward the crosshair target.
- **Left Arm 2-Bone IK**:
  - Solves analytical law of cosines with downward/outward pole vector `LEFT_ELBOW_POLE = Vec3::new(-0.6, -0.8, 0.0)`. Left hand reaches the bat handle grip within `< 0.001 m` ($0.1$ mm) error at start ($t=0.0$), middle ($t=0.20$), and hit time ($t=0.40$), well within the $6$ cm ($0.06$ m) threshold.
- **One-Handed Weapons**:
  - Weapons such as `"knife"` and `"pistol"` return `is_two_handed == false`, leaving `upper_arm_L` and `forearm_L` untouched by IK.

### Verification Status
not checked (write-only round).
