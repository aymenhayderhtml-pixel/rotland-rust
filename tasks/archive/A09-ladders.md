# A09 (Neon): climbing ladders with hands and feet on the rungs

Files: humanoid/src/ladder_anim.rs, humanoid/src/ladder_anim_tests.rs, data/ladder_anim.json, tools/blender/characters/poses_ladder.py, tasks/A09-ladders.md, humanoid/src/animator.rs, humanoid/src/lib.rs, tools/blender/characters/poses.py, tools/blender/characters/poses_climb.py, assets/models/characters/human_player.glb, rotland/src/app/state.rs, rotland/src/app/render_frame.rs
Read: humanoid/src/animator.rs, humanoid/src/swim.rs, humanoid/src/layers.rs, tools/blender/characters/poses_air.py
API: humanoid/src/airborne.rs, humanoid/src/full_body.rs, humanoid/src/mask.rs, game/src/ladders/climb.rs, game/src/ladders/rail.rs

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 9 of 20 of a deep character animation system. Done so far: layers, weapon stances, aiming, directional legs, foot IK, jump/fall/land (`airborne.rs`), crouch/sprint/tired (`posture.rs`), swimming (`swim.rs`). Copy the structure of `swim.rs` (config with `from_json` + `load_shared`, a small state with a `driven` flag, and helpers that write into the layers so `animator.rs` only gets a few lines).

**The problem:** ladder climbing already works in the game (`game/src/ladders`: E at the foot or the deck; W/S move up/down at 2.5 m/s; at the top a 0.3 s path steps onto the deck; starting from the deck walks a 0.3 s path onto the ladder). But the body plays the falling clip (the player is not on the floor), so the character flails in the air next to the ladder.

**After this job:**
- On the rungs, `ladder_climb` plays **driven by the distance climbed**: hands and feet move rung by rung, stop when the player stops (holding the current pose), and play backwards when climbing down.
- Getting on at the bottom (`ladder_enter_bottom`), stepping off at the top (`ladder_exit_top`), getting on from the top (`ladder_enter_top`), stepping off at the bottom (`ladder_exit_bottom`).
- No falling clip, no landing clip and no weapon stance/aim pose while on the ladder.
- Animation only: **do not change climbing, speeds or controls.**

## Output rules (important)
- Output **complete files only for**: `humanoid/src/ladder_anim.rs`, `humanoid/src/ladder_anim_tests.rs`, `data/ladder_anim.json`, `tools/blender/characters/poses_ladder.py`.
- **Do not output** the other files. In `=== NOTES ===`, give **every** edit for `animator.rs`, `lib.rs` and `poses.py`: file, exact existing lines to find (copied exactly, unique in that file), and the complete replacement. A helper applies them literally. `animator.rs` is long, so keep logic in `ladder_anim.rs` and add only a few lines there. If you need a small new method in `airborne.rs` (for example `reset()`), give it as an edit too.
- The game wiring edits are fixed below; repeat them in NOTES unchanged.
- **Keep the whole answer under 40,000 characters.** `ladder_anim.rs` under 300 lines, tests under 200 lines, clips compact. Do not stop until the final line `END OF EDITS`.
- No new crates. No sounds. No allocation per frame.

## New clips: `tools/blender/characters/poses_ladder.py`
Same keyframe format as `poses_air.py`; export `CLIPS`. These **replace** five weak clips with the same names in `poses_climb.py` (the helper deletes the old ones), so use exactly these names.

**Sign rules of this rig (important, a previous answer got them all backwards):**
- hips `rot` X **negative** = lean the body forward; hips `loc` Y negative = lower the body.
- spine/chest/neck/head X negative = bend forward / look down; positive = bend back / look up.
- `upper_arm_*` X positive = arm swings forward and up (about 90 = pointing straight forward, 160 = nearly straight up); the arms-down offset is added by the builder, so 0 = arm hanging down.
- `forearm_*` X positive = elbow bends (forearm comes forward/up).
- `thigh_*` X positive = leg lifts forward; `shin_*` X **negative** = knee bends; `foot_*` X positive = toes point down.
- Look at `poses_air.py` as a working example of these signs.

