# A10b (Neon): jumping over low walls and fences (vault)

Files: game/src/vault.rs, game/tests/all/vault_tests.rs, data/vault.json, tasks/A10b-vault.md, game/src/lib.rs, game/tests/all/main.rs, rotland/src/app/frame_actors.rs, rotland/src/app/state.rs
Read: game/src/player.rs, game/src/ladders/climb.rs, game/src/ladders/tick.rs, physics/src/world.rs, game/tests/all/melee_tests.rs, humanoid/src/full_body.rs
API: rotland/src/app/frame_actors.rs (lines 236-260 shown in the wiring section)

Rotland is a low-poly 3D zombie survival game in Rust (custom engine, rapier physics, glam math). This is a gameplay job added to the character animation plan. The owner decided (2026-11-24) that low walls and fences get a **real jump-over move**.

**The problem:** a low wall or fence (about 0.5 m to 1.25 m high) blocks the player. The only way over is a normal jump, which often fails, and looks bad. The player model already has two clips for this (in `tools/blender/characters/poses_climb.py`, both in place, the game moves the body): `vault_low` (0.7 s, one hand on top, legs swing over, for ~1 m obstacles) and `step_over` (0.6 s, high step, for ~0.5 m obstacles).

**After this job:** when the player presses **jump** while on the ground, standing (not crouched), moving forward toward a low, thin obstacle right in front, the player does a short scripted vault instead of a jump: the body moves up over the top and down on the other side along a smooth path, and the full-body clip plays (`step_over` for low ones, `vault_low` for higher ones). If there is no vaultable obstacle, jump works exactly as today. Walls too high, too thick (a building), or with no floor behind stay blocked.

## Output rules (important)
- Output **complete files only for**: `game/src/vault.rs`, `game/tests/all/vault_tests.rs`, `data/vault.json`.
- **Do not output** other files. In `=== NOTES ===`, give every edit for `game/src/lib.rs` and `game/tests/all/main.rs` (exact existing lines to find, unique in the file, and the complete replacement). A helper applies them literally.
- The game wiring edits are fixed below; repeat them in NOTES unchanged (you may fix obvious mistakes, say so).
- **Keep the whole answer under 30,000 characters.** `vault.rs` under 280 lines, tests under 200 lines. Finish with the final line `END OF EDITS`.
- No new crates. No sounds. No allocation per frame (the path is a fixed array). No new clips.
- Every number in Rust code needs a source tag: `// data: vault.json <key>` or `// new: DECISIONS 2026-11-24`.
- Answer format: each full file as `=== FILE: path ===`, a code fence, `=== END ===`. Then `=== NOTES ===` with the edits, then `=== END ===`, then `END OF EDITS`.

## Coordinates and physics you need
- Y is up. Player `position` is the **feet** (bottom of the capsule). Stand height 1.8 m. Forward from yaw: `Vec3::new(-yaw.sin(), 0.0, -yaw.cos())`.
- `physics::PhysicsWorld::raycast(origin: Vec3, dir: Vec3, max_distance: f32) -> Option<Hit>` hits static world only (ground chunks, props, buildings, fences), never the player. `Hit` has `point: Vec3`, `normal: Vec3`, `distance: f32` (check `physics/src/types.rs` names when unsure; read `physics/src/world.rs`).
- Tests: build a world like `game/tests/all/melee_tests.rs` (flat ground chunk with `add_ground_chunk`, a box wall with `add_static(&[points], position, yaw, scale)`), then `world.update_broad_phase()` if that file does so.
- `game::Player` fields used: `position`, `velocity`, `yaw`, `on_floor`, `crouching`, `stamina` (`try_jump()` spends the jump cost and returns false if too tired), `in_deep_water`.

## `data/vault.json`
```json
{
  "version": 1,
  "probeDistance": 0.9,
  "minHeight": 0.45,
  "maxHeight": 1.25,
  "stepOverMaxHeight": 0.75,
  "maxThickness": 0.7,
  "landingDistance": 0.6,
  "maxLandingDrop": 1.0,
  "clearanceAbove": 0.9,
  "topLift": 0.15,
  "minForwardInput": 0.3,
  "vaultSeconds": 0.7,
  "stepOverSeconds": 0.6,
  "speedAfter": 3.0
}
```
Meaning: look up to `probeDistance` m ahead; vault only if the top is `minHeight`..`maxHeight` above the feet; under `stepOverMaxHeight` use `step_over`, else `vault_low`; the wall must end within `maxThickness` m; the landing point is `landingDistance` m past the far side and its ground may be at most `maxLandingDrop` m lower (and not higher than the top); `clearanceAbove` m of free space above the top is needed (a ray at top + clearance must not hit). The body rises `topLift` above the top. `speedAfter` m/s of forward velocity is kept on landing so running continues.

