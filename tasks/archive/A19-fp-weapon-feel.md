# A19 (Neon): first-person weapon that moves with the body

Files: game/src/fp_motion.rs, game/tests/all/fp_motion.rs, data/fp_motion.json, tasks/A19-fp-weapon-feel.md, game/src/lib.rs, game/tests/all/main.rs, rotland/src/fp_weapon.rs, rotland/src/app/frame_actors.rs
Read: rotland/src/fp_weapon.rs, rotland/src/app/frame_actors.rs, game/src/guns.rs
API: game/src/lib.rs, game/src/player.rs

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 19 of a deep character animation system (the third-person body has its own jobs; this one is the first-person view only).

**The problem:** in first person the held weapon is glued to the camera. It has only a small walking bob and a shot/swing kick (`FpSway` in `rotland/src/fp_weapon.rs`). Turning, jumping, landing, sprinting, crouching, reloading and switching weapons do not move it at all, so it feels stiff and floating.

**After this job** the first-person weapon:
- **Lags behind fast turns** (mouse look) and springs back, a little tilt toward the turn.
- **Sprinting:** lowered and turned inward (a "run carry"), with a stronger bob.
- **Crouching:** slightly lower and closer.
- **Jumping:** lags down a little when going up, rises a little while falling; **landing** dips by how hard the landing was, then springs back.
- **Reloading:** dips down and rolls to the side while reloading, back up when done.
- **Switching weapons:** the new weapon rises from below the screen.
- **Idle breathing:** a very small slow sway; bigger when stamina is low.
- Aiming in first person is not changed.
- All of it smooth and frame-rate independent. **Do not change any gameplay, aiming direction, hit detection or the camera.**

## Output rules (important)
- Output **complete files only for**: `game/src/fp_motion.rs`, `game/tests/all/fp_motion.rs`, `data/fp_motion.json`.
- **Do not output** the other files. In `=== NOTES ===`, give **every** edit for `game/src/lib.rs`, `game/tests/all/main.rs`, `rotland/src/fp_weapon.rs` and `rotland/src/app/frame_actors.rs`: file, exact existing lines to find (copied exactly, unique in that file), and the complete replacement. A helper applies them literally.
- **Keep the whole answer under 35,000 characters.** `fp_motion.rs` under 280 lines, tests under 180 lines. Do not stop until the final line `END OF EDITS`.
- No new crates. No sounds. No allocation per frame. Pure logic in `game/` (only `glam` + `serde`), loaded with `paths::repo_path("data/fp_motion.json")` once (see how other configs in `game/` load, e.g. `game/src/swimming.rs` `from_repo`).

## `game/src/fp_motion.rs`
```rust
#[derive(Clone, Copy, Debug, Deserialize)] #[serde(rename_all = "camelCase")]
pub struct FpMotionConfig { /* every number below */ }
impl FpMotionConfig { pub fn from_json(text: &str) -> Result<Self, serde_json::Error>; pub fn from_repo() -> Option<Self>; }
impl Default for FpMotionConfig { /* same values as the json */ }

#[derive(Clone, Copy, Debug, Default)]
pub struct FpMotionInput {
    pub yaw: f32, pub pitch: f32,          // camera angles, radians
    pub horizontal_speed: f32,
    pub vertical_velocity: f32,
    pub on_floor: bool, pub sprinting: bool, pub crouching: bool,
    pub reloading: bool,
    pub weapon_key: u64,                    // changes when the held weapon changes (0 = none)
    pub stamina_frac: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FpMotion { /* springs, last yaw/pitch, timers; Copy, no Vec */ }
impl FpMotion {
    pub fn update(&mut self, dt: f32, input: &FpMotionInput, cfg: &FpMotionConfig);
    /// Camera-space offset (+X right, +Y up, +Z toward the camera), metres.
    pub fn offset(&self) -> glam::Vec3;
    /// Extra rotation in degrees (x pitch, y yaw, z roll), added to the hold rotation.
    pub fn rot_deg(&self) -> glam::Vec3;
}
```
Rules:
- Turn lag: from the yaw/pitch change per second (wrap yaw correctly across ±PI), clamped by `maxLookLag`, via a damped spring (`lookStiffness`, `lookDamping`). A camera teleport/snap (a change larger than `snapAngle` in one frame) is ignored.
- Each effect has a target pose blended with its own rate; landing and switching are springs/timers. The first `update` after creation must not cause a jump (no lag from yaw 0 to the real yaw).
- Equal results at 30 and 120 fps within a small tolerance; finite at dt 0 and 0.25.
- The total offset is clamped so the weapon never comes closer than the existing near clamp (fp_weapon already clamps z; keep yours within `maxOffset`).

