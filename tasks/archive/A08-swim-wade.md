# A08 (Neon): wading, swimming, treading water and climbing out

Files: humanoid/src/swim.rs, humanoid/src/swim_tests.rs, data/swim_anim.json, tools/blender/characters/poses_swim.py, tasks/A08-swim-wade.md, humanoid/src/animator.rs, humanoid/src/lib.rs, tools/blender/characters/poses.py, assets/models/characters/human_player.glb, game/src/player.rs, rotland/src/app/frame_sim.rs, rotland/src/app/state.rs, rotland/src/app/render_frame.rs
Read: humanoid/src/animator.rs, humanoid/src/airborne.rs, humanoid/src/posture.rs, tools/blender/characters/poses.py
API: humanoid/src/layers.rs, humanoid/src/mask.rs, humanoid/src/full_body.rs, game/src/swimming.rs

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 8 of 20 of a deep character animation system. Done so far: layers, weapon stances, aiming, directional legs, feet on the ground, jump/fall/land (`airborne.rs`), crouch walk/sprint/tired (`posture.rs`). Copy the structure of `airborne.rs` and `posture.rs` (config with `from_json` + `load_shared`, a small state with `tick`, a `driven` flag).

**The problem:** in the lake the player walks normally through water and, in deep water, floats upright still walking. (Gameplay is already done: in deep water the body floats at `water_level - chestHeight`, held items are stowed, no sprint, no jump, stamina drains.)

**After this job:**
- **Wading** (water between about knee and chest deep, not swimming): high, slow, pushing steps, arms lifted a little for balance; stronger the deeper the water.
- **Treading water** (deep, slow or still): upright, legs kicking slowly, arms sculling, head above water.
- **Swimming** (deep, moving): body tilted toward horizontal, breaststroke arms and frog kick, head above water. Tempo follows speed.
- **Tired swimming** (deep, stamina low): weaker, slower strokes, head tilted back, struggling.
- **Climbing out**: when the player leaves deep water, one short clip of pushing up and stepping out.
- Animation only: **do not change movement, floating height, stamina or any gameplay.**

## Output rules (important)
- Output **complete files only for**: `humanoid/src/swim.rs`, `humanoid/src/swim_tests.rs`, `data/swim_anim.json`, `tools/blender/characters/poses_swim.py`.
- **Do not output** the other files. In `=== NOTES ===`, give **every** edit for `animator.rs`, `lib.rs` and `poses.py`: file, exact existing lines to find (copied exactly, unique in that file), and the complete replacement. A helper applies them literally. `animator.rs` is already 765 lines, so put the logic in `swim.rs` and add only a few lines there.
- The game wiring edits are fixed below; repeat them in NOTES unchanged.
- **Keep the whole answer under 40,000 characters.** `swim.rs` under 300 lines, tests under 200 lines, clips compact. Do not stop until the final line `END OF EDITS`.
- No new crates. No sounds. No allocation per frame.

## New clips: `tools/blender/characters/poses_swim.py`
Same keyframe format as the other pose files; export `CLIPS`. All in place. The game places the body's feet origin at `water_level - chestHeight` (about 1.25 m below the surface) when swimming, so the hips are under water and the head must stay above it.
- `wade_walk` (loop, 1.1 s): slow high steps pushing through water, arms out low for balance, small body sway.
- `swim_tread` (loop, 1.6 s): upright, slow scissor kick, hands sculling in front at chest height.
- `swim_forward` (loop, 1.2 s): body pitched forward about 60 degrees around the hips (head stays up and looks forward), breaststroke arms (reach forward, sweep out and back, recover), frog kick in time.
- `swim_tired` (loop, 1.6 s): like `swim_tread` but weaker and uneven, head tilted back, an arm sometimes slapping the surface.
- `swim_climb_out` (no loop, 1.1 s): hands push down on the edge in front, one knee comes up, stand up straight.

## `data/swim_anim.json`
```json
{
  "version": 1,
  "wadeMinDepth": 0.35,
  "wadeFullDepth": 1.0,
  "swimMoveSpeed": 0.6,
  "swimFullSpeed": 2.5,
  "tiredStamina": 0.25,
  "blendRate": 6.0,
  "climbOutSeconds": 1.1
}
```
Depth is metres of water above the feet. Speeds are horizontal m/s.