## Logic: `game/src/vault.rs`
```rust
pub struct VaultConfig { ... }              // serde camelCase, Default = the JSON above
impl VaultConfig { pub fn from_json(s: &str) -> Result<Self, String>; pub fn from_repo() -> Result<Self, String>; } // from_repo reads paths::repo_path("data/vault.json") (see how game::FpMotionConfig::from_repo does it in game/src/fp_motion.rs)

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VaultKind { StepOver, Vault }
impl VaultKind { pub fn clip(self) -> &'static str } // "step_over" / "vault_low"

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VaultPlan { pub kind: VaultKind, pub start: Vec3, pub top: Vec3, pub land: Vec3, pub seconds: f32 }

/// Pure probe: returns a plan if a vaultable obstacle is straight ahead of `feet` along `forward` (unit, flat).
pub fn find_vault(physics: &PhysicsWorld, feet: Vec3, forward: Vec3, cfg: &VaultConfig) -> Option<VaultPlan>;

#[derive(Clone, Debug, Default)]
pub struct VaultState { /* active plan, elapsed */ }
impl VaultState {
    pub fn is_active(&self) -> bool;
    /// Starts a vault if allowed: on_floor, not crouching, not in deep water, forward input >= minForwardInput,
    /// a plan is found along the player's facing (player.yaw), and stamina.try_jump() succeeds (check stamina LAST).
    /// Returns the clip to play when it starts.
    pub fn try_start(&mut self, player: &mut Player, physics: &PhysicsWorld, forward_input: f32, cfg: &VaultConfig) -> Option<&'static str>;
    /// Moves the player along the path. Returns true while still vaulting. On the last frame puts the feet
    /// exactly on `land`, sets velocity to forward * speedAfter (y = 0), on_floor = true.
    pub fn step(&mut self, player: &mut Player, dt: f32) -> bool;
    pub fn cancel(&mut self);
}
```
Probe steps (`find_vault`):
1. Low ray from `feet + up*0.3` along forward, `probeDistance`: must hit (the wall face); else `None`. Use the hit distance `d`.
2. High ray from `feet + up*(maxHeight + 0.05)` along forward, `d + 0.2`: must NOT hit (else the wall is too tall).
3. Down ray from `feet + forward*(d + 0.1) + up*(maxHeight + 0.1)` straight down: the hit is the top; `h = top.y - feet.y` must be in `minHeight..=maxHeight`.
4. Clearance: ray up from the top point + 0.02 must not hit within `clearanceAbove`.
5. Thickness: walk forward from `d + 0.1` in 0.1 m steps up to `d + maxThickness`; at each step a down ray from `top.y + 0.1`: the first step whose hit is lower than `top.y - 0.3` (or no hit) is the far edge. No far edge found = too thick, `None`.
6. Landing: down ray at `far_edge + forward*landingDistance` from `top.y + 0.1`: must hit ground with `top.y - hit.y >= 0.3` and `feet.y - hit.y <= maxLandingDrop`.
7. Plan: `start = feet`, `top = (point above wall middle at height top.y + topLift)`, `land = landing hit point`, kind by `h`, seconds from config.
Path in `step`: smoothstep time `t = elapsed/seconds`; first half moves start→top, second half top→land, both on a quadratic curve so it never dips into the wall (e.g. first half: lerp xz linearly, y eases out toward `top.y`; second half: xz linear, y eases in toward `land.y`). `dt <= 0` changes nothing. Finite for dt up to 0.1.

## Wiring (fixed, repeat in NOTES)
1. `game/src/lib.rs`: `pub mod vault;` and `pub use vault::{VaultConfig, VaultKind, VaultPlan, VaultState, find_vault};` (put them next to the other `pub mod` / `pub use` lines; copy an existing line as anchor).
2. `game/tests/all/main.rs`: add `mod vault_tests;`.
3. `rotland/src/app/state.rs`: in `pub struct ActorsState`, next to the field `pub fp_motion: ...`, add `pub vault: game::VaultState,` and in its constructor/`Default` add `vault: game::VaultState::default(),` (the helper finds the right place).
4. `rotland/src/app/frame_actors.rs`, replace exactly:
```rust
        if in_car {
            // the car moves the player (above)
        } else if climbing {
            rig.yaw = p.yaw;
        } else {
            p.update(actions, rig.yaw, &world.physics, &actors.mover, dt);
        }
```
with:
```rust
        static VAULT_CFG: std::sync::OnceLock<game::VaultConfig> = std::sync::OnceLock::new();
        let vault_cfg = VAULT_CFG.get_or_init(|| game::VaultConfig::from_repo().unwrap_or_default());
        if dead || in_car || climbing {
            actors.vault.cancel();
        }
        if in_car {
            // the car moves the player (above)
        } else if climbing {
            rig.yaw = p.yaw;
        } else if actors.vault.is_active() {
            actors.vault.step(p, dt);
        } else if let Some(clip) = (!dead && actions.jump)
            .then(|| actors.vault.try_start(p, &world.physics, -actions.move_vec.y, vault_cfg))
            .flatten()
        {
            if let Some(pv) = actors.player_visual.as_mut() {
                let mut opts = humanoid::FullBodyOptions::from_config(pv.animator.config());
                opts.hold_last = false;
                pv.animator.play_full_body(clip, opts);
            }
        } else {
            p.update(actions, rig.yaw, &world.physics, &actors.mover, dt);
        }
```
(`-actions.move_vec.y` is forward input, as in the ladder code just above. If `pv.animator.config()` does not exist, write a `// HELPER:` note saying "build FullBodyOptions the way other play_full_body callers do".)

## Tests: `game/tests/all/vault_tests.rs`
Use a flat ground at y = 0 and box walls. Cover: config parses and equals Default; a 1.0 m high, 0.3 m thick wall 0.5 m ahead gives `Vault` with land beyond the wall; a 0.6 m wall gives `StepOver`; a 2.0 m wall gives `None`; a 1.0 m high but 3 m thick block gives `None`; no wall gives `None`; wall behind the player (facing away) gives `None`; `try_start` refuses while crouching, in the air, with no forward input, and with empty stamina (stamina unchanged when refused for another reason); `step` reaches `land` after `seconds` at dt 1/60 and at dt 0.1, never goes below `top.y - 0.05` while above the wall's xz span, and stays finite; `dt = 0` does nothing.

## Worker notes
