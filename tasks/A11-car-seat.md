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

