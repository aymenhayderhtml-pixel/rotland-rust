# R25-N7 (Neon): special zombies play differently

Files: game/src/zombie/specials.rs, game/src/zombie/mod.rs, game/tests/all/special_zombie_tests.rs, game/tests/all/main.rs, game/src/player.rs, data/special_zombies.json, tasks/R25-N7-special-zombies.md, rotland/src/zombies/mod.rs, rotland/src/zombies/types.rs, rotland/src/zombies/spawning.rs, rotland/src/zombies/combat_glue.rs, rotland/src/zombies/specials.rs, rotland/src/app/frame_sim.rs
Read: game/src/zombie/state.rs, game/src/zombie/defs.rs
API: game/src/zombie/brain_extras.rs, game/src/zombie/mod.rs
Data: data/zombies.json

Rotland is a calm-looking, low-poly zombie survival game. Brute, screamer and crawler mostly differ from walkers only by numbers in `data/zombies.json`. The screamer's call is basic: it screams once per life and alerts zombies with a fixed radius. Write the pure logic for three distinct behaviours. **A helper does the game-loop wiring afterwards, so Neon must NOT edit anything except the four `Files:` above** (do not edit `zombie/mod.rs`, any `brain*.rs` file, `lib.rs` or `tests/all/main.rs`; the helper adds `pub mod specials;` and `mod special_zombie_tests;`). No sounds of any kind. No new crates. No new HUD. The target is a weak laptop with 100+ zombies: `tick` must not allocate, and only the three special kinds get a `SpecialState`. Walkers and runners get no state and no cost.

Current code facts (Neon must not change them, the helper will): the brain already screams once per life for `is_screamer` zombies (`brain_actions.rs` `spot_player`), and `rotland/src/zombies/mod.rs` calls the alert with a hardcoded radius `60.0`. The helper will turn the brain's own screamer scream off for specials and use the new directive instead. Brute and crawler have no special brain code at all, only the crawler's crawl mover.

## How it must play
1. **Brute charge.** When the player is at medium range (between `chargeMinRange` and `chargeMaxRange`) and the brute can see them and is not on cooldown, it stops for a visible wind-up (`windupSeconds`, standing still and facing the player). Then it charges in a straight line at `chargeSpeed` for at most `chargeSeconds`. It turns at most `turnRateDeg` degrees per second while charging. If it reaches the player (horizontal distance within `hitRange`) it deals `hitDamage` with knockback and the charge ends (no stun). If it misses (the time runs out) or hits a wall, it is stunned for `stunSeconds`, and while stunned it is open to extra damage. After any charge it waits `cooldownSeconds` before the next one.
2. **Screamer call.** When it can see the player within `triggerRange`, and is not on cooldown, it stops and takes a visible wind-up (`windupSeconds`, facing the player). It keeps the wind-up even if sight is lost. Killing it during the wind-up (`on_hit` with `killed` true) cancels the call. If the wind-up finishes, it makes one call: the directive carries the player position, `callRadius` and `callMaxZombies`. Then it starts `callCooldownSeconds`, and backs away from the player for `retreatSeconds`.
3. **Crawler lurk and lunge.** When it cannot see the player, or the player is farther than `lurkRange`, it stays low and still (pose Lurk, velocity zero). When the player is closer than `lungeTriggerRange` and it can see them and is not on cooldown, it rolls: with `lungeChance` it lunges in a straight line at `lungeSpeed` for `lungeSeconds`, dealing `lungeDamage` if it reaches the player within `hitRange`. Otherwise it waits `retrySeconds`. After a lunge it recovers for `recoverSeconds` standing still, and it is open to extra damage during recovery (easy to hit). Between `lungeTriggerRange` and `lurkRange` with sight, the brain moves it as usual.

