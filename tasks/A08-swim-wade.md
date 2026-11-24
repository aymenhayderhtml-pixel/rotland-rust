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