The character stands at the ladder foot point facing the ladder; rungs are about 0.35 m in front of the chest and 0.3 m apart.
- `ladder_climb` (loop, 0.8 s): **one cycle = the body rises 0.6 m (two rungs)**. Hands high: the reaching hand goes up to about head height + 0.3 m (upper arm X about 150-165, elbow slightly bent), the other hand at chest height (upper arm X about 80-100, elbow bent ~70). Feet clearly lift: the stepping leg's thigh X about 55-65 with shin X about -80 to -90, the other leg nearly straight. Right hand + left foot move together, then left hand + right foot. Body leans slightly toward the ladder (hips X about -5). First and last frames identical.
- `ladder_enter_bottom` (no loop, 0.5 s): from standing, both hands reach up to the rungs, right foot steps up onto the first rung; ends exactly in the first frame pose of `ladder_climb`.
- `ladder_exit_top` (no loop, 0.9 s): starts in the `ladder_climb` first frame, pushes up with the arms, one knee onto the deck, steps forward, ends standing.
- `ladder_enter_top` (no loop, 0.9 s): from standing on the deck, crouch, hands down to the rails, step down; ends in the `ladder_climb` first frame.
- `ladder_exit_bottom` (no loop, 0.4 s): from the `ladder_climb` first frame, step back off onto the ground, hands let go, stand.

## `data/ladder_anim.json`
```json
{
  "version": 1,
  "metresPerCycle": 0.6,
  "teleportDistance": 1.5,
  "enterBottomSeconds": 0.5,
  "exitTopSeconds": 0.45,
  "enterTopSeconds": 0.45,
  "exitBottomSeconds": 0.4,
  "fade": 0.12
}
```
The `*Seconds` values are how long each enter/exit clip should take in game (the clip playback speed = clip length / seconds). The game's own step path at the top takes 0.3 s.

## `humanoid/src/ladder_anim.rs`
```rust
pub struct LadderAnimConfig { /* serde camelCase */ }
impl LadderAnimConfig { pub fn from_json(text: &str) -> Result<Self, serde_json::Error>; pub fn load_shared() -> &'static LadderAnimConfig; }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LadderEvent { EnterBottom, ExitTop, EnterTop, ExitBottom }
pub struct LadderAnimState { /* driven, active, on_rungs, last y, climbed distance, last vertical direction */ }
impl LadderAnimState {
    pub fn new() -> Self;
    /// active = the game's climb is active; on_rungs = hands on the rungs (not the top step path); y = the player's height.
    pub fn set_input(&mut self, active: bool, on_rungs: bool, y: f32);
    /// Returns at most one event per tick.
    pub fn tick(&mut self, dt: f32) -> Option<LadderEvent>;
    pub fn is_on_ladder(&self) -> bool; // true while active (including the top step path)
    /// Clip time for ladder_climb from the climbed distance (wraps; goes backwards when climbing down).
    pub fn climb_time(&self, clip_length: f32) -> f32;
}
pub fn ladder_event_clip(event: LadderEvent) -> &'static str;
```
Rules:
- Not driven = nothing happens (the job 1 golden test must still pass).
- Events: active turns on with on_rungs true → `EnterBottom`; active turns on with on_rungs false → `EnterTop`; while active, on_rungs turns false after climbing up → `ExitTop`; active turns off and the last vertical direction was down → `ExitBottom`. Never two events in one tick.
- A y jump larger than `teleportDistance` is ignored (no wild clip jump). Finite at dt 0 and 0.1.

## How the animator uses it (you write these edits)
- Field `ladder: LadderAnimState`; `pub fn set_ladder(&mut self, active: bool, on_rungs: bool, y: f32)`.
- While `is_on_ladder()`: `ladder_climb` owns the `Air` layer at target/weight 1, and its time is **set from `climb_time()` every frame** (the layer must not advance it on its own, for example speed 0 or set after the advance). The air state is not ticked and is reset, so no fall clip shows and **no landing fires** when the climb ends. Swim and wade are skipped. Stance and Aim layers fade to 0 (weapon arms must not pull the hands off the rungs); posture is ignored. Foot IK is inactive.
- Events play their clip through `play_full_body` with the speed from the `*Seconds` values and `fade`, never while dying or in a held pose; an event may replace another ladder clip that is still fading.
- When the climb ends, everything goes back to the normal logic smoothly.

## Game wiring (fixed; repeat these in NOTES exactly)
1. `rotland/src/app/state.rs`: find `    pub swimming: bool,` and replace with:
```rust
    pub swimming: bool,
    pub climbing: bool,
    pub on_rungs: bool,
```
   and find `            swimming: p.in_deep_water,` and replace with:
```rust
            swimming: p.in_deep_water,
            climbing: self.climb.is_active(),
            on_rungs: self.climb.is_on_rungs(),
```
2. `rotland/src/app/render_frame.rs`: find `            pv.animator.set_water(pose.water_depth, pose.swimming, pose.horizontal_speed, pose.stamina_frac);` and replace with:
```rust
            pv.animator.set_water(pose.water_depth, pose.swimming, pose.horizontal_speed, pose.stamina_frac);
            pv.animator.set_ladder(pose.climbing, pose.on_rungs, pose.position.y);
```
3. `tools/blender/characters/poses_climb.py`: the helper deletes the five old entries `ladder_climb`, `ladder_enter_bottom`, `ladder_exit_top`, `ladder_enter_top`, `ladder_exit_bottom` (keep the stairs/vault/step clips). In `poses.py`, import `poses_ladder` as `_LADDER` and add `**_LADDER` to `PLAYER_ONLY_CLIPS`.

## Tests: `humanoid/src/ladder_anim_tests.rs`
Compact: data parses; all 5 clips are defined in `poses_ladder.py` (read as text); never-driven does nothing; each of the 4 events fires exactly once in the right situation and never on land; climb_time advances with distance up, goes back when climbing down, holds when still, wraps, and stays finite; teleport ignored; no NaN at dt 0 and 0.1.

Every new number in Rust code needs a source tag on its line (`// data: ladder_anim.json key` or `// new: DECISIONS 2026-11-24`).

## Worker notes

## Neon notes

No edits to `humanoid/src/airborne.rs` are needed: `self.air = AirState::new();` resets `AirState` without heap allocation (its `AirConfig` is a stack copy from `OnceLock`).

Note on `humanoid/src/lib.rs` and `tools/blender/characters/poses.py`: neither file was included under `Read`, so the find blocks below follow the A08 (`swim`) and prompt conventions; verify surrounding lines if they differ.

### 1. `humanoid/src/animator.rs`

Edit 1a — find:

```rust
use crate::stance::{HoldTransform, StanceController, StanceTable};
use crate::swim::{SwimState, SWIM_CLIMB_OUT_CLIP};

```

replace with:

```rust
use crate::ladder_anim::{
    is_ladder_clip, ladder_event_clip, LadderAnimState, LadderEvent, LADDER_CLIMB_CLIP,
};
use crate::stance::{HoldTransform, StanceController, StanceTable};
use crate::swim::{SwimState, SWIM_CLIMB_OUT_CLIP};

```

Edit 1b — find:

```rust
    posture: PostureState,
    swim: SwimState,
    foot_ik: FootIkState,

```

replace with:

```rust
    posture: PostureState,
    swim: SwimState,
    ladder: LadderAnimState,
    foot_ik: FootIkState,

```

Edit 1c — find:

```rust
            posture: PostureState::new(),
            swim: SwimState::new(),
            foot_ik: FootIkState::new(),

```

replace with:

```rust
            posture: PostureState::new(),
            swim: SwimState::new(),
            ladder: LadderAnimState::new(),
            foot_ik: FootIkState::new(),

```

Edit 1d — find:

```rust
        let climb_out = self.swim.tick(step);
        if self.swim.is_swimming() {
            self.swim.apply_swim(&mut self.layers);
        } else {
            let (air_target, air_clip, landing) = if self.air_driven {
                let landing = self.air.tick(step, locomotion.on_floor);
                (self.air.air_target(), self.air.air_clip().unwrap_or("jump"), landing)
            } else {
                (if locomotion.on_floor { 0.0 } else { 1.0 }, "jump", None)
            };
            if locomotion.on_floor && air_target <= LAYER_EPSILON && self.swim.wants_wade() {
                self.swim.apply_wade(&mut self.layers);
            } else {
                self.swim.release_air(&mut self.layers);
                let layer = self.layers.get_mut(LayerId::Air);
                layer.target = air_target;
                if self.air_driven && layer.clip.as_deref() != Some(air_clip) {
                    layer.clip = Some(air_clip.to_owned());
                    layer.time = 0.0;
                }
            }
            if let Some(kind) = landing {
                self.start_landing(kind);
            }
        }
        if climb_out {
            self.start_climb_out();
        }
        self.layers.get_mut(LayerId::Crouch).target = if locomotion.crouching { 1.0 } else { 0.0 };
        self.motion.tick(step, locomotion.on_floor);
        let tempo = tempo_for(self.blend_pos) * self.tempo_scale * self.motion.tempo_scale();
        self.layers.get_mut(LayerId::Locomotion).speed = tempo;
        self.layers.advance(step);
        self.time_idle += step;

        self.aim.tick(step);
        self.advance_action(step);
        self.advance_stance(step);
        self.advance_posture(step);
        self.advance_full_body(step);
        self.foot_ik
            .set_active(locomotion.on_floor && !self.swim.is_swimming() && self.full_body.is_none());
        self.foot_ik.tick(step, self.blend_pos);

```