## Data: `data/special_zombies.json`
Starting numbers for the whole file. Numbers copied from `data/zombies.json` are noted; all others are new design numbers (`// new: DECISIONS 2026-11-23`). Write the file exactly like this (camelCase keys):
```json
{
  "vulnerableMultiplier": 2.0,
  "brute": {
    "chargeMinRange": 6.0,
    "chargeMaxRange": 16.0,
    "windupSeconds": 0.9,
    "chargeSeconds": 1.8,
    "chargeSpeed": 6.0,
    "turnRateDeg": 25.0,
    "hitRange": 1.6,
    "hitDamage": 25.0,
    "knockbackSpeed": 5.0,
    "stunSeconds": 2.0,
    "cooldownSeconds": 14.0
  },
  "screamer": {
    "triggerRange": 24.0,
    "windupSeconds": 1.2,
    "callRadius": 25.0,
    "callMaxZombies": 6,
    "callCooldownSeconds": 20.0,
    "retreatSeconds": 2.5,
    "retreatSpeed": 1.8
  },
  "crawler": {
    "lurkRange": 14.0,
    "lungeTriggerRange": 3.0,
    "lungeSpeed": 7.0,
    "lungeSeconds": 0.35,
    "hitRange": 1.0,
    "lungeDamage": 8.0,
    "lungeChance": 0.6,
    "recoverSeconds": 1.2,
    "retrySeconds": 2.0
  }
}
```
Source notes: brute `hitDamage` 25 matches `zombies.json` brute `attackDamage`; brute `hitRange` 1.6 matches brute `attackRange`; screamer `triggerRange` 24 matches screamer `sightRange`; screamer `callRadius` 25 matches `zombies.json` behaviour `screamAlertRadius`; crawler `lurkRange` 14 matches crawler `sightRange`; crawler `hitRange` 1.0 and `lungeDamage` 8 match crawler `attackRange` and `attackDamage`.

## Logic: `game/src/zombie/specials.rs`
Must be pure (no rendering, no physics, no files except `from_repo`). The file is about 300 lines at most. Use these exact names:
```rust
pub enum SpecialKind { Brute, Screamer, Crawler }
impl SpecialKind {
    pub fn from_id(id: &str) -> Option<Self>; // "brute", "screamer", "crawler"; anything else (walker, runner) is None
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpecialPose { #[default] Normal, Windup, Charge, Stunned, Lurk, Lunge, Recover, Scream, Retreat }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpecialHit { pub amount: f32, pub knockback: Vec3 } // knockback: horizontal, same units as the existing combat knockback
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpecialCall { pub target: Vec3, pub radius: f32, pub max_zombies: u32 }
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpecialDirective {
    pub velocity: Option<Vec3>, // horizontal m/s the brain must use; None = brain decides
    pub face: Option<Vec3>,     // unit horizontal direction to turn to; None = brain decides
    pub pose: SpecialPose,
    pub damage: Option<SpecialHit>, // set only on the frame the hit lands
    pub call: Option<SpecialCall>,  // set only on the frame the call is made
    pub stunned: bool,
    pub vulnerable: bool,           // true while open to extra damage (stunned or recovering)
}
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpecialConfig { pub vulnerable_multiplier: f32, pub brute: BruteConfig, pub screamer: ScreamerConfig, pub crawler: CrawlerConfig }
// BruteConfig, ScreamerConfig, CrawlerConfig: one pub field per key above, same derives and serde attributes.
impl SpecialConfig {
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error>;
    pub fn from_repo() -> Result<Self, std::io::Error>; // paths::repo_path("data/special_zombies.json")
}
pub struct SpecialState { /* kind, phase (private enum), timer, cooldown, dir, hit_done */ }
impl SpecialState {
    pub fn new(kind: SpecialKind) -> Self;
    pub fn kind(&self) -> SpecialKind;
    pub fn tick(&mut self, cfg: &SpecialConfig, own: Vec3, player: Vec3, los: bool, hit_wall: bool, dt: f32, roll: f32) -> SpecialDirective;
    pub fn on_hit(&mut self, killed: bool); // killed during a screamer wind-up cancels its call
}
```
Note: `kind` is given once in `new`, not on every `tick`, so the state can never be used for another kind. Only `crawler` uses `roll`; brute and screamer ignore it.

