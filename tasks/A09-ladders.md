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