## `humanoid/src/swim.rs`
```rust
pub struct SwimAnimConfig { /* serde camelCase */ }
impl SwimAnimConfig { pub fn from_json(text: &str) -> Result<Self, serde_json::Error>; pub fn load_shared() -> &'static SwimAnimConfig; }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaterPhase { Dry, Wading, Treading, Swimming, Tired }
pub struct SwimState { /* driven flag, phase, smoothed weights, previous swimming flag */ }
impl SwimState {
    pub fn new() -> Self;
    /// depth = water above the feet (0 on land); swimming = the game says deep water.
    pub fn set_input(&mut self, depth: f32, swimming: bool, speed: f32, stamina_frac: f32);
    /// Returns true exactly once when the player leaves deep water (start the climb-out clip).
    pub fn tick(&mut self, dt: f32) -> bool;
    pub fn phase(&self) -> WaterPhase;
    pub fn is_swimming(&self) -> bool; // Treading, Swimming or Tired
    pub fn swim_clip(&self) -> Option<&'static str>; // None unless swimming
    pub fn swim_tempo(&self) -> f32; // playback speed for swim_forward from speed
    pub fn wade_weight(&self) -> f32; // 0..1, smoothed, by depth between wadeMinDepth and wadeFullDepth
}
```
Rules:
- Not driven (setter never called) = everything 0/None/false, so the animator is unchanged and the job 1 golden test still passes.
- Swimming picks Tired when stamina is at or below `tiredStamina`, else Swimming at or above `swimMoveSpeed`, else Treading. Changes between them must not flicker on the edge (small hysteresis).
- Wading only when not swimming. Weights are smoothed with `blendRate`, frame-rate independent; finite at dt 0 and 0.1.

## How the animator uses it (you write these edits in NOTES)
- Field `swim: SwimState`; `pub fn set_water(&mut self, depth: f32, swimming: bool, speed: f32, stamina_frac: f32)`.
- **While swimming**, the swim clip plays on the `Air` layer (jumping is impossible in deep water): skip the air state tick for that frame, set the Air layer clip to `swim_clip()` with target 1. For `swim_forward` scale its playback with `swim_tempo()` if the layer allows a speed; otherwise explain in NOTES. When swimming ends the Air layer goes back to the normal air logic.
- While swimming: foot IK inactive, no landings start, and posture's sprint/crouch weights do not matter (they are off anyway because the game blocks sprint).
- **Wading** blends `wade_walk` on the legs only (Lower mask) by `wade_weight()`, using an existing layer that is free while wading; say which one and why it does not fight posture (posture uses Crouch for crouch/sprint, Aim for crouch walk/catch breath/sprint arms, Additive for tired breath).
- When `tick` returns true, play `swim_climb_out` through `play_full_body` with short fades, but never while dying, in a held pose, or while another full-body clip is active.

## Game wiring (fixed; repeat these in NOTES exactly)
1. `game/src/player.rs`, find `    pub in_deep_water: bool,` and replace with:
```rust
    pub in_deep_water: bool,
    /// Metres of lake water above the feet (0 on land); set each frame by the game loop.
    pub water_depth: f32,
```
   and find `            in_deep_water: false,` and replace with:
```rust
            in_deep_water: false,
            water_depth: 0.0,
```
2. `rotland/src/app/frame_sim.rs`, find `        p.apply_water_movement(&water_res);` and replace with:
```rust
        p.apply_water_movement(&water_res);
        p.water_depth = match water_res.zone {
            world::LakeZone::Shallow | world::LakeZone::Deep if !on_dock => {
                (lake.water_level - p.position.y).max(0.0)
            }
            _ => 0.0,
        };
```
3. `rotland/src/app/state.rs`: in `pub struct PlayerPose`, find `    pub stamina_frac: f32,` and replace with:
```rust
    pub stamina_frac: f32,
    pub water_depth: f32,
    pub swimming: bool,
```
   and find `            first_person: rig.is_first_person,` and replace with:
```rust
            water_depth: p.water_depth,
            swimming: p.in_deep_water,
            first_person: rig.is_first_person,
```
4. `rotland/src/app/render_frame.rs`, find `        if pose.on_floor && !pose.first_person {` and replace with:
```rust
        if !pose.first_person {
            pv.animator.set_water(pose.water_depth, pose.swimming, pose.horizontal_speed, pose.stamina_frac);
        }
        if pose.on_floor && !pose.first_person && !pose.swimming {
```

