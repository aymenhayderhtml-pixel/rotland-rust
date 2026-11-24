# A06 (Neon): jumping, falling, landing and rolling

Files: humanoid/src/airborne.rs, humanoid/src/airborne_tests.rs, data/airborne.json, tools/blender/characters/poses_air.py, tasks/A06-jump-fall-land.md, humanoid/src/animator.rs, humanoid/src/lib.rs, tools/blender/characters/poses.py, assets/models/characters/human_player.glb
Read: humanoid/src/animator.rs, humanoid/src/full_body.rs, tools/blender/characters/poses.py
API: humanoid/src/layers.rs, humanoid/src/motion.rs, humanoid/src/foot_ik.rs

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 6 of 20 of a deep character animation system. Done so far: layers, weapon stances, aiming and recoil, directional legs, feet on the ground.

**The problem:** one looping `jump` clip plays for the whole time the player is off the ground. There is no push-off, no difference between rising and falling, and landing from any height looks the same.

**After this job:** a jump has a clear take-off, rise, top, and fall. Landing depends on how fast the player was falling: a small dip, a heavy crouch, or a forward roll when moving fast. The animation only: **do not change player movement, jump height, fall damage or controls.**

## Output rules (important)
- Output **complete files only for**: `humanoid/src/airborne.rs`, `humanoid/src/airborne_tests.rs`, `data/airborne.json`, `tools/blender/characters/poses_air.py`.
- **Do not output** `animator.rs`, `lib.rs` or `poses.py`. In `=== NOTES ===`, give **every** exact edit: file, exact existing lines to find (copied exactly, unique in that file), and the complete replacement. A helper applies them literally. `animator.rs` is already too long, so keep logic in `airborne.rs` and add only a few lines there.
- **Keep the whole answer under 40,000 characters.** `airborne.rs` under 300 lines, tests under 200 lines, clips written compactly. Do not stop until the final line `END OF EDITS`.
- No new crates. No sounds. No allocation per frame.

## New clips: `tools/blender/characters/poses_air.py`
Same keyframe format as the other pose files; export `CLIPS`. Full body unless stated.
- `air_takeoff` (0.18 s, no loop): quick push off the toes, arms swing up.
- `air_rise` (0.5 s, loop): knees drawn slightly up, body tall.
- `air_fall` (0.6 s, loop): legs reaching down, arms out for balance, small flail.
- `land_soft` (0.28 s, no loop, **legs and hips only**): small knee dip and recover.
- `land_hard` (0.65 s, no loop): deep crouch, one hand toward the ground, slowly stands.
- `land_roll` (0.75 s, no loop): a forward shoulder roll. Rotate the hips through a full turn around their X axis, with the body tucked, and lower the hips with `loc` so the character does not rise above or sink through the ground, then stand.

## `data/airborne.json`
```json
{
  "version": 1,
  "minAirSeconds": 0.12,
  "riseSpeed": 0.6,
  "fallSpeed": -1.2,
  "airBlendRate": 12.0,
  "softLandSpeed": 3.0,
  "hardLandSpeed": 7.5,
  "rollMinHorizontalSpeed": 3.5,
  "landCooldown": 0.25
}
```
`minAirSeconds` ignores tiny hops over bumps and stairs, so that walking down a slope does not trigger falling.

## `humanoid/src/airborne.rs`
```rust
pub struct AirConfig { /* serde camelCase */ }
impl AirConfig { pub fn from_json(text: &str) -> Result<Self, serde_json::Error>; pub fn load_shared() -> &'static AirConfig; }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AirPhase { Grounded, Takeoff, Rise, Fall }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandingKind { Soft, Hard, Roll }
pub struct AirState { /* phase, timers, peak fall speed */ }
impl AirState {
    pub fn new() -> Self;
    pub fn set_velocity(&mut self, velocity: Vec3);
    /// Advances one step. Returns a landing exactly once when one should play.
    pub fn tick(&mut self, dt: f32, on_floor: bool) -> Option<LandingKind>;
    pub fn phase(&self) -> AirPhase;
    pub fn air_clip(&self) -> Option<&'static str>; // None while grounded
    /// 0 until the character has been airborne for `minAirSeconds`, so small bumps do nothing.
    pub fn air_target(&self) -> f32;
}
pub fn landing_clip(kind: LandingKind) -> &'static str;
```
Rules:
- `Takeoff` only when leaving the ground with upward speed above `riseSpeed`. Walking off a ledge goes straight to `Fall` after `minAirSeconds`.
- `Rise` while going up, `Fall` when vertical speed is under `fallSpeed`; between them keep the current phase, so it does not flicker at the top.
- Landing uses the **fastest fall speed in this air time**: below `softLandSpeed` for less than `minAirSeconds` of air is no landing; otherwise Soft. At or above `hardLandSpeed` it is Hard, or Roll if horizontal speed at landing is at least `rollMinHorizontalSpeed`.
- `landCooldown` prevents repeated landings on bumpy ground.
- If `set_velocity` was never called, it must behave so the old `jump` clip and weights are unchanged, so the job 1 golden test still passes (`air_clip` returns None and the animator keeps using `"jump"`).
- Stable and finite at dt 0 and 0.1.

