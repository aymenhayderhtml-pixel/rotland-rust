# A12 (Neon): body animations for picking up, looting, eating, drinking, bandaging, building and fishing

Files: humanoid/src/action_anim.rs, humanoid/src/action_anim_tests.rs, data/action_anim.json, tools/blender/characters/poses_use_v2.py, tasks/A12-actions.md, humanoid/src/animator.rs, humanoid/src/lib.rs, tools/blender/characters/poses.py, rotland/src/play.rs, rotland/src/app/frame_input.rs, rotland/src/app/frame_water.rs, rotland/src/app/frame_actors.rs
Read: humanoid/src/animator.rs, humanoid/src/full_body.rs, humanoid/src/car_anim.rs, tools/blender/characters/poses_air.py

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 12 of 20 of a deep character animation system. Done so far: layers, stances, aiming, directional legs, foot IK, jump/land, crouch/sprint/tired, swimming, ladders, stairs, car seat (`car_anim.rs`), vaulting. Copy the structure of `car_anim.rs` (config with `from_json` + a shared loader, a small state with a `driven` flag, helpers so `animator.rs` gets only a few lines).

**The problem:** when the player picks something up, opens a loot crate, eats, drinks, uses a bandage, places a building piece or fishes, the body does nothing. The player model already has clips for all of these (in `tools/blender/characters/poses_use.py`): `pickup_low` (0.8 s once), `loot_search` (1.6 s loop), `eat` (1.5 s once), `drink` (1.5 s once), `bandage` (1.2 s loop), `hammer_build` (0.7 s loop), `fish_cast` (1.0 s once), `fish_reel` (0.8 s loop).