## Tests: `humanoid/src/swim_tests.rs`
Compact: data parses; all 5 clips are defined in `poses_swim.py` (read as text); never-driven state returns all zero/None/false; wade weight follows depth and is 0 while swimming; Treading/Swimming/Tired chosen correctly with no flicker at the edges; climb-out fires exactly once when swimming turns off and never on land; tempo grows with speed and stays finite; no NaN at dt 0 and 0.1.

Every new number in Rust code needs a source tag on its line (`// data: swim_anim.json key` or `// new: DECISIONS 2026-11-24`).

## Worker notes

## Neon notes

### Design notes

* **Layer choice for wading (`LayerId::Air`):** While wading on the ground (`on_floor == true`, `air_target == 0`), `LayerId::Air` is idle and completely free. Using `Air` with `MaskId::Lower` for `wade_walk` avoids any conflict with `posture.rs` (`Crouch` for crouch/sprint, `Aim` for crouch walk/catch breath/sprint arms, `Additive` for tired breath) as well as weapon `Stance` and `Action` layers. If the player jumps in shallow water, `on_floor` becomes false and `SwimState::release_air` restores `MaskId::Full` and `speed = 1.0` so jump clips play normally.
* **Swim playback speed:** `AnimLayer` exposes `pub speed: f32`, so `SwimState::apply_swim` sets `layer.speed = self.swim_tempo()` for `swim_forward` and `1.0` for `swim_tread` / `swim_tired`.
* **Missing files from Read:** `humanoid/src/lib.rs` was not included under `Read`; the `lib.rs` edit below shows the exact module declarations to add alongside `posture`. Also verify the bone names and `CLIPS` dict structure in `tools/blender/characters/poses_swim.py` against `poses_air.py` / `poses_posture.py` before re-exporting `assets/models/characters/human_player.glb`.

---

### Edits for `humanoid/src/animator.rs`

1. Find:

```rust
use crate::posture::PostureState;
use crate::stance::{HoldTransform, StanceController, StanceTable};

```

Replace with:

```rust
use crate::posture::PostureState;
use crate::stance::{HoldTransform, StanceController, StanceTable};
use crate::swim::{SwimState, SWIM_CLIMB_OUT_CLIP};

```

2. Find:

```rust
    posture: PostureState,
    foot_ik: FootIkState,

```

Replace with:

```rust
    posture: PostureState,
    swim: SwimState,
    foot_ik: FootIkState,

```

3. Find:

```rust
            posture: PostureState::new(),
            foot_ik: FootIkState::new(),

```

Replace with:

```rust
            posture: PostureState::new(),
            swim: SwimState::new(),
            foot_ik: FootIkState::new(),

```

4. Find:

```rust
        if self.air_driven {
            let landing = self.air.tick(step, locomotion.on_floor);
            let target = self.air.air_target();
            let clip = self.air.air_clip().unwrap_or("jump");
            let layer = self.layers.get_mut(LayerId::Air);
            layer.target = target;
            if layer.clip.as_deref() != Some(clip) {
                layer.clip = Some(clip.to_owned());
                layer.time = 0.0;
            }
            if let Some(kind) = landing {
                self.start_landing(kind);
            }
        } else {
            self.layers.get_mut(LayerId::Air).target = if locomotion.on_floor { 0.0 } else { 1.0 };
        }

```

Replace with:

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

```

5. Find:

```rust
        self.foot_ik.set_active(locomotion.on_floor && self.full_body.is_none());

```

Replace with:

```rust
        self.foot_ik
            .set_active(locomotion.on_floor && !self.swim.is_swimming() && self.full_body.is_none());

```

6. Find:

```rust
    /// Feeds crouch, sprint and stamina so the posture layers can react (third person only).
    pub fn set_posture(&mut self, speed: f32, crouching: bool, sprinting: bool, stamina_frac: f32) {
        self.posture.set_input(speed, crouching, sprinting, stamina_frac);
        self.dirty = true;
    }

```

Replace with:

```rust
    /// Feeds crouch, sprint and stamina so the posture layers can react (third person only).
    pub fn set_posture(&mut self, speed: f32, crouching: bool, sprinting: bool, stamina_frac: f32) {
        self.posture.set_input(speed, crouching, sprinting, stamina_frac);
        self.dirty = true;
    }

    /// Feeds lake depth, deep-water flag, horizontal speed and stamina.
    pub fn set_water(&mut self, depth: f32, swimming: bool, speed: f32, stamina_frac: f32) {
        self.swim.set_input(depth, swimming, speed, stamina_frac);
        self.dirty = true;
    }