## Exact edits to give in the notes
1. `poses.py`: merge the clips into `PLAYER_ONLY_CLIPS`.
2. `lib.rs`: `pub mod airborne;`, `#[cfg(test)] mod airborne_tests;`, exports.
3. `animator.rs`:
   - Add `air: AirState` and a bool `air_driven` (true after the first `set_move_velocity`).
   - In `set_move_velocity`, also call `self.air.set_velocity(velocity)` and set `air_driven = true`.
   - In `update`, when `air_driven`: tick the state, set the Air layer clip to `air_clip()` (keeping `"jump"` when None), and use `air_target()` as the Air layer target instead of the plain on-floor check. When not `air_driven`, keep the current line.
   - On a landing: Soft plays through `play_action`-style lower-body blending if that exists; if not, use `play_full_body(landing_clip(kind), FullBodyOptions { ..Default::default() })` with short fades. Hard and Roll use `play_full_body`. **Never start a landing while dying, in a held pose, or while another full-body clip is active** (ladders and cars will use those later).
   - For Soft, state exactly how you keep the weapon arms on the stance: prefer a `LayerId::Additive` or a mask-limited approach using the existing layers.

## Tests: `humanoid/src/airborne_tests.rs`
Compact: data parses; all 6 clips are defined in `poses_air.py` (read as text); a bump shorter than `minAirSeconds` does nothing; jump goes Takeoff, Rise, Fall; ledge goes straight to Fall; no flicker at the top; Soft, Hard and Roll by speed; landing fires exactly once; cooldown; without `set_velocity` nothing changes; no NaN at dt 0.

Every new number in Rust code needs a source tag on its line (`// data: airborne.json key` or `// new: DECISIONS 2026-11-23`).

## Worker notes

**Done:** new `airborne.rs` state machine (take-off → rise → fall → landing) and its
`data/airborne.json` values, wired into the animator, plus six new clips in
`tools/blender/characters/poses_air.py` (merged into `PLAYER_ONLY_CLIPS`). The player model was
rebuilt and `assets/models.pack` re-baked, so the clips are in the game data.

**Wiring notes for Claude:**
- `HumanoidAnimator::set_move_velocity` now also feeds the air state. The game already calls it
  for movement, so nothing else is needed. If it is never called, the Air layer behaves exactly
  as before (`"jump"` clip, plain on-floor weight), so job 1's golden test is untouched.
- The Air layer clip changes only when the state machine reports a new phase; the clip timer is
  reset on change. Target weight stays 0 for the first `minAirSeconds` so bumps and slope steps
  do not flicker, except a real push-off (Takeoff) which shows at once.
- Landings go through `play_full_body`: Soft uses the Lower mask so the weapon arms stay on the
  stance, Hard/Roll use the full body. A landing is skipped while dying, in a held pose, or
  while any other full-body clip is running (ladders/cars later).
- Soft vs Hard vs Roll uses the fastest fall speed seen during that air time, with
  `rollMinHorizontalSpeed` deciding Roll at `hardLandSpeed`. `landCooldown` stops repeats on
  bumpy ground.

**Known issues:**
- Gate scope reports 15 files outside this task's `Files:` list. All 15 are the owner's
  pre-existing uncommitted car/settlement changes (previews, `.glb`s, `settlement.py`,
  `car_parts.py`); none were touched by this task. Clippy, tests (914), numbers and unused all pass.
- Rebuilding the player also rewrites `assets/models.pack` (git-ignored) and the
  `human_player_*` preview sheets (git-ignored). `human_player.glb` is in the task's `Files:` list.
- `docs/DECISIONS.md` has no row for 2026-11-23, so no `// new: DECISIONS` tags were used;
  every tuning value is tagged `// data: airborne.json <key>`, and landing fades are derived
  from `airBlendRate`.

## Gate report
- date: 2026-11-24
- base: main
- files changed (24): assets/models/buildings/settlement.glb, assets/models/characters/human_player.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/airborne.json, humanoid/src/airborne.rs, humanoid/src/airborne_tests.rs, humanoid/src/animator.rs, humanoid/src/lib.rs, tasks/A06-jump-fall-land.md, tools/blender/buildings/settlement.py, tools/blender/characters/poses_air.py, tools/blender/characters/poses.py, tools/blender/vehicles/car_parts.py
- tests passed: 914
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
