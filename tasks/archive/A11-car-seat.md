# A11 (Neon): the driver sits in the car, steers, gets out and is thrown out

Files: humanoid/src/car_anim.rs, humanoid/src/car_anim_tests.rs, data/car_anim.json, tasks/A11-car-seat.md, humanoid/src/animator.rs, humanoid/src/lib.rs, rotland/src/player_visual.rs, rotland/src/app/render_frame.rs
Read: humanoid/src/animator.rs, humanoid/src/ladder_anim.rs, rotland/src/player_visual.rs, rotland/src/app/render_frame.rs
API: humanoid/src/layers.rs, humanoid/src/mask.rs, humanoid/src/full_body.rs, game/src/vehicles/car.rs, game/src/vehicles/body.rs

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 11 of 20 of a deep character animation system. Done so far: layers, stances, aiming, directional legs, foot IK, jump/land, crouch/sprint/tired, swimming, ladders (`ladder_anim.rs`: a good example of a state that owns the `Air` layer while active), stairs. Copy the structure of `ladder_anim.rs` (config with `from_json` + `load_shared`, a small state with a `driven` flag, helpers that write into the layers so `animator.rs` gets only a few lines).

**The problem:** driving works (E near a car: `Cars::try_enter`; E again when slow or flipped: `Cars::exit` puts the player at the left exit spot; the car's own door already swings). But while driving, **the player's body is not drawn at all** (`render_frame.rs` skips it when `driving`), so the car looks empty, and getting out pops the player beside the car in a normal standing pose.

**After this job:**
- While driving (third-person car camera), the player is **drawn sitting in the driver's seat** playing `car_sit_drive` (hands on the wheel), moving and tilting with the car body (full car rotation, not only yaw). **No weapon** is drawn while seated.
- The hands **turn the wheel**: `car_steer_left` / `car_steer_right` (upper body) blend in by the car's steer angle (`car.body.steer_angle`, max `maxSteerRad` 0.5 in data/vehicles.json; positive = left, check `body.rs` and say which sign you used).
- **Getting in:** the seated pose fades in over `enterFade` (no standing get-in clip: it would poke through the roof).
- **Getting out:** at the exit spot the player plays `car_get_out` once (it starts seated and stands up).
- **Thrown out:** if the car was flipped when the player got out (`car.body.is_flipped()`), play `car_thrown_out` instead.
- In first person nothing is drawn while driving (as today). Animation only: **no gameplay, camera or car change.**

These clips already exist in the player model (made earlier): `car_sit_drive` (loop 2 s, hips loc Y -0.45, hands on a wheel ~0.4 m in front of the chest), `car_steer_left`, `car_steer_right` (loop 1 s, upper body, wheel turned ~60 degrees), `car_get_out` (1 s), `car_thrown_out` (1.2 s, ends lying), `car_get_in`, `car_open_door`, `car_sit_passenger` (not used now).

## Output rules (important)
- Output **complete files only for**: `humanoid/src/car_anim.rs`, `humanoid/src/car_anim_tests.rs`, `data/car_anim.json`.
- **Do not output** the other files. In `=== NOTES ===`, give **every** edit for `animator.rs`, `lib.rs`, `player_visual.rs` and `render_frame.rs`: file, exact existing lines to find (copied exactly, unique in that file), and the complete replacement. A helper applies them literally. Keep `animator.rs` edits to a few lines.
- **Keep the whole answer under 35,000 characters.** `car_anim.rs` under 250 lines, tests under 180 lines. Do not stop until the final line `END OF EDITS`.
- No new crates. No sounds. No allocation per frame (do **not** call `Car::draw_state()` for this, it allocates; build the matrix from `car.body.rotation` and `car.body.position`).

## `data/car_anim.json`
```json
{
  "version": 1,
  "seatOffset": [-0.36, 0.05, -0.12],
  "seatYawDeg": 0.0,
  "enterFade": 0.3,
  "steerFullRad": 0.45,
  "steerRate": 8.0,
  "steerMaxWeight": 1.0,
  "exitFade": 0.15,
  "thrownFade": 0.1
}
```
`seatOffset` is where the character's feet origin sits in car-local space (x right, y up, z forward is -Z) so that the seated hips land on the seat; the car's `seat_local()` is (-0.36, 0.72, -0.12) but that is the eye/ride point, so the Y here is a starting guess that Claude will tune from a screenshot. The car model faces -Z like the character.

## `humanoid/src/car_anim.rs`
```rust
pub struct CarAnimConfig { /* serde camelCase */ }
impl CarAnimConfig { pub fn from_json(text: &str) -> Result<Self, serde_json::Error>; pub fn load_shared() -> &'static CarAnimConfig; }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarEvent { GotOut, ThrownOut }
pub struct CarAnimState { /* driven, seated, steer weight (smoothed, signed), last flipped flag */ }
impl CarAnimState {
    pub fn new() -> Self;
    /// seated = the player is driving; steer = car steer angle (rad); flipped = car upside down/on its side.
    pub fn set_input(&mut self, seated: bool, steer: f32, flipped: bool);
    /// Returns an event once, on the frame the player leaves the seat.
    pub fn tick(&mut self, dt: f32) -> Option<CarEvent>;
    pub fn is_seated(&self) -> bool;
    pub fn steer_weight(&self) -> f32; // -1..1, smoothed with steerRate
}
pub fn car_event_clip(event: CarEvent) -> &'static str;
/// Character world matrix for the seat from the car body's world matrix.
pub fn seat_world(car_world: glam::Mat4, cfg: &CarAnimConfig) -> glam::Mat4;
```
Not driven = nothing happens (the job 1 golden test must still pass). Finite at dt 0 and 0.1.

## How the animator and the game use it (you write these edits)
- Animator: field `car: CarAnimState`; `pub fn set_car(&mut self, seated: bool, steer: f32, flipped: bool)`. While seated: `car_sit_drive` owns the `Air` layer (like the ladder; air/swim/wade/ladder/incline logic skipped, no landing fires afterwards), steer clips blend on the upper body by `|steer_weight|` (pick a free layer; stance/aim fade to 0; posture ignored), foot IK inactive. On an event, play its clip through `play_full_body` with the fades from data, never while dying or in a held pose.
- `player_visual.rs`: add `pub fn pose_world(&mut self, dt, world: Mat4, locomotion, bones, draw_weapon: bool)` that is the same as `pose()` but takes the full character world matrix (refactor `pose()` to call it, so there is no copy of the body). With `draw_weapon` false return no weapon batch.
- `render_frame.rs`: today the player is drawn only when not `driving`. Change it so that when driving **and not first person**, the animator gets `set_car(true, steer, flipped)` and the body is drawn with `pose_world(seat_world(car_world, cfg), draw_weapon = false)`; when not driving, call `set_car(false, 0.0, last_flipped)` every frame (keep the last flipped value seen while seated, e.g. in the animator state, so the exit event knows) and draw as today. Find the driven car with `actors.cars.as_ref().and_then(|c| c.driven_car())`. In first person while driving, still draw nothing.

## Tests: `humanoid/src/car_anim_tests.rs`
Compact: data parses; the clip names used exist in `tools/blender/characters/poses_car.py` (read as text, search for `"car_sit_drive"` etc.); not driven = nothing; seated on/off fires GotOut exactly once; flipped exit fires ThrownOut; steer weight follows steer with smoothing, signed, clamped; `seat_world` with identity car = translation by seatOffset, and with a rotated car rotates the offset; no NaN at dt 0 and 0.1.

Every new number in Rust code needs a source tag on its line (`// data: car_anim.json key` or `// new: DECISIONS 2026-11-24`).

## Worker notes

- Applied all Neon edits (animator.rs 8 edits, player_visual.rs pose_world, render_frame.rs driver branch) and added `pub mod car_anim;` + `#[cfg(test)] mod car_anim_tests;` to lib.rs next to ladder_anim (the notes gave no find text for lib.rs).
- Gate: clippy PASS, tests PASS (979). Scope check fails only on the 15 owner files (not touched).
- Screenshot check (release, `--animals=drive`; `--near=drive` is not a real flag): the car is driven, but the rear chase camera hides the seated driver inside the cabin at seatOffset Y 0.05. Diagnostic with Y 1.5 showed the driver drawn and following the car, so the path works. Seat height is NOT verified; needs a side or front camera view to tune.
- `car_anim.rs` is 360 lines (spec says under 250); not split yet.
- Steering, get-out and thrown-out not visually checked.

## Neon notes

### Sign convention & missing files

* **Steer sign:** Positive `car.body.steer_angle` blends `car_steer_left` (positive `steer_weight`); negative blends `car_steer_right` (negative `steer_weight`). Note: only the API signatures for `game/src/vehicles/body.rs` were provided (not its implementation), so this follows the Godot +Y right-hand yaw convention and task specification ("positive = left").
* **Borrowed steering layer:** While seated, `CarAnimState::apply_car` uses `LayerId::Air` (`MaskId::Full`) for `car_sit_drive` and temporarily borrows `LayerId::Crouch` (`MaskId::Upper`, `LayerBlend::Override`) for `car_steer_left` / `car_steer_right`, saving its original clip/mask/blend/speed/looping once on enter and restoring them in `release_layers`.
* **Missing from Read:** `humanoid/src/lib.rs` was not included under `## Read`. Add `pub mod car_anim;` and `#[cfg(test)] mod car_anim_tests;` next to the existing `ladder_anim` declarations (and optionally re-export `CarAnimConfig`, `CarAnimState`, `CarEvent`, `car_event_clip`, `seat_world` if `ladder_anim` does).

---

### Edits for `humanoid/src/lib.rs`

File not provided in `## Read`; next to `pub mod ladder_anim;` add:

```rust
pub mod car_anim;
#[cfg(test)]
mod car_anim_tests;

```

---

### Edits for `humanoid/src/animator.rs`

#### Edit 1 — Import `car_anim`

Find:

```rust
use crate::ladder_anim::{
    is_ladder_clip, ladder_event_clip, LadderAnimState, LadderEvent, LADDER_CLIMB_CLIP,
};

```

Replace:

```rust
use crate::car_anim::{car_event_clip, CarAnimState, CarEvent};
use crate::ladder_anim::{
    is_ladder_clip, ladder_event_clip, LadderAnimState, LadderEvent, LADDER_CLIMB_CLIP,
};

```

#### Edit 2 — Add `car` field to `HumanoidAnimator`

Find:

```rust
    swim: SwimState,
    ladder: LadderAnimState,
    incline: InclineState,

```

Replace:

```rust
    swim: SwimState,
    ladder: LadderAnimState,
    car: CarAnimState,
    incline: InclineState,

```

#### Edit 3 — Initialize `car` in `with_config`

Find:

```rust
            swim: SwimState::new(),
            ladder: LadderAnimState::new(),
            incline: InclineState::new(),

```

Replace:

```rust
            swim: SwimState::new(),
            ladder: LadderAnimState::new(),
            car: CarAnimState::new(),
            incline: InclineState::new(),

```

#### Edit 4 — Drive car state before ladder/swim/air/incline in `update`

Find:

```rust
        let ladder_event = self.ladder.tick(step);
        self.incline.tick(step);
        let on_ladder = self.ladder.is_on_ladder();
        if on_ladder {

```

Replace:

```rust
        let car_event = self.car.tick(step);
        let in_car = self.car.is_seated();
        let (on_ladder, ladder_event) = if in_car {
            (false, None)
        } else {
            self.car.release_layers(&mut self.layers);
            if let Some(event) = car_event {
                self.start_car_event(event);
            }
            let ev = self.ladder.tick(step);
            self.incline.tick(step);
            (self.ladder.is_on_ladder(), ev)
        };
        if in_car {
            self.ladder.release_air(&mut self.layers);
            self.swim.release_air(&mut self.layers);
            self.incline.release_air(&mut self.layers);
            self.air = AirState::new();
            self.car.apply_car(&mut self.layers);
        } else if on_ladder {

```

#### Edit 5 — Skip crouch target, stance, posture, and foot IK while seated in `update`

Find:

```rust
        self.layers.get_mut(LayerId::Crouch).target =
            if !on_ladder && locomotion.crouching { 1.0 } else { 0.0 };
        self.motion.tick(step, locomotion.on_floor && !on_ladder);
        let tempo = tempo_for(self.blend_pos) * self.tempo_scale * self.motion.tempo_scale();
        self.layers.get_mut(LayerId::Locomotion).speed = tempo;
        self.layers.advance(step);
        self.time_idle += step;

        self.aim.tick(step);
        self.advance_action(step);
        if !on_ladder {
            self.advance_stance(step);
            self.advance_posture(step);
        }
        self.advance_full_body(step);
        if self.full_body.is_some()
            || self.posture.crouch_walk_weight() > LAYER_EPSILON
            || self.posture.sprint_weight() > LAYER_EPSILON
        {
            self.incline.release_air(&mut self.layers);
        }
        self.foot_ik.set_active(
            locomotion.on_floor
                && !on_ladder
                && !self.swim.is_swimming()
                && self.full_body.is_none(),
        );

```

Replace:

```rust
        if !in_car {
            self.layers.get_mut(LayerId::Crouch).target =
                if !on_ladder && locomotion.crouching { 1.0 } else { 0.0 };
        }
        self.motion.tick(step, locomotion.on_floor && !on_ladder && !in_car);
        let tempo = tempo_for(self.blend_pos) * self.tempo_scale * self.motion.tempo_scale();
        self.layers.get_mut(LayerId::Locomotion).speed = tempo;
        self.layers.advance(step);
        self.time_idle += step;

        self.aim.tick(step);
        self.advance_action(step);
        if !on_ladder && !in_car {
            self.advance_stance(step);
            self.advance_posture(step);
        }
        self.advance_full_body(step);
        if self.full_body.is_some()
            || self.posture.crouch_walk_weight() > LAYER_EPSILON
            || self.posture.sprint_weight() > LAYER_EPSILON
        {
            self.incline.release_air(&mut self.layers);
        }
        self.foot_ik.set_active(
            locomotion.on_floor
                && !in_car
                && !on_ladder
                && !self.swim.is_swimming()
                && self.full_body.is_none(),
        );

```

#### Edit 6 — Add `set_car`

Find:

```rust
    /// Feeds world position and grounded flag for stairs and steep-slope leg blends.
    pub fn set_incline(&mut self, position: Vec3, on_floor: bool) {
        self.incline.set_input(position, on_floor);
        self.dirty = true;
    }

```

Replace:

```rust
    /// Feeds world position and grounded flag for stairs and steep-slope leg blends.
    pub fn set_incline(&mut self, position: Vec3, on_floor: bool) {
        self.incline.set_input(position, on_floor);
        self.dirty = true;
    }

    /// Feeds car driving state: seated flag, steer angle (rad) and flipped flag.
    pub fn set_car(&mut self, seated: bool, steer: f32, flipped: bool) {
        self.car.set_input(seated, steer, flipped);
        self.dirty = true;
    }

```

#### Edit 7 — Add `start_car_event`

Find:

```rust
        self.play_full_body(clip_name, opts);
    }

    /// Smoothly stops the active full-body clip using `fade_out` (or the clip's configured default).

```

Replace:

```rust
        self.play_full_body(clip_name, opts);
    }

    /// Plays the car exit or thrown-out clip once unless dying or in a held pose.
    fn start_car_event(&mut self, event: CarEvent) {
        if self.dying || self.held_pose.is_some() {
            return;
        }
        let fade = self.car.event_fade(event);
        let opts = FullBodyOptions {
            speed: 1.0, // new: DECISIONS 2026-11-24
            looping: false,
            hold_last: false,
            fade_in: fade,
            fade_out: fade,
        };
        self.play_full_body(car_event_clip(event), opts);
    }

    /// Smoothly stops the active full-body clip using `fade_out` (or the clip's configured default).

```

#### Edit 8 — Skip aim pitch, lean, and left-hand IK while seated in `recompute_skin`

Find:

```rust
        // 7b. Aim pitch and recoil (before left-hand IK so off-hand follows the gun)
        if !self.ladder.is_on_ladder() {
            self.aim.apply(&self.aim_joints, self.pool.get_mut(0), self.stance.aim_weight());
            self.motion.apply_lean(&self.motion_joints, self.pool.get_mut(0));
        }
        // 7c. Foot IK (before left-hand IK so off-hand follows when hips drop)
        if let Some(fj) = self.foot_joints {
            self.foot_ik.apply(
                &fj,
                &self.parents,
                &self.order,
                self.pool.get_mut(0),
                &mut self.foot_globals,
            );
        }
        // 8. Left-hand IK
        if !self.ladder.is_on_ladder() {
            self.apply_ik_stage(rig);
        }

```

Replace:

```rust
        // 7b. Aim pitch and recoil (before left-hand IK so off-hand follows the gun)
        if !self.ladder.is_on_ladder() && !self.car.is_seated() {
            self.aim.apply(&self.aim_joints, self.pool.get_mut(0), self.stance.aim_weight());
            self.motion.apply_lean(&self.motion_joints, self.pool.get_mut(0));
        }
        // 7c. Foot IK (before left-hand IK so off-hand follows when hips drop)
        if let Some(fj) = self.foot_joints {
            self.foot_ik.apply(
                &fj,
                &self.parents,
                &self.order,
                self.pool.get_mut(0),
                &mut self.foot_globals,
            );
        }
        // 8. Left-hand IK
        if !self.ladder.is_on_ladder() && !self.car.is_seated() {
            self.apply_ik_stage(rig);
        }

```

---

### Edits for `rotland/src/player_visual.rs`

#### Edit 1 — Add `pose_world` and delegate `pose` to it

Find:

```rust
    /// Poses the character, writes skin matrices to `bones`, and returns the body and weapon batches.
    /// The weapon uses static instance bone_base 0 (identity matrix).
    pub fn pose(
        &mut self,
        dt: f32,
        pos: Vec3,
        yaw: f32,
        locomotion: Locomotion,
        bones: &mut Vec<Mat4>,
    ) -> (SkinnedBatch<'_>, Option<SkinnedBatch<'_>>) {
        self.animator.update(dt, locomotion);

        let base = bones.len();
        let joints = self.rig.joint_count();
        bones.resize(base + joints, Mat4::IDENTITY);
        self.animator.write_skin(&self.rig, &mut bones[base..]);
        let weapon_hold = self.animator.weapon_hold();

        let char_world = Mat4::from_translation(pos) * Mat4::from_rotation_y(yaw);
        let body_batch = SkinnedBatch {
            mesh: &self.body_mesh,
            instances: vec![SkinInstance::new(char_world, base as u32)],
        };

        let weapon_batch = self.weapon_mesh.as_ref().map(|wmesh| {
            let name = self.weapon_name.as_deref().unwrap_or("");
            let weapon_world = if let Some(hold) = weapon_hold {
                weapon_world_matrix(
                    char_world,
                    &self.rig,
                    &bones[base..],
                    self.hand_joint,
                    hold.pos,
                    hold.rot_deg,
                )
            } else {
                weapon_world_matrix_named(
                    char_world,
                    &self.rig,
                    &bones[base..],
                    self.hand_joint,
                    name,
                )
            };
            SkinnedBatch {
                mesh: wmesh,
                instances: vec![SkinInstance::new(weapon_world, 0)],
            }
        });

        (body_batch, weapon_batch)
    }

```

Replace:

```rust
    /// Poses the character, writes skin matrices to `bones`, and returns the body and weapon batches.
    /// The weapon uses static instance bone_base 0 (identity matrix).
    pub fn pose(
        &mut self,
        dt: f32,
        pos: Vec3,
        yaw: f32,
        locomotion: Locomotion,
        bones: &mut Vec<Mat4>,
    ) -> (SkinnedBatch<'_>, Option<SkinnedBatch<'_>>) {
        let char_world = Mat4::from_translation(pos) * Mat4::from_rotation_y(yaw);
        self.pose_world(dt, char_world, locomotion, bones, true)
    }

    /// Poses the character with an explicit world matrix; skips the weapon batch when `draw_weapon` is false.
    pub fn pose_world(
        &mut self,
        dt: f32,
        world: Mat4,
        locomotion: Locomotion,
        bones: &mut Vec<Mat4>,
        draw_weapon: bool,
    ) -> (SkinnedBatch<'_>, Option<SkinnedBatch<'_>>) {
        self.animator.update(dt, locomotion);

        let base = bones.len();
        let joints = self.rig.joint_count();
        bones.resize(base + joints, Mat4::IDENTITY);
        self.animator.write_skin(&self.rig, &mut bones[base..]);
        let weapon_hold = self.animator.weapon_hold();

        let body_batch = SkinnedBatch {
            mesh: &self.body_mesh,
            instances: vec![SkinInstance::new(world, base as u32)],
        };

        let weapon_batch = if draw_weapon {
            self.weapon_mesh.as_ref().map(|wmesh| {
                let name = self.weapon_name.as_deref().unwrap_or("");
                let weapon_world = if let Some(hold) = weapon_hold {
                    weapon_world_matrix(
                        world,
                        &self.rig,
                        &bones[base..],
                        self.hand_joint,
                        hold.pos,
                        hold.rot_deg,
                    )
                } else {
                    weapon_world_matrix_named(
                        world,
                        &self.rig,
                        &bones[base..],
                        self.hand_joint,
                        name,
                    )
                };
                SkinnedBatch {
                    mesh: wmesh,
                    instances: vec![SkinInstance::new(weapon_world, 0)],
                }
            })
        } else {
            None
        };

        (body_batch, weapon_batch)
    }

```

---

### Edits for `rotland/src/app/render_frame.rs`

#### Edit 1 — Draw seated driver in third person while driving; pass `set_car(false, ...)` when on foot

Find:

```rust
    let driving = actors.cars.as_ref().is_some_and(|c| c.driven_car_id.is_some());
    if let (Some(pv), Some(pose), false) = (actors.player_visual.as_mut(), player_pose, driving) {

```

Replace:

```rust
    let driven_car = actors.cars.as_ref().and_then(|c| c.driven_car()).map(|car| {
        let world = Mat4::from_rotation_translation(car.body.rotation, car.body.position);
        (world, car.body.steer_angle, car.body.is_flipped())
    });
    if let Some((car_world, steer, flipped)) = driven_car {
        if let (Some(pv), Some(pose)) = (actors.player_visual.as_mut(), player_pose)
            && !pose.first_person
        {
            pv.animator.set_car(true, steer, flipped);
            let cfg = humanoid::car_anim::CarAnimConfig::load_shared();
            let seat = humanoid::car_anim::seat_world(car_world, cfg);
            let loco = humanoid::Locomotion {
                speed: 0.0, // new: DECISIONS 2026-11-24
                on_floor: true,
                crouching: false,
            };
            let (body, _weapon) =
                pv.pose_world(session.last_dt, seat, loco, &mut render.bones, false);
            main.push(engine::render::SkinnedBatch {
                mesh: body.mesh,
                instances: body.instances.clone(),
            });
            shadow.push(body);
        }
    } else if let (Some(pv), Some(pose)) = (actors.player_visual.as_mut(), player_pose) {
        pv.animator.set_car(false, 0.0, false); // new: DECISIONS 2026-11-24

```

## Gate report
- date: 2026-11-24
- base: main
- files changed (23): assets/models/buildings/settlement.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/car_anim.json, humanoid/src/animator.rs, humanoid/src/car_anim.rs, humanoid/src/car_anim_tests.rs, humanoid/src/lib.rs, rotland/src/app/render_frame.rs, rotland/src/player_visual.rs, tasks/A11-car-seat.md, tools/blender/buildings/settlement.py, tools/blender/vehicles/car_parts.py
- tests passed: 979
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