Rules for `tick`:
- Horizontal distance and direction use x and z only. `horizontal_dir` returns `Vec3::ZERO` when the length is below `1e-4`, so identical positions give no NaN.
- Timers count down by `dt`. A phase that ends in a tick hands over at the next tick; two phases never change in one tick. Cooldowns count down during every phase.
- Brute: Idle with no velocity wish until a charge starts. The wind-up sets velocity zero, face to player, pose Windup. Charge: direction is set at wind-up end, then turns toward the player by at most `turnRateDeg * dt` per tick (signed angle from `atan2` on x and z), velocity is `dir * chargeSpeed`, pose Charge. Hit check each tick, once only (`hit_done`). Wall (`hit_wall` while charging) or timeout means stunned: velocity zero, pose Stunned, `stunned` and `vulnerable` true for `stunSeconds`. A hit ends the charge at once, with no stun. Cooldown starts when the charge ends.
- Screamer: the call is emitted on the first tick the wind-up ends, with `velocity` zero and pose Scream. Then pose Retreat with velocity `horizontal_dir(player, own) * retreatSpeed` for `retreatSeconds`, then Idle. `on_hit(true)` during the wind-up sets the phase back to Idle and marks the call cancelled, so no call is ever emitted for that wind-up and no cooldown starts. `vulnerable` is never true for the screamer.
- Crawler: Lurk when `!los` or distance `> lurkRange` (velocity zero, pose Lurk). When distance is between `lungeTriggerRange` and `lurkRange` with sight, velocity `None` (the brain moves it). Inside `lungeTriggerRange` with sight, off cooldown: `roll < lungeChance` starts a lunge (dir fixed at start, velocity `dir * lungeSpeed`, pose Lunge, face dir); otherwise cooldown `retrySeconds`. During a lunge, a hit within `hitRange` is dealt once and ends the lunge early. Lunge end leads to Recover (zero velocity, pose Recover, `vulnerable` true, `recoverSeconds`), then Idle with cooldown `retrySeconds`. The crawler's `damage` has zero knockback.
- `vulnerable` for stunned brute and recovering crawler; the helper multiplies incoming damage by `vulnerableMultiplier` while `vulnerable` is true.
- `tick` returns a plain `Copy` struct. It must not allocate, and must not panic on any input.

