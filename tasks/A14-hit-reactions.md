# A14 (Neon): the player flinches the right way when hit

Files: game/src/hit_react.rs, game/tests/all/hit_react_tests.rs, data/hit_react.json, tasks/A14-hit-reactions.md, game/src/lib.rs, game/tests/all/main.rs, rotland/src/app/state.rs, rotland/src/app/frame_sim.rs
Read: game/src/vault.rs, game/src/zombie/anim_fade.rs, humanoid/src/full_body.rs

Rotland is a low-poly 3D zombie survival game in Rust (custom engine, glam math). This is job 14 of 20 of a deep character animation system. Copy the config style of `game/src/vault.rs` (serde camelCase config with `Default`, `from_json`, `from_repo` via `paths::repo_path`).

**The problem:** whenever a zombie hits the player, the same `hit` clip plays on the upper body, wherever the hit came from. Big hits from special zombies (the brute's shove, which already pushes the player back) look the same as a scratch.

**After this job:**
- A normal hit plays a **directional flinch on the upper body** (legs keep walking): `hit_front`, `hit_back`, `hit_left` or `hit_right` (0.35 s each, already in the player model, from `tools/blender/characters/poses_hurt.py`), chosen from where the attacker stands relative to the way the player faces. Unknown direction → `hit_front`.
- A **big hit** (special-zombie knockback at or above `staggerKnockback` m/s) plays `stagger_back` on the **full body** (0.9 s) when the push comes from the front, and `hit_back` (upper body) when the push comes from behind. Knockdown is not used in this job.
- Several zombies biting in the same moment don't restart the flinch every frame: a new flinch only starts if `minGapSeconds` has passed, **except** a big hit, which always plays.
- **Animation only.** No damage, knockback or control changes.

## `data/hit_react.json`
```json
{
  "version": 1,
  "minGapSeconds": 0.3,
  "staggerKnockback": 4.0,
  "flinchSpeed": 1.0,
  "staggerFadeIn": 0.08,
  "staggerFadeOut": 0.25
}
```

## Logic: `game/src/hit_react.rs`
```rust
pub struct HitReactConfig { ... }  // serde camelCase, Default = JSON above, from_json, from_repo

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitSide { Front, Back, Left, Right }

/// Side of the player the attacker is on. yaw forward = (-sin yaw, 0, -cos yaw); facing east (+X) is yaw -PI/2.
/// Right of the player = forward rotated -90° about Y = (cos yaw?, 0, ...) — derive it carefully and test it.
pub fn hit_side(player_pos: Vec3, player_yaw: f32, attacker_pos: Vec3) -> HitSide; // 4 sectors of 90°, flat (ignore y); zero distance → Front

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitReaction { pub clip: &'static str, pub full_body: bool, pub speed: f32, pub fade_in: f32, pub fade_out: f32 }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitInput {
    pub time: f64,               // game clock in seconds
    pub player_pos: Vec3,
    pub player_yaw: f32,
    pub attacker: Option<Vec3>,  // a zombie that hit this frame, if known
    pub knockback: Vec3,         // biggest special-hit push this frame (ZERO if none)
}

#[derive(Clone, Debug, Default)]
pub struct HitReactState { /* last flinch time */ }
impl HitReactState {
    /// Returns the clip to play for this hit, or None when inside minGapSeconds of the last one (big hits always play).
    pub fn on_hit(&mut self, input: &HitInput, cfg: &HitReactConfig) -> Option<HitReaction>;
}
```
For a big hit, the push direction tells where the attacker is: a push pointing **backward** relative to the player's facing means the hit came from the **front** → `stagger_back`; a push pointing forward means from behind → `hit_back`.

## Wiring (fixed, repeat in NOTES unchanged; the helper applies them literally)
1. `game/src/lib.rs`: `pub mod hit_react;` and `pub use hit_react::{HitInput, HitReactConfig, HitReactState, HitReaction, HitSide, hit_side};` (helper places them next to the `vault` lines).
2. `game/tests/all/main.rs`: `mod hit_react_tests;`.
3. `rotland/src/app/state.rs`: in `pub struct ActorsState` (it derives Default), next to `pub vault: game::VaultState,` add `pub hit_react: game::HitReactState,`.
4. `rotland/src/app/frame_sim.rs`, find:
```rust
        if alive {
            for z_actor in &z.list {
                if z_actor.last_out.as_ref().is_some_and(|o| o.attack_hit.is_some()) {
                    ui.hud.on_damage(z_actor.position);
                }
            }
        }
```
replace:
```rust
        let mut hit_from: Option<Vec3> = None;
        let big_knock = frame
            .special_hits
            .iter()
            .map(|h| h.knockback)
            .max_by(|a, b| a.length_squared().total_cmp(&b.length_squared()))
            .unwrap_or(Vec3::ZERO);
        if alive {
            for z_actor in &z.list {
                if z_actor.last_out.as_ref().is_some_and(|o| o.attack_hit.is_some()) {
                    ui.hud.on_damage(z_actor.position);
                    hit_from.get_or_insert(z_actor.position);
                }
            }
        }
```
(If `Vec3` is not imported in frame_sim.rs, use `glam::Vec3`.)
5. `rotland/src/app/frame_sim.rs`, find:
```rust
            } else if ev.bitten {
                if let Some(pv) = actors.player_visual.as_mut() {
                    pv.animator.play_hit();
                }
```
replace:
```rust
            } else if ev.bitten {
                static HIT_CFG: std::sync::OnceLock<game::HitReactConfig> = std::sync::OnceLock::new();
                let hit_cfg = HIT_CFG.get_or_init(|| game::HitReactConfig::from_repo().unwrap_or_default());
                let player_yaw = actors.player_ctrl.as_ref().map_or(0.0, |p| p.yaw);
                let input = game::HitInput {
                    time: session.time,
                    player_pos: actors.player,
                    player_yaw,
                    attacker: hit_from,
                    knockback: big_knock,
                };
                if let (Some(r), Some(pv)) = (actors.hit_react.on_hit(&input, hit_cfg), actors.player_visual.as_mut()) {
                    if r.full_body {
                        let opts = humanoid::FullBodyOptions {
                            speed: r.speed,
                            hold_last: false,
                            fade_in: r.fade_in,
                            fade_out: r.fade_out,
                            ..humanoid::FullBodyOptions::default()
                        };
                        pv.animator.play_full_body(r.clip, opts);
                    } else {
                        pv.animator.play_action(r.clip, r.speed);
                    }
                }
```
(`session.time` is f64 here; if it is f32, cast. If a borrow error appears, the helper reorders.)

## Output rules (important)
- Output **complete files only for**: `game/src/hit_react.rs`, `game/tests/all/hit_react_tests.rs`, `data/hit_react.json`.
- **Do not output** other files. In `=== NOTES ===`, repeat the fixed wiring above.
- **Keep the whole answer under 20,000 characters.** `hit_react.rs` under 180 lines, tests under 160 lines. Finish with the final line `END OF EDITS`.
- No new crates. No sounds. No allocation per frame.
- Every new number in Rust code needs a source tag (`// data: hit_react.json key` or `// new: DECISIONS 2026-11-24`).
- Answer format: each full file as `=== FILE: path ===`, a code fence, `=== END ===`. Then `=== NOTES ===` with the edits, then `=== END ===`, then `END OF EDITS`.

## Tests: `game/tests/all/hit_react_tests.rs`
Config parses and equals Default; `hit_side` for yaw 0 (facing -Z): attacker at -Z → Front, +Z → Back, +X → Right, -X → Left; same for yaw -PI/2 (facing +X): attacker at +X → Front, +Z → Right; zero distance → Front; `on_hit` with no attacker → `hit_front`; second hit within minGapSeconds → None, after → Some; big push pointing backward → `stagger_back` full body, even inside the gap; big push pointing forward → `hit_back` upper body; small push below threshold → normal directional flinch.

## Worker notes