replace with:

```rust
        let ladder_event = self.ladder.tick(step);
        let on_ladder = self.ladder.is_on_ladder();
        if on_ladder {
            self.swim.release_air(&mut self.layers);
            self.air = AirState::new();
            let clip_len = self.clips.get(LADDER_CLIMB_CLIP).map_or(0.0, |c| c.duration);
            self.ladder.apply_ladder(&mut self.layers, clip_len);
            self.ladder.suppress_upper(&mut self.layers);
        } else {
            self.ladder.release_air(&mut self.layers);
            let climb_out = self.swim.tick(step);
            if self.swim.is_swimming() {
                self.swim.apply_swim(&mut self.layers);
            } else {
                let (air_target, air_clip, landing) = if self.air_driven {
                    let landing = self.air.tick(step, locomotion.on_floor);
                    (self.air.air_target(), self.air.air_clip().unwrap_or("jump"), landing)
                } else {
                    (if locomotion.on_floor { 0.0 } else { 1.0 }, "jump", None)
                };
                if locomotion.on_floor && air_target <= LAYER_EPSILON && self.swim.wants_wade() {
                    self.swim.apply_wade(&mut self.layers);
                } else {
                    self.swim.release_air(&mut self.layers);
                    let layer = self.layers.get_mut(LayerId::Air);
                    layer.target = air_target;
                    if self.air_driven && layer.clip.as_deref() != Some(air_clip) {
                        layer.clip = Some(air_clip.to_owned());
                        layer.time = 0.0;
                    }
                }
                if let Some(kind) = landing {
                    self.start_landing(kind);
                }
            }
            if climb_out {
                self.start_climb_out();
            }
        }
        if let Some(event) = ladder_event {
            self.start_ladder_event(event);
        }
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
        self.foot_ik.set_active(
            locomotion.on_floor
                && !on_ladder
                && !self.swim.is_swimming()
                && self.full_body.is_none(),
        );
        self.foot_ik.tick(step, self.blend_pos);

```

Edit 1e — find:

```rust
    /// Feeds lake depth, deep-water flag, horizontal speed and stamina.
    pub fn set_water(&mut self, depth: f32, swimming: bool, speed: f32, stamina_frac: f32) {
        self.swim.set_input(depth, swimming, speed, stamina_frac);
        self.dirty = true;
    }

```

replace with:

```rust
    /// Feeds lake depth, deep-water flag, horizontal speed and stamina.
    pub fn set_water(&mut self, depth: f32, swimming: bool, speed: f32, stamina_frac: f32) {
        self.swim.set_input(depth, swimming, speed, stamina_frac);
        self.dirty = true;
    }

    /// Feeds ladder climb activity, on-rungs flag and player height.
    pub fn set_ladder(&mut self, active: bool, on_rungs: bool, y: f32) {
        self.ladder.set_input(active, on_rungs, y);
        self.dirty = true;
    }

```

Edit 1f — find:

```rust
    /// Plays the climb-out clip once on leaving deep water unless dying, held or busy.
    fn start_climb_out(&mut self) {
        if self.dying || self.held_pose.is_some() || self.full_body.is_some() {
            return;
        }
        let speed = self
            .clips
            .get(SWIM_CLIMB_OUT_CLIP)
            .map_or(1.0, |c| self.swim.climb_out_speed(c.duration));
        let (fade_in, fade_out) = self.swim.climb_out_fades();
        let opts = FullBodyOptions { speed, looping: false, hold_last: false, fade_in, fade_out };
        self.play_full_body(SWIM_CLIMB_OUT_CLIP, opts);
    }

```

replace with:

```rust
    /// Plays the climb-out clip once on leaving deep water unless dying, held or busy.
    fn start_climb_out(&mut self) {
        if self.dying || self.held_pose.is_some() || self.full_body.is_some() {
            return;
        }
        let speed = self
            .clips
            .get(SWIM_CLIMB_OUT_CLIP)
            .map_or(1.0, |c| self.swim.climb_out_speed(c.duration));
        let (fade_in, fade_out) = self.swim.climb_out_fades();
        let opts = FullBodyOptions { speed, looping: false, hold_last: false, fade_in, fade_out };
        self.play_full_body(SWIM_CLIMB_OUT_CLIP, opts);
    }

    /// Plays a ladder enter/exit clip unless dying, held or blocked by a non-ladder override.
    fn start_ladder_event(&mut self, event: LadderEvent) {
        if self.dying || self.held_pose.is_some() {
            return;
        }
        if let Some(fb) = &self.full_body
            && !is_ladder_clip(fb.clip_name())
        {
            return;
        }
        let clip_name = ladder_event_clip(event);
        let Some(duration) = self.clips.get(clip_name).map(|c| c.duration) else {
            return;
        };
        let speed = self.ladder.event_speed(event, duration);
        let fade = self.ladder.fade();
        let opts = FullBodyOptions {
            speed,
            looping: false,
            hold_last: false,
            fade_in: fade,
            fade_out: fade,
        };
        self.play_full_body(clip_name, opts);
    }

```

Edit 1g — find:

```rust
        // 7b. Aim pitch and recoil (before left-hand IK so off-hand follows the gun)
        self.aim.apply(&self.aim_joints, self.pool.get_mut(0), self.stance.aim_weight());
        self.motion.apply_lean(&self.motion_joints, self.pool.get_mut(0));
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
        self.apply_ik_stage(rig);

```

replace with:

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

### 2. `humanoid/src/lib.rs` (not in Read — verify anchor lines)

Edit 2a — find:

```rust
pub mod swim;

```

replace with:

```rust
pub mod ladder_anim;
pub mod swim;

```

Edit 2b — find:

```rust
#[cfg(test)]
mod swim_tests;

```

replace with:

```rust
#[cfg(test)]
mod ladder_anim_tests;
#[cfg(test)]
mod swim_tests;

```

### 3. `tools/blender/characters/poses.py` and `poses_climb.py` (not in Read — verify anchor lines)

* `tools/blender/characters/poses_climb.py`: delete the five old entries `ladder_climb`, `ladder_enter_bottom`, `ladder_exit_top`, `ladder_enter_top`, `ladder_exit_bottom` (keep the stairs/vault/step clips).
* `tools/blender/characters/poses.py`: import `poses_ladder` as `_LADDER` and add `**_LADDER` to `PLAYER_ONLY_CLIPS`:

Edit 3a — find:

```python
from poses_swim import CLIPS as _SWIM

```

replace with:

```python
from poses_ladder import CLIPS as _LADDER
from poses_swim import CLIPS as _SWIM

```

Edit 3b — find:

```python
    **_SWIM,

```

replace with:

```python
    **_LADDER,
    **_SWIM,

```

### 4. Game wiring (fixed)

1. `rotland/src/app/state.rs`: find `    pub swimming: bool,` and replace with:

```rust
    pub swimming: bool,
    pub climbing: bool,
    pub on_rungs: bool,

```

and find `            swimming: p.in_deep_water,` and replace with:

```rust
            swimming: p.in_deep_water,
            climbing: self.climb.is_active(),
            on_rungs: self.climb.is_on_rungs(),

```

2. `rotland/src/app/render_frame.rs`: find `            pv.animator.set_water(pose.water_depth, pose.swimming, pose.horizontal_speed, pose.stamina_frac);` and replace with:

```rust
            pv.animator.set_water(pose.water_depth, pose.swimming, pose.horizontal_speed, pose.stamina_frac);
            pv.animator.set_ladder(pose.climbing, pose.on_rungs, pose.position.y);

```

3. Rebuild `assets/models/characters/human_player.glb` with the Blender character builder after applying the `poses.py` / `poses_climb.py` / `poses_ladder.py` updates, then run `cargo clippy --all-targets -- -D warnings` and `cargo test -p humanoid`.

## Gate report
- date: 2026-11-24
- base: main
- files changed (27): assets/models/buildings/settlement.glb, assets/models/characters/human_player.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/ladder_anim.json, humanoid/src/animator.rs, humanoid/src/ladder_anim.rs, humanoid/src/ladder_anim_tests.rs, humanoid/src/lib.rs, rotland/src/app/render_frame.rs, rotland/src/app/state.rs, tasks/A09-ladders.md, tools/blender/buildings/settlement.py, tools/blender/characters/poses_climb.py, tools/blender/characters/poses_ladder.py, tools/blender/characters/poses.py, tools/blender/vehicles/car_parts.py
- tests passed: 942
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