## Look
No HUD, no sound, no new egui. The visible part is only the pose and velocity that the helper will map to existing `AnimClip` values. Suggested mapping for the helper (not Neon's job): Normal: brain decides; Windup: Idle; Charge: Run; Stunned: Hit; Lurk: Idle; Lunge: Attack; Recover: Hit; Scream: Scream; Retreat: Walk. The wind-up must be visibly still for its whole time.

## Tests: `game/tests/all/special_zombie_tests.rs`
Use `game::zombie::specials::*` and `SpecialConfig::from_repo()` for the real numbers. Cover, one test per line:
- each wind-up (brute, screamer) is visible (pose Windup, velocity zero) for its full `windupSeconds` (within one `dt`), for dt 0.016;
- brute charge range: no charge closer than `chargeMinRange` or farther than `chargeMaxRange`;
- brute cooldown: after a charge ends, no new charge starts until `cooldownSeconds` have passed, even with the player in range;
- brute misses (charge times out) and is stunned for `stunSeconds`, with `vulnerable` true;
- brute hits a wall while charging and is stunned, `vulnerable` true;
- brute reaches the player: one `damage` with `amount` 25 and knockback pointing away from the brute, no stun, exactly one hit;
- brute turn limit: the charge direction turns by no more than `turnRateDeg * dt` per tick;
- screamer killed in wind-up (`on_hit(true)`) makes no call, ever;
- screamer call: exactly one `call` with `radius` 25 and `max_zombies` 6 at the end of the wind-up, pose Scream, then Retreat moving away from the player;
- screamer call limit and cooldown: a second sighting within `callCooldownSeconds` makes no call, a sighting after it does;
- crawler idle: no sight or distance above `lurkRange` gives pose Lurk, velocity zero;
- crawler lunge range: lunges at 3.0 m but not at 3.5 m (with `roll` 0.0); lunge travel is `lungeSpeed * lungeSeconds` (within 0.05 m);
- crawler recovery: after a lunge, pose Recover with `vulnerable` true for `recoverSeconds`;
- crawler lunge damage: one hit of 8 when the player stays in `hitRange`, no hit when the player steps away;
- no NaN and no panic when `own` and `player` are the same point, for every kind and every pose;
- dt from 0.001 to 0.1 stays stable: totals of each phase match the config within one `dt`, and no NaN in any output;
- `on_hit(false)` changes nothing;
- `SpecialKind::from_id("walker")` and `from_id("runner")` are `None`;
- `data/special_zombies.json` loads with `from_repo()` and every number above matches.

Every new number in Rust code needs a source tag (`// data: special_zombies.json key` or `// new: DECISIONS 2026-11-23`). Test expected values taken from the config must use the config, not a copied literal, unless the test is about the literal in the JSON.

## Notes to write
In `=== NOTES ===` list the exact wiring steps for the helper: the `pub mod specials;` line, one `Option<SpecialState>` per zombie only for special kinds, which brain scream to switch off for specials, which hardcoded `60.0` alert radius to replace with `call.radius`, how `call` gets zombies within `radius` (at most `max_zombies`), and how `damage` reaches the player. Also list any number you changed and why.

## Worker notes

## Neon notes

Helper wiring steps

1.  Register modules & tests:

      - In game/src/zombie/mod.rs, add pub mod specials;.
      - In game/tests/all/main.rs, add mod special_zombie_tests;.

2.  Load config once at startup:

      - Call SpecialConfig::from_repo() alongside ZombieDefs::load_default() and
        store it on the zombie manager / game state.

3.  Per-zombie state (zero cost for walkers and runners):

      - Add special: Option<SpecialState> (and special_vulnerable: bool to
        remember the last frame's directive.vulnerable) to each zombie instance.
      - At spawn time initialize with
        SpecialKind::from_id(type_id).map(SpecialState::new) so only "brute",
        "screamer", and "crawler" carry state; "walker" and "runner" stay None
        and skip tick.

4.  Disable the brain's built-in screamer scream for specials:

      - In brain_actions.rs (spot_player) / zombie spawn setup, do not enable
        the brain's own screamer scream for special screamers: call
        brain.set_screamer(false) and brain.set_scream_chance(0.0) (from
        game/src/zombie/brain_extras.rs) whenever special is Some for a
        screamer, so SpecialState is the sole driver of screamer calls.

5.  Per-frame tick, velocity, facing, and pose mapping:

      - Each frame, for zombies with Some(ref mut special), call: let d =
        special.tick(&special_cfg, zombie_pos, player_pos, has_los,
        hit_wall_last_step, dt, roll); where roll is a uniform [0.0, 1.0) float
        (only read by Crawler) and hit_wall_last_step reports horizontal wall
        contact from the mover.
      - Save zombie.special_vulnerable = d.vulnerable;.
      - If d.velocity is Some(v), override the horizontal movement velocity
        instead of using the brain's velocity; if None, let the brain move the
        zombie as usual.
      - If d.face is Some(dir), set yaw using the Godot convention
        (-dir.x).atan2(-dir.z).
      - Map d.pose to AnimClip: Normal = brain decides; Windup = Idle; Charge =
        Run; Stunned = Hit; Lurk = Idle; Lunge = Attack; Recover = Hit; Scream =
        Scream; Retreat = Walk.

6.  Replace hardcoded 60.0 scream radius & enforce call.max_zombies:

      - In rotland/src/zombies/mod.rs, replace the hardcoded 60.0 alert radius
        with call.radius from d.call.
      - When d.call is Some(call), collect living zombies (excluding the
        screamer itself) whose horizontal distance to the screamer is <=
        call.radius, take at most call.max_zombies as usize (e.g. closest
        first), and call other.brain.alert_chase(call.target).

7.  Player damage & knockback:

      - When d.damage is Some(hit), apply hit.amount to the player via the
        existing player damage handler and apply hit.knockback (horizontal Vec3
        in m/s, same units as existing combat knockback) to the player's
        knockback/velocity.

8.  Incoming zombie damage (vulnerable) & death (on_hit):

      - When a zombie is damaged, if zombie.special_vulnerable is true, multiply
        the incoming damage by special_cfg.vulnerable_multiplier before
        subtracting health.
      - Call special.on_hit(killed) on every hit (killed = health <= 0.0). This
        cancels any active screamer wind-up immediately so a killed screamer
        never emits a call.

Numbers changed

  - None of the design numbers in data/special_zombies.json were changed; every
    key and value matches the task specification verbatim.
  - Internally, SpecialState stores timer and cooldown as f64 with
    TIMER_EPS_FRACTION = 1e-2 (relative to dt) and DIR_EPSILON = 1e-4 so phase
    durations stay accurate within one dt across dt = 0.001..0.1 without f32
    subtraction drift or division-by-zero NaNs. === END ===

## Gate report
- date: 2026-11-23
- base: main
- files changed (28): assets/models/buildings/settlement.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/special_zombies.json, game/src/player.rs, game/src/zombie/mod.rs, game/src/zombie/specials.rs, game/tests/all/main.rs, game/tests/all/special_zombie_tests.rs, rotland/src/app/frame_sim.rs, rotland/src/zombies/combat_glue.rs, rotland/src/zombies/mod.rs, rotland/src/zombies/spawning.rs, rotland/src/zombies/specials.rs, rotland/src/zombies/types.rs, tasks/R25-N7-special-zombies.md, tools/blender/buildings/settlement.py, tools/blender/vehicles/car_parts.py
- tests passed: 849
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