## `data/fp_motion.json`
Put every number here: look lag (stiffness, damping, maxLookLag, snapAngle, tilt per lag), sprint pose (offset, rot, bob multiplier), crouch offset, air (rise/fall offsets), landing (dip per m/s, max dip, spring), reload pose (offset, rot), switch (drop distance, raise time), breathing (amplitude, period, tired multiplier). Choose sensible small values (offsets a few centimetres, rotations under 25 degrees) and list them in your answer.

## Edits you write in NOTES
1. `game/src/lib.rs`: `pub mod fp_motion;` and re-export `FpMotion, FpMotionConfig, FpMotionInput`.
2. `game/tests/all/main.rs`: `mod fp_motion;`.
3. `rotland/src/fp_weapon.rs`: add `motion_offset: Vec3` and `motion_rot_deg: Vec3` to `FpSway` (Default zero) and add them in `fp_weapon_matrix` next to the kick values. Keep the existing tests passing.
4. `rotland/src/app/frame_actors.rs`: right after `            actors.fp_sway.update(dt, p.horizontal_speed());` build the `FpMotionInput` from what is available in that function (player `p`: yaw, velocity, on_floor, is_sprinting, crouching, stamina; camera pitch from the camera rig; reloading from the held gun's `is_reloading()` if reachable there; weapon_key from a hash of the held weapon id), call `update` on an `FpMotion` that lives with `fp_sway` (add the field where `fp_sway` is declared and initialised: find it in the code shown, or say exactly where if it is not in the files you see), and copy `offset()`/`rot_deg()` into `actors.fp_sway.motion_offset`/`motion_rot_deg`. The config is loaded once (a field or a `OnceLock`), not every frame.
   If something you need (pitch, reload state, the place `fp_sway` is declared) is not visible in the files you were given, write the edit with a clear `// HELPER:` comment saying what to look up; the helper will find it.

## Tests: `game/tests/all/fp_motion.rs`
Compact: data parses and matches Default; still player stays at rest (only breathing, tiny); a fast turn lags then returns; snap ignored; first update no jump; sprint lowers; crouch lowers; landing dips more for harder landings then recovers; reload dips and returns; switching weapons starts low and rises; 30 vs 120 fps agree; no NaN at dt 0 and 0.25.

Every new number in Rust code needs a source tag on its line (`// data: fp_motion.json key` or `// new: DECISIONS 2026-11-24`).

## Worker notes

## Neon notes

### Tuning values in `data/fp_motion.json`

* Look lag: `lookStiffness = 180.0`, `lookDamping = 22.0`, `lookLagScale = 0.014`, `maxLookLag = 0.045` m, `snapAngle = 1.0` rad, `lookTiltPerLag = 120.0` deg/m (max ~5.4 deg), `lookRotPerLag = 140.0` deg/m (max ~6.3 deg).
* Sprint: `sprintOffset = [-0.03, -0.055, 0.02]` m, `sprintRotDeg = [-10.0, 22.0, 12.0]` deg, `sprintBobMultiplier = 1.8`, `sprintBobAmp = [0.015, 0.012]` m, `sprintBobSpeed = 11.0` rad/s.
* Crouch: `crouchOffset = [-0.01, -0.035, 0.035]` m, `crouchRotDeg = [0.0, 0.0, 4.0]` deg.
* Air: `airRiseOffset = -0.035` m, `airFallOffset = 0.028` m, `airRefSpeed = 5.0` m/s, `airBlendRate = 10.0`.
* Landing: `landingDipPerMps = 0.012` m/(m/s), `landingMaxDip = 0.10` m, `landingStiffness = 140.0`, `landingDamping = 18.0`, `landingPitchPerM = 140.0` deg/m (max ~14 deg).
* Reload: `reloadOffset = [-0.02, -0.09, 0.02]` m, `reloadRotDeg = [-14.0, 8.0, 22.0]` deg, `reloadBlendRate = 10.0`.
* Weapon switch: `switchDropDistance = 0.18` m, `switchRaiseTime = 0.28` s, `switchPitchDeg = -14.0` deg.
* Breathing: `breathAmplitude = [0.0025, 0.004]` m, `breathPeriod = 3.6` s, `breathTiredMultiplier = 2.5`.
* General: `poseBlendRate = 10.0`, `maxOffset = [0.12, 0.24, 0.10]` m.

---

### Literal edits to apply

#### 1. `game/src/lib.rs`

Find:

```rust
pub mod fishing;
pub mod health;

```

Replace with:

```rust
pub mod fishing;
pub mod fp_motion;
pub use fp_motion::{FpMotion, FpMotionConfig, FpMotionInput};
pub mod health;

```

(Note: if `game/src/lib.rs` groups `pub use` re-exports at the bottom of the file, the `pub use` line may optionally be placed there instead.)

---

#### 2. `game/tests/all/main.rs`

`game/tests/all/main.rs` was not included in the Read section.
`// HELPER:` add the following module declaration to `game/tests/all/main.rs` (in alphabetical order with the other `mod ...;` lines):

```rust
mod fp_motion;

```

---

#### 3. `rotland/src/fp_weapon.rs`

Edit A — add `motion_offset` and `motion_rot_deg` to `FpSway` and its `Default` impl.
Find:

```rust
    /// Distance in metres the weapon is pulled toward the camera (+Z) to avoid clipping into walls.
    pub wall_pull: f32,
}

impl Default for FpSway {
    fn default() -> Self {
        Self {
            bob_phase: 0.0,
            move_speed: 0.0,
            kick_offset: Vec3::ZERO,
            kick_rot_deg: Vec3::ZERO,
            wall_pull: 0.0,
        }
    }
}

```

Replace with:

```rust
    /// Distance in metres the weapon is pulled toward the camera (+Z) to avoid clipping into walls.
    pub wall_pull: f32,
    /// Procedural body-coupled translation offset in camera space (+X right, +Y up, +Z toward camera).
    pub motion_offset: Vec3,
    /// Procedural body-coupled extra rotation in degrees (x pitch, y yaw, z roll).
    pub motion_rot_deg: Vec3,
}

impl Default for FpSway {
    fn default() -> Self {
        Self {
            bob_phase: 0.0,
            move_speed: 0.0,
            kick_offset: Vec3::ZERO,
            kick_rot_deg: Vec3::ZERO,
            wall_pull: 0.0,
            motion_offset: Vec3::ZERO,
            motion_rot_deg: Vec3::ZERO,
        }
    }
}

```

Edit B — apply `motion_offset` and `motion_rot_deg` in `fp_weapon_matrix` next to kick values.
Find:

```rust
    let bob = sway.bob_offset();
    let x = hold_pos.x + bob.x + sway.kick_offset.x;
    let y = hold_pos.y + bob.y + sway.kick_offset.y;
    // Positive wall_pull pulls toward the camera along +Z. Clamped so it doesn't cross near plane.
    let z = (hold_pos.z + sway.wall_pull + sway.kick_offset.z).min(-MIN_CAMERA_DISTANCE);
    let local_pos = Vec3::new(x, y, z);

    let rot_x = (hold_rot_deg.x + sway.kick_rot_deg.x).to_radians();
    let rot_y = (hold_rot_deg.y + sway.kick_rot_deg.y).to_radians();
    let rot_z = (hold_rot_deg.z + sway.kick_rot_deg.z).to_radians();

```

Replace with:

```rust
    let bob = sway.bob_offset();
    let x = hold_pos.x + bob.x + sway.kick_offset.x + sway.motion_offset.x;
    let y = hold_pos.y + bob.y + sway.kick_offset.y + sway.motion_offset.y;
    // Positive wall_pull pulls toward the camera along +Z. Clamped so it doesn't cross near plane.
    let z = (hold_pos.z + sway.wall_pull + sway.kick_offset.z + sway.motion_offset.z)
        .min(-MIN_CAMERA_DISTANCE);
    let local_pos = Vec3::new(x, y, z);

    let rot_x = (hold_rot_deg.x + sway.kick_rot_deg.x + sway.motion_rot_deg.x).to_radians();
    let rot_y = (hold_rot_deg.y + sway.kick_rot_deg.y + sway.motion_rot_deg.y).to_radians();
    let rot_z = (hold_rot_deg.z + sway.kick_rot_deg.z + sway.motion_rot_deg.z).to_radians();

```

---

#### 4. `rotland/src/app/state.rs` (not in Read — helper lookup required)

`// HELPER:` `ActorsState` (where `fp_sway: FpSway` is declared and initialised) is in `rotland/src/app/state.rs` (imported via `use super::state::ActorsState;` in `frame_actors.rs`). Add the field right beside `fp_sway`:

```rust
    pub fp_motion: game::FpMotion,

```

and initialise it with `fp_motion: game::FpMotion::default(),` wherever `ActorsState` is constructed.

---

#### 5. `rotland/src/app/frame_actors.rs`

Find:

```rust
            actors.fp_sway.update(dt, p.horizontal_speed());

```

Replace with:

```rust
            actors.fp_sway.update(dt, p.horizontal_speed());
            static FP_MOTION_CFG: std::sync::OnceLock<game::FpMotionConfig> =
                std::sync::OnceLock::new();
            let fp_cfg = FP_MOTION_CFG
                .get_or_init(|| game::FpMotionConfig::from_repo().unwrap_or_default());
            let weapon_key = c.weapon_id().map_or(0, |id| {
                use std::hash::{Hash, Hasher};
                let mut h = std::collections::hash_map::DefaultHasher::new();
                id.hash(&mut h);
                h.finish().max(1)
            });
            let stamina_frac = if p.stamina.max > 0.0 {
                p.stamina.current / p.stamina.max
            } else {
                1.0
            };
            // HELPER: look up the active `game::guns::Gun` (in `play.play` or `actors`) for
            // `c.weapon_id()` and replace `false` with its `.is_reloading()`.
            let reloading = false;
            let fp_in = game::FpMotionInput {
                yaw: rig.yaw,
                pitch: rig.pitch,
                horizontal_speed: p.horizontal_speed(),
                vertical_velocity: p.velocity.y,
                on_floor: p.on_floor,
                sprinting: p.is_sprinting,
                crouching: p.crouching,
                reloading,
                weapon_key,
                stamina_frac,
            };
            // HELPER: requires `pub fp_motion: game::FpMotion` on `ActorsState` (`rotland/src/app/state.rs`).
            actors.fp_motion.update(dt, &fp_in, fp_cfg);
            actors.fp_sway.motion_offset = actors.fp_motion.offset();
            actors.fp_sway.motion_rot_deg = actors.fp_motion.rot_deg();

```

### Known missing items / risks

* `rotland/src/app/state.rs` and `game/tests/all/main.rs` were not in Read; `// HELPER:` comments mark the `ActorsState.fp_motion` field declaration/initialisation and the `mod fp_motion;` registration.
* The held `game::guns::Gun` storage is not visible inside `rotland/src/app/frame_actors.rs` (`play_state.gun` is `Option<WeaponDef>`, not `Gun`); `// HELPER:` marks where to wire `gun.is_reloading()`.
* Note: `rotland/src/fp_weapon.rs` in the prompt had `-` instead of `*` on `cam_world` (chat markdown artifact); Edit A and Edit B avoid touching those lines.

## Gate report
- date: 2026-11-24
- base: main
- files changed (33): assets/models/buildings/settlement.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/fp_motion.json, data/incline.json, game/src/fp_motion.rs, game/src/lib.rs, game/tests/all/fp_motion.rs, game/tests/all/main.rs, humanoid/src/animator.rs, humanoid/src/incline.rs, humanoid/src/incline_tests.rs, humanoid/src/lib.rs, rotland/src/app/frame_actors.rs, rotland/src/app/render_frame.rs, rotland/src/app/state.rs, rotland/src/fp_weapon.rs, rotland/src/play.rs, rotland/src/shooting.rs, tasks/A10-stairs-slopes.md, tasks/A19-fp-weapon-feel.md, tools/blender/buildings/settlement.py, tools/blender/vehicles/car_parts.py
- tests passed: 962
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