**After this job:** each of those actions plays its clip once (looping clips play for a set time from the data file):
- **Pick up** an item from the ground: `pickup_low`, full body (bends down), 0.8 s.
- **Open a loot crate:** `loot_search`, full body, 1.2 s.
- **Eat / drink / bandage** (from the hotbar or bag): `eat` / `drink` / `bandage`, **upper body only**, so the player can keep walking. Bandage plays 1.2 s.
- **Place a building piece:** `hammer_build`, upper body, 0.7 s.
- **Fishing:** cast → `fish_cast` (upper body, once). Bite → `fish_reel` loops on the upper body until the fish is caught, escapes or the cast is cancelled (`FishStop` cue), then fades out.
- A full-body action is **cancelled with a fade** when the player moves faster than `cancelSpeed` (so walking away is never frozen in a pose). Upper-body actions are never cancelled by moving.
- A new cue replaces the one playing. Nothing plays while dying, in the car, on a ladder or swimming (use the animator's existing flags; if there is no simple flag for one of them, skip that check and say so in NOTES).
- **Gameplay does not change** (no timing, no locking the player).

## Fix one weak clip: `tools/blender/characters/poses_use_v2.py`
The old `drink` reads badly. Write a new `drink` in `poses_use_v2.py` (`CLIPS = {"drink": {...}}`, 1.5 s, no loop, same format as poses_air.py): right upper arm brings the bottle up to the mouth (upper_arm_R X ≈ 100-120, forearm_R X ≈ 110-130), head tips back (head X ≈ +20 to +25 at the peak 0.5-1.0 s), chest leans back a little (+5), left arm stays relaxed, then back to rest by 1.5 s. Sign rules: spine/chest/head X negative = bend forward (positive = lean back); upper_arm X positive = arm forward/up (0 = hanging); forearm X positive = elbow bends. Upper body bones only. In `poses.py` import it and merge it **after** the old use clips so it overrides: `PLAYER_ONLY_CLIPS = {..., **_USE, **_USE_V2, ...}` (give the exact edit in NOTES).

## `data/action_anim.json`
```json
{
  "version": 1,
  "cancelSpeed": 1.2,
  "cues": {
    "pickUp":    { "clip": "pickup_low",   "fullBody": true,  "seconds": 0.8, "speed": 1.0, "fadeIn": 0.12, "fadeOut": 0.2 },
    "openCrate": { "clip": "loot_search",  "fullBody": true,  "seconds": 1.2, "speed": 1.0, "fadeIn": 0.15, "fadeOut": 0.25 },
    "eat":       { "clip": "eat",          "fullBody": false, "seconds": 1.5, "speed": 1.0, "fadeIn": 0.12, "fadeOut": 0.2 },
    "drink":     { "clip": "drink",        "fullBody": false, "seconds": 1.5, "speed": 1.0, "fadeIn": 0.12, "fadeOut": 0.2 },
    "bandage":   { "clip": "bandage",      "fullBody": false, "seconds": 1.2, "speed": 1.0, "fadeIn": 0.12, "fadeOut": 0.2 },
    "build":     { "clip": "hammer_build", "fullBody": false, "seconds": 0.7, "speed": 1.0, "fadeIn": 0.08, "fadeOut": 0.15 },
    "fishCast":  { "clip": "fish_cast",    "fullBody": false, "seconds": 1.0, "speed": 1.0, "fadeIn": 0.1,  "fadeOut": 0.2 },
    "fishReel":  { "clip": "fish_reel",    "fullBody": false, "seconds": 0.0, "speed": 1.0, "fadeIn": 0.15, "fadeOut": 0.3 }
  }
}
```
`seconds: 0` means "loop until a stop cue". Tune nothing else.

## Logic: `humanoid/src/action_anim.rs`
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionCue { PickUp, OpenCrate, Eat, Drink, Bandage, Build, FishCast, FishReel, FishStop }
```
Plus the config, and an `ActionAnimState` that owns timing and decides which layer and weight each frame. You choose how it drives the layers: reuse the existing full-body path (`play_full_body` / `FullBodyState`) for full-body cues and the existing upper-body action layer (`play_action` / `ActionState`) for upper-body cues, or drive a layer directly like `car_anim.rs` does, whichever needs fewer `animator.rs` lines. Looping clips must stop after `seconds` (or on `FishStop`) with a fade, not a pop. No allocation per frame.

Animator API to add (few lines in `animator.rs`):
```rust
/// Starts the body animation for something the player just did.
pub fn play_cue(&mut self, cue: ActionCue);
```
Moving speed is already fed by `set_move_velocity` (read animator.rs). **If `play_cue` is never called, the animator behaves exactly as before** (the job 1 golden test must pass unchanged).

## Wiring (fixed, repeat in NOTES unchanged; the helper applies them literally)
1. `rotland/src/play.rs`, find:
```rust
    /// Something was picked up or a crate opened this frame (pickup sound).
    pub picked_up: bool,
```
replace:
```rust
    /// Something was picked up or a crate opened this frame (pickup sound).
    pub picked_up: bool,
    /// Body animation for something the player just did (taken by frame_actors).
    pub anim_cue: Option<humanoid::ActionCue>,
```
2. `rotland/src/play.rs`, find `            picked_up: false,` replace `            picked_up: false,\n            anim_cue: None,`
3. `rotland/src/play.rs`, find:
```rust
                    self.camp.bounties.on_crate_looted();
                    self.picked_up = true;
```
replace:
```rust
                    self.camp.bounties.on_crate_looted();
                    self.picked_up = true;
                    self.anim_cue = Some(humanoid::ActionCue::OpenCrate);
```
4. `rotland/src/play.rs`, find:
```rust
                InteractResult::PickedUpItem { toast, item_id, count, .. } => {
                    self.picked_up = true;
```
replace:
```rust
                InteractResult::PickedUpItem { toast, item_id, count, .. } => {
                    self.picked_up = true;
                    self.anim_cue = Some(humanoid::ActionCue::PickUp);
```
5. `rotland/src/play.rs`, find `        self.use_cooldown = USE_COOLDOWN;` replace:
```rust
        self.use_cooldown = USE_COOLDOWN;
        self.anim_cue = Some(match def.category {
            ItemCategory::Food => humanoid::ActionCue::Eat,
            ItemCategory::Drink => humanoid::ActionCue::Drink,
            _ => humanoid::ActionCue::Bandage,
        });
```
6. `rotland/src/app/frame_input.rs`, find:
```rust
            if placed
                && let (Some(toast), Some(play_state)) = (
```
replace:
```rust
            if placed && let Some(ps) = play.play.as_mut() {
                ps.anim_cue = Some(humanoid::ActionCue::Build);
            }
            if placed
                && let (Some(toast), Some(play_state)) = (
```
7. `rotland/src/app/frame_water.rs`, find `    let event = play.water.fishing.tick(&input, dt);` replace:
```rust
    let event = play.water.fishing.tick(&input, dt);
    let fish_cue = match event.as_ref() {
        Some(FishingEvent::Cast) => Some(humanoid::ActionCue::FishCast),
        Some(FishingEvent::Bite) => Some(humanoid::ActionCue::FishReel),
        Some(FishingEvent::Caught | FishingEvent::Escaped | FishingEvent::Cancelled) => Some(humanoid::ActionCue::FishStop),
        None => None,
    };
    if let (Some(cue), Some(ps)) = (fish_cue, play.play.as_mut()) {
        ps.anim_cue = Some(cue);
    }
```
8. `rotland/src/app/frame_actors.rs`, find `        let mut swung = None;` replace:
```rust
        if let Some(cue) = play.play.as_mut().and_then(|ps| ps.anim_cue.take())
            && let Some(pv) = actors.player_visual.as_mut()
        {
            pv.animator.play_cue(cue);
        }
        let mut swung = None;
```
9. `humanoid/src/lib.rs`: `pub mod action_anim;`, `#[cfg(test)] mod action_anim_tests;`, and `pub use action_anim::ActionCue;` (anchor on the existing `car_anim` lines; the helper places them).

## Output rules (important)
- Output **complete files only for**: `humanoid/src/action_anim.rs`, `humanoid/src/action_anim_tests.rs`, `data/action_anim.json`, `tools/blender/characters/poses_use_v2.py`.
- **Do not output** other files. In `=== NOTES ===`, give every edit for `animator.rs`, `lib.rs` and `poses.py` (exact existing lines to find, unique in that file, and the complete replacement), then the fixed wiring above.
- **Keep the whole answer under 30,000 characters.** `action_anim.rs` under 260 lines, tests under 200 lines. Finish with the final line `END OF EDITS`.
- No new crates. No sounds. No allocation per frame.
- Every new number in Rust code needs a source tag (`// data: action_anim.json key` or `// new: DECISIONS 2026-11-24`).
- Answer format: each full file as `=== FILE: path ===`, a code fence, `=== END ===`. Then `=== NOTES ===` with the edits, then `=== END ===`, then `END OF EDITS`.

## Tests: `humanoid/src/action_anim_tests.rs`
Data parses and has all 8 cues; every clip name used exists in poses_use.py or poses_use_v2.py (read the files as text); never-called changes nothing; full-body cue cancels (fades) above cancelSpeed and not below; upper-body cue is not cancelled by speed; a `seconds` cue ends after its time; `FishReel` keeps going until `FishStop`; a new cue replaces the old one; stable and finite at dt 0 and 0.1.

## Worker notes

Applied by Claude (orchestrator helper, 2026-11-24).
- Full files from neon/in applied with neon_apply; animator.rs, lib.rs and poses.py edits applied from NOTES; fixed wiring applied literally.
- Neon slip fixed by hand: action_anim_tests.rs had a broken string literal (`contains("ed)`); now `contains(&quoted)`.
- Data tags shortened: the gate checks the key text as a plain substring, so `cues.pickUp.seconds` failed; tags now name the JSON key only (`seconds`, `speed`, `fadeIn`, `fadeOut`).
- Gate: clippy PASS, tests PASS (1015 passed, whole workspace incl. A14), numbers PASS, unused PASS. Scope FAIL only from owner's uncommitted files and the A14 files (expected).
- Player model rebuilt with build_characters.py and models.pack baked.

## Neon notes

### Edits for `humanoid/src/animator.rs`

1. Add import — find:

```rust
use crate::action::ActionState;

```

replace:

```rust
use crate::action::ActionState;
use crate::action_anim::{ActionAnimCommand, ActionAnimState, ActionCue};

```

2. Add field to `HumanoidAnimator` — find:

```rust
    action: Option<ActionState>,
    full_body: Option<FullBodyState>,

```

replace:

```rust
    action: Option<ActionState>,
    action_anim: ActionAnimState,
    full_body: Option<FullBodyState>,

```

3. Initialize field in `HumanoidAnimator::with_config` — find:

```rust
            action: None,
            full_body: None,

```

replace:

```rust
            action: None,
            action_anim: ActionAnimState::new(),
            full_body: None,

```

4. Tick `action_anim` in `HumanoidAnimator::update` — find:

```rust
        self.aim.tick(step);
        self.advance_action(step);

```

replace:

```rust
        self.aim.tick(step);
        self.advance_action(step);
        if self.action_anim.is_driven() {
            let blocked = in_car || on_ladder || self.swim.is_swimming();
            if let ActionAnimCommand::StopFullBody { fade_out } =
                self.action_anim.tick(step, blocked)
            {
                self.stop_full_body(Some(fade_out));
            }
            if self.action.is_none() {
                self.action_anim.apply_upper(&mut self.layers);
            }
        }

```

5. Feed horizontal movement speed in `HumanoidAnimator::set_move_velocity` — find:

```rust
    pub fn set_move_velocity(&mut self, velocity: Vec3, yaw: f32) {
        self.motion.set_input(velocity, yaw);
        self.air.set_velocity(velocity);
        self.air_driven = true;
        self.dirty = true;
    }

```

replace:

```rust
    pub fn set_move_velocity(&mut self, velocity: Vec3, yaw: f32) {
        self.motion.set_input(velocity, yaw);
        self.air.set_velocity(velocity);
        self.action_anim.set_move_speed(velocity.x.hypot(velocity.z));
        self.air_driven = true;
        self.dirty = true;
    }

```

6. Add `play_cue` method — find:

```rust
    pub fn play_hit(&mut self) {
        self.play_action("hit", 1.0); // godot: HumanoidAnimTree.cs:152
    }

```

replace:

```rust
    /// Starts the body animation for something the player just did.
    pub fn play_cue(&mut self, cue: ActionCue) {
        let blocked = self.dying
            || self.held_pose.is_some()
            || self.car.is_seated()
            || self.ladder.is_on_ladder()
            || self.swim.is_swimming();
        match self.action_anim.play_cue(cue, blocked) {
            ActionAnimCommand::None => {}
            ActionAnimCommand::StartFullBody { cue, opts } => {
                if let Some(clip) = self.action_anim.clip_name(cue) {
                    self.play_full_body(clip, opts);
                }
            }
            ActionAnimCommand::StopFullBody { fade_out } => {
                self.stop_full_body(Some(fade_out));
            }
        }
        self.dirty = true;
    }

    pub fn play_hit(&mut self) {
        self.play_action("hit", 1.0); // godot: HumanoidAnimTree.cs:152
    }

```

7. Render upper-body cue in `HumanoidAnimator::apply_action_stage` — find:

```rust
    fn apply_action_stage(&mut self, rig: &Rig) {
        let Some(act) = &self.action else { return };

```

replace:

```rust
    fn apply_action_stage(&mut self, rig: &Rig) {
        let Some(act) = &self.action else {
            if self.action_anim.is_upper_active() {
                self.apply_layer(rig, LayerId::Action);
            }
            return;
        };

```

### Edits for `humanoid/src/lib.rs`

Anchor on the existing `car_anim` lines:

* Add `pub mod action_anim;`
* Add `#[cfg(test)] mod action_anim_tests;`
* Add `pub use action_anim::ActionCue;`

### Edits for `tools/blender/characters/poses.py`

1. Import `CLIPS as _USE_V2` from `poses_use_v2` right after the `poses_use` import:

```python
from poses_use_v2 import CLIPS as _USE_V2

```

2. Merge `**_USE_V2` immediately after `**_USE` in `PLAYER_ONLY_CLIPS`:

```python
PLAYER_ONLY_CLIPS = {..., **_USE, **_USE_V2, ...}

```

### Fixed wiring (unchanged from task spec)

1. `rotland/src/play.rs`, find:

```rust
    /// Something was picked up or a crate opened this frame (pickup sound).
    pub picked_up: bool,

```

replace:

```rust
    /// Something was picked up or a crate opened this frame (pickup sound).
    pub picked_up: bool,
    /// Body animation for something the player just did (taken by frame_actors).
    pub anim_cue: Option<humanoid::ActionCue>,

```

2. `rotland/src/play.rs`, find `            picked_up: false,` replace `            picked_up: false,\n            anim_cue: None,`
3. `rotland/src/play.rs`, find:

```rust
                    self.camp.bounties.on_crate_looted();
                    self.picked_up = true;

```

replace:

```rust
                    self.camp.bounties.on_crate_looted();
                    self.picked_up = true;
                    self.anim_cue = Some(humanoid::ActionCue::OpenCrate);

```

4. `rotland/src/play.rs`, find:

```rust
                InteractResult::PickedUpItem { toast, item_id, count, .. } => {
                    self.picked_up = true;

```

replace:

```rust
                InteractResult::PickedUpItem { toast, item_id, count, .. } => {
                    self.picked_up = true;
                    self.anim_cue = Some(humanoid::ActionCue::PickUp);

```

5. `rotland/src/play.rs`, find `        self.use_cooldown = USE_COOLDOWN;` replace:

```rust
        self.use_cooldown = USE_COOLDOWN;
        self.anim_cue = Some(match def.category {
            ItemCategory::Food => humanoid::ActionCue::Eat,
            ItemCategory::Drink => humanoid::ActionCue::Drink,
            _ => humanoid::ActionCue::Bandage,
        });

```

6. `rotland/src/app/frame_input.rs`, find:

```rust
            if placed
                && let (Some(toast), Some(play_state)) = (

```

replace:

```rust
            if placed && let Some(ps) = play.play.as_mut() {
                ps.anim_cue = Some(humanoid::ActionCue::Build);
            }
            if placed
                && let (Some(toast), Some(play_state)) = (

```

7. `rotland/src/app/frame_water.rs`, find `    let event = play.water.fishing.tick(&input, dt);` replace:

```rust
    let event = play.water.fishing.tick(&input, dt);
    let fish_cue = match event.as_ref() {
        Some(FishingEvent::Cast) => Some(humanoid::ActionCue::FishCast),
        Some(FishingEvent::Bite) => Some(humanoid::ActionCue::FishReel),
        Some(FishingEvent::Caught | FishingEvent::Escaped | FishingEvent::Cancelled) => Some(humanoid::ActionCue::FishStop),
        None => None,
    };
    if let (Some(cue), Some(ps)) = (fish_cue, play.play.as_mut()) {
        ps.anim_cue = Some(cue);
    }

```

8. `rotland/src/app/frame_actors.rs`, find `        let mut swung = None;` replace:

```rust
        if let Some(cue) = play.play.as_mut().and_then(|ps| ps.anim_cue.take())
            && let Some(pv) = actors.player_visual.as_mut()
        {
            pv.animator.play_cue(cue);
        }
        let mut swung = None;

```

9. `humanoid/src/lib.rs`: `pub mod action_anim;`, `#[cfg(test)] mod action_anim_tests;`, and `pub use action_anim::ActionCue;` (anchor on the existing `car_anim` lines; the helper places them).

### Missing context notes

* `humanoid/src/car_anim.rs` and `tools/blender/characters/poses_air.py` were not included in the Read section; `action_anim.rs` follows the `car_anim` structure described in the task prompt, and `poses_use_v2.py` uses the standard `CLIPS = {"drink": {"duration", "loop", "bones"}}` format.

## Gate report
- date: 2026-11-24
- base: main
- files changed (35): assets/models/buildings/settlement.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/action_anim.json, data/hit_react.json, game/src/hit_react.rs, game/src/lib.rs, game/tests/all/hit_react_tests.rs, game/tests/all/main.rs, humanoid/src/action_anim.rs, humanoid/src/action_anim_tests.rs, humanoid/src/animator.rs, humanoid/src/lib.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_input.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_water.rs, rotland/src/app/state.rs, rotland/src/play.rs, tasks/A12-actions.md, tasks/A14-hit-reactions.md, tools/blender/buildings/settlement.py, tools/blender/characters/poses.py, tools/blender/characters/poses_use_v2.py, tools/blender/vehicles/car_parts.py
- tests passed: 1015
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