```

7. Find:

```rust
        // A soft landing only keys legs and hips, so keep the weapon arms on the stance.
        let mask = if kind == LandingKind::Soft { MaskId::Lower } else { MaskId::Full };
        self.layers.get_mut(LayerId::FullBody).mask = mask;
    }

```

Replace with:

```rust
        // A soft landing only keys legs and hips, so keep the weapon arms on the stance.
        let mask = if kind == LandingKind::Soft { MaskId::Lower } else { MaskId::Full };
        self.layers.get_mut(LayerId::FullBody).mask = mask;
    }

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

---

### Edits for `humanoid/src/lib.rs`

(`humanoid/src/lib.rs` was not provided in `Read`; add alongside `posture` / `posture_tests`.)

1. Find:

```rust
pub mod posture;

```

Replace with:

```rust
pub mod posture;
pub mod swim;

```

2. Find:

```rust
#[cfg(test)]
mod posture_tests;

```

Replace with:

```rust
#[cfg(test)]
mod posture_tests;
#[cfg(test)]
mod swim_tests;

```

---

### Edits for `tools/blender/characters/poses.py`

1. Find:

```python
from poses_stance import CLIPS as _STANCE
from poses_use import CLIPS as _USE

```

Replace with:

```python
from poses_stance import CLIPS as _STANCE
from poses_swim import CLIPS as _SWIM
from poses_use import CLIPS as _USE

```

2. Find:

```python
PLAYER_ONLY_CLIPS = {**_PLAYER_ONLY, **_STANCE, **_AIM, **_MOVE, **_AIR, **_POSTURE,
    **_CLIMB, **_CAR, **_USE, **_HURT, **_WOUND}

```

Replace with:

```python
PLAYER_ONLY_CLIPS = {**_PLAYER_ONLY, **_STANCE, **_AIM, **_MOVE, **_AIR, **_POSTURE, **_SWIM,
    **_CLIMB, **_CAR, **_USE, **_HURT, **_WOUND}

```

---

### Game wiring edits (fixed)

1. `game/src/player.rs`, find `    pub in_deep_water: bool,` and replace with:

```rust
    pub in_deep_water: bool,
    /// Metres of lake water above the feet (0 on land); set each frame by the game loop.
    pub water_depth: f32,

```

and find `            in_deep_water: false,` and replace with:

```rust
            in_deep_water: false,
            water_depth: 0.0,

```

2. `rotland/src/app/frame_sim.rs`, find `        p.apply_water_movement(&water_res);` and replace with:

```rust
        p.apply_water_movement(&water_res);
        p.water_depth = match water_res.zone {
            world::LakeZone::Shallow | world::LakeZone::Deep if !on_dock => {
                (lake.water_level - p.position.y).max(0.0)
            }
            _ => 0.0,
        };

```

3. `rotland/src/app/state.rs`: in `pub struct PlayerPose`, find `    pub stamina_frac: f32,` and replace with:

```rust
    pub stamina_frac: f32,
    pub water_depth: f32,
    pub swimming: bool,

```

and find `            first_person: rig.is_first_person,` and replace with:

```rust
            water_depth: p.water_depth,
            swimming: p.in_deep_water,
            first_person: rig.is_first_person,

```

4. `rotland/src/app/render_frame.rs`, find `        if pose.on_floor && !pose.first_person {` and replace with:

```rust
        if !pose.first_person {
            pv.animator.set_water(pose.water_depth, pose.swimming, pose.horizontal_speed, pose.stamina_frac);
        }
        if pose.on_floor && !pose.first_person && !pose.swimming {

```

### Caller steps

* Re-run the Blender character export script to bake the 5 new clips into `assets/models/characters/human_player.glb`.
* Run `cargo test -p humanoid` and `cargo clippy --all-targets -- -D warnings`.

## Gate report
- date: 2026-11-24
- base: main
- files changed (28): assets/models/buildings/settlement.glb, assets/models/characters/human_player.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/swim_anim.json, game/src/player.rs, humanoid/src/animator.rs, humanoid/src/lib.rs, humanoid/src/swim.rs, humanoid/src/swim_tests.rs, rotland/src/app/frame_sim.rs, rotland/src/app/render_frame.rs, rotland/src/app/state.rs, tasks/A08-swim-wade.md, tools/blender/buildings/settlement.py, tools/blender/characters/poses.py, tools/blender/characters/poses_swim.py, tools/blender/vehicles/car_parts.py
- tests passed: 933
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
