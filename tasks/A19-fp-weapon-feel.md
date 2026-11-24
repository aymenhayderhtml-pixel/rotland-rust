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

