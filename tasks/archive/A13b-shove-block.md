# A13b (Neon): shove and block with melee weapons

Files: game/src/melee_guard.rs, game/tests/all/melee_guard_tests.rs, data/melee_guard.json, tools/blender/characters/poses_guard.py, tasks/A13b-shove-block.md, game/src/lib.rs, game/tests/all/main.rs, game/src/zombie/brain.rs, rotland/src/zombies/combat_glue.rs, rotland/src/combat.rs, rotland/src/app/state.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_sim.rs, data/stances.json, tools/blender/characters/poses.py
Read: game/src/melee.rs, game/src/vault.rs, tools/blender/characters/poses_combat.py, humanoid/src/stance.rs
API: game/src/player.rs, game/src/weapons.rs
Data: data/stances.json

Rotland is a low-poly 3D zombie survival game in Rust (custom engine, glam math). This is job 13b of a deep character animation system. Job 13a (just done) added melee combos (`game/src/melee_combo.rs`). Copy the config style of `game/src/vault.rs` (serde camelCase config with `Default` equal to the JSON, `from_json`, `from_repo` via `paths::repo_path`).

**The problem:** with a melee weapon the player can only attack. Crowds of zombies swarm them with no way to make space or defend.

**After this job (owner's choices, DECISIONS 2026-11-24), the right mouse button with a melee weapon in hand:**
- **Tap (released within `tapSeconds`) = shove.** Zombies within `range` in a front `arcDeg` cone (nearest first, at most `maxTargets`) are pushed away (`push` m/s, flat) and **staggered** for `staggerSeconds` (they stop attacking/moving, the same stagger a strong hit gives). No damage. Costs `staminaCost`; not possible without enough stamina or during `cooldown`. Zombie kinds in `immuneKinds` (the brute) are not pushed or staggered. The `shove` clip plays on the upper body.
- **Hold (longer than `tapSeconds`) = block.** The weapon comes up in front (the stance's aim layer plays `block_1h` / `block_2h`, legs keep walking). While blocking, **zombie bites from the front `arcDeg`** do `damageScale` of their damage and each costs `staminaPerHit`; `block_hit` plays on the upper body. Blocking drains `drainPerSecond` stamina. Bites from behind or the side, special-zombie hits and animal bites are not blocked. Block needs at least `minStamina` to start.
- **Guard break:** if stamina runs out while blocking (or a blocked hit costs more than is left, then that hit does full damage), the block drops and cannot start again for `breakSeconds`.
- No attacking while blocking. No shove/block while dead, in a car, climbing, in build mode, while a swing is winding up, or with a gun, bow, fishing rod or a throwable weapon (the spear keeps its right-click throw).

## `data/melee_guard.json` (use exactly)
```json
{
  "version": 1,
  "tapSeconds": 0.2,
  "shove": {
    "range": 1.8, "arcDeg": 100, "maxTargets": 3, "staminaCost": 15,
    "push": 4.0, "staggerSeconds": 0.9, "cooldown": 0.8,
    "clip": "shove", "clipSpeed": 1.0, "immuneKinds": ["brute"]
  },
  "block": {
    "arcDeg": 120, "damageScale": 0.3, "staminaPerHit": 12, "drainPerSecond": 3,
    "minStamina": 5, "breakSeconds": 1.5, "hitClip": "block_hit"
  }
}
```

## Logic: `game/src/melee_guard.rs`
```rust
pub struct GuardConfig { ... }   // + ShoveConfig, BlockConfig. Default = the JSON above. from_json, from_repo ("data/melee_guard.json")

#[derive(Clone, Copy, Debug, Default)]
pub struct GuardInput { pub held: bool, pub can_guard: bool }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardEvent { None, Shove, BlockStart, BlockEnd, GuardBroken }

#[derive(Clone, Debug, Default)]
pub struct GuardState { /* held time, blocking, shove cooldown left, break time left, was held last frame */ }
impl GuardState {
    /// One frame. Press → start timing; held past tapSeconds → block (if stamina >= minStamina and not broken);
    /// released before tapSeconds → shove (pays stamina, starts cooldown) if allowed. Blocking drains stamina
    /// (also reset `stamina.time_since_use` like Stamina::try_use); at 0 → GuardBroken. can_guard false → block ends, nothing starts.
    pub fn update(&mut self, dt: f32, input: GuardInput, stamina: &mut Stamina, cfg: &GuardConfig) -> GuardEvent;
    pub fn is_blocking(&self) -> bool;
    /// One zombie bite on the player. Returns (damage to take, blocked). Blocked only when blocking and the attacker
    /// is inside the front arc (flat, yaw forward = (-sin yaw, 0, -cos yaw)); unknown attacker → not blocked.
    pub fn filter_bite(&mut self, damage: f32, player_pos: Vec3, player_yaw: f32, attacker: Option<Vec3>, stamina: &mut Stamina, cfg: &GuardConfig) -> (f32, bool);
}

/// Targets a shove reaches, nearest first, at most maxTargets: pushes `(id, push velocity)` into `out` (cleared first).
pub fn shove_targets(position: Vec3, facing: Vec3, targets: &[MeleeTarget], cfg: &GuardConfig, out: &mut Vec<(u64, Vec3)>);
```
Range check on flat distance minus the target's `radius`. No allocation per frame (`out` is reused by the caller).

## Clips: `tools/blender/characters/poses_guard.py`
`CLIPS = {...}`, format of `poses_combat.py` (compact one-line-per-frame style of `gun_bash`; bones spine, chest, neck, head, upper_arm_R/L, forearm_R/L, hand_R; (pitch, yaw, roll) degrees, same signs). Upper body only.
- `shove` 0.45 s, no loop: both arms pull in (forearms ≈ 100) then push straight forward at chest height (upper arms ≈ 80, forearms ≈ 15) by 0.15 s, chest leans forward −10, back to the `melee_1h` rest pose by 0.45 s.
- `block_1h` 0.5 s, **loop** (a held pose; all frames nearly equal, tiny breathing motion): right forearm raised across the face, weapon held up and across (upper_arm_R ≈ 70, forearm_R ≈ 110, hand_R roll ≈ 70), left arm up as a guard (upper_arm_L ≈ 50, forearm_L ≈ 120), chest slightly hunched (−6), head down a little.
- `block_2h` 0.5 s, **loop**: both hands raise the weapon horizontally in front of the face (upper arms ≈ 75, forearms ≈ 100, both hands in front), chest −6.
- `block_hit` 0.3 s, no loop: from the `block_2h` pose, both arms and chest jolt back (chest +8, upper arms −15) at 0.08 s, then back to `block_2h` by 0.3 s.

## Wiring (fixed, repeat in NOTES unchanged; the helper applies them literally)
1. `game/src/lib.rs`: `pub mod melee_guard;` and `pub use melee_guard::{GuardConfig, GuardEvent, GuardInput, GuardState, shove_targets};` (next to the `melee_combo` lines).
2. `game/tests/all/main.rs`: `mod melee_guard_tests;`.
3. `game/src/zombie/brain.rs`, in `impl ZombieBrain`, after `pub fn last_seen(...)` add:
```rust
    /// Staggers the zombie for at least `seconds` (a shove): no moving or attacking until it ends.
    pub fn stagger_for(&mut self, seconds: f32) {
        self.stagger_timer = self.stagger_timer.max(seconds);
    }
```
4. `rotland/src/zombies/combat_glue.rs`, in `impl Zombies` after the `hit` function add:
```rust
    /// A shove from the player: flat push and stagger, no damage. Returns false for dead or immune zombies.
    pub fn shove(&mut self, id: u64, push: Vec3, stagger_seconds: f32, immune_kinds: &[String]) -> bool {
        let Some(z) = self.list.iter_mut().find(|z| z.id == id) else {
            return false;
        };
        if z.health.is_dead() || immune_kinds.iter().any(|k| *k == z.kind) {
            return false;
        }
        let flat = Vec3::new(push.x, 0.0, push.z);
        z.knock += flat;
        z.last_knock = flat;
        z.brain.stagger_for(stagger_seconds);
        true
    }
```
5. `rotland/src/combat.rs`, in `impl Combat` after `pub fn is_dead(...)` add:
```rust
    /// True while a melee swing or bash is winding up.
    pub fn swing_busy(&self) -> bool {
        self.swing.state() == SwingState::WindingUp
    }
```
6. `rotland/src/app/state.rs`: in `pub struct ActorsState`, next to `pub vault: game::VaultState,` add `pub guard: game::GuardState,` and `pub shove_scratch: Vec<(u64, Vec3)>,` (use `glam::Vec3` if `Vec3` is not imported there).
7. `data/stances.json`: in `classes.melee_1h` add `"aimClip": "block_1h",`; in `classes.melee_2h` add `"aimClip": "block_2h",` and set its `"gripWhenAiming": true`. In NOTES, also give a new `aimHold.rot` for `melee_2h` if the weapon must turn sideways to lie across the hands (say which value and why), or say none is needed.
8. `tools/blender/characters/poses.py`: after the `_MELEE_V2` import add `from poses_guard import CLIPS as _GUARD  # A13b: shove and block`; find `**_WOUND, **_MELEE_V2` replace `**_WOUND, **_MELEE_V2, **_GUARD`.
9. `rotland/src/app/frame_actors.rs`:
   - Find `            let ev = c.update(dt, attack, p, &mut prey, &world.physics);` replace `            let ev = c.update(dt, attack && !actors.guard.is_blocking(), p, &mut prey, &world.physics);`
   - Find:
```rust
        let captured = ui.cursor_captured && !ui.screen_open();
        let right_pressed = captured && input.mouse_pressed(MouseButton::Right);
```
replace:
```rust
        let captured = ui.cursor_captured && !ui.screen_open();
        let right_pressed = captured && input.mouse_pressed(MouseButton::Right);
        static GUARD_CFG: std::sync::OnceLock<game::GuardConfig> = std::sync::OnceLock::new();
        let guard_cfg = GUARD_CFG.get_or_init(|| game::GuardConfig::from_repo().unwrap_or_default());
        let guard_weapon = actors.combat.as_ref().and_then(|c| c.weapon_def()).is_some_and(|w| {
            !w.is_gun() && !w.is_bow() && !w.throwable && w.id != "fishing_rod"
        });
        let swing_busy = actors.combat.as_ref().is_some_and(|c| c.swing_busy());
        let guard_in = game::GuardInput {
            held: captured && input.mouse_held(MouseButton::Right),
            can_guard: !dead && !in_car && !climbing && !play.build_mode.active && guard_weapon && !swing_busy,
        };
        match actors.guard.update(dt, guard_in, &mut p.stamina, guard_cfg) {
            game::GuardEvent::Shove => {
                if let Some(z) = actors.zombies.as_mut() {
                    let facing = Vec3::new(-p.yaw.sin(), 0.0, -p.yaw.cos());
                    game::shove_targets(p.position, facing, &z.targets(), guard_cfg, &mut actors.shove_scratch);
                    for &(id, push) in &actors.shove_scratch {
                        z.shove(id, push, guard_cfg.shove.stagger_seconds, &guard_cfg.shove.immune_kinds);
                    }
                }
                if let Some(pv) = actors.player_visual.as_mut() {
                    pv.animator.play_action(&guard_cfg.shove.clip, guard_cfg.shove.clip_speed);
                }
            }
            game::GuardEvent::GuardBroken => {
                if let Some(play_state) = play.play.as_mut() {
                    play_state.show_toast("Guard broken!".to_string());
                }
            }
            _ => {}
        }
```
   - Find `        if let Some(pv) = actors.player_visual.as_mut() { pv.animator.set_aiming(aiming); pv.animator.set_look_pitch(rig.pitch); }` replace:
```rust
        let melee_in_hand = actors.combat.as_ref().and_then(|c| c.weapon_def()).is_some_and(|w| !w.is_gun() && !w.is_bow() && !w.throwable);
        let anim_aiming = if melee_in_hand { actors.guard.is_blocking() } else { aiming };
        if let Some(pv) = actors.player_visual.as_mut() { pv.animator.set_aiming(anim_aiming); pv.animator.set_look_pitch(rig.pitch); }
```
   (If `show_toast` takes `&str`, drop `.to_string()`. If a borrow error appears, the helper reorders.)
10. `rotland/src/app/frame_sim.rs`, find `        bites.extend(frame.bites.iter().map(|b| b.damage));` replace:
```rust
        {
            static GUARD_CFG: std::sync::OnceLock<game::GuardConfig> = std::sync::OnceLock::new();
            let guard_cfg = GUARD_CFG.get_or_init(|| game::GuardConfig::from_repo().unwrap_or_default());
            // frame.bites are pushed in z.list order, so the biters line up with them
            let mut biters = z
                .list
                .iter()
                .filter(|zz| zz.last_out.as_ref().is_some_and(|o| o.attack_hit.is_some()))
                .map(|zz| zz.position);
            let mut blocked_any = false;
            for b in &frame.bites {
                let attacker = biters.next();
                let dmg = if let Some(pc) = actors.player_ctrl.as_mut() {
                    let (d, blocked) = actors.guard.filter_bite(b.damage, pc.position, pc.yaw, attacker, &mut pc.stamina, guard_cfg);
                    blocked_any |= blocked;
                    d
                } else {
                    b.damage
                };
                bites.push(dmg);
            }
            if blocked_any && let Some(pv) = actors.player_visual.as_mut() {
                pv.animator.play_action(&guard_cfg.block.hit_clip, 1.0);
            }
        }
```
11. Helper after applying: rebuild the player model (`blender -b --python tools/blender/characters/build_characters.py`) and bake (`cargo run --release -p baker`) under the heavy lock, then the gate.

## Output rules (important)
- Output **complete files only for**: `game/src/melee_guard.rs`, `game/tests/all/melee_guard_tests.rs`, `data/melee_guard.json`, `tools/blender/characters/poses_guard.py`.
- **Do not output** other files. In `=== NOTES ===`, repeat the fixed wiring above (plus your `aimHold` answer for step 7).
- **Keep the whole answer under 25,000 characters.** `melee_guard.rs` under 230 lines, tests under 180 lines. Finish with the final line `END OF EDITS`.
- No new crates. No sounds. No allocation per frame.
- Every new number in Rust code needs a source tag (`// data: melee_guard.json key` or `// new: DECISIONS 2026-11-24`).
- Answer format: each full file as `=== FILE: path ===`, a code fence, `=== END ===`. Then `=== NOTES ===` with the edits, then `=== END ===`, then `END OF EDITS`.

## Tests: `game/tests/all/melee_guard_tests.rs`
Config parses and equals Default; `from_repo` loads. Tap: press then release before tapSeconds → `Shove`, stamina −staminaCost; second tap inside cooldown → `None`; tap with too little stamina → `None`. Hold: past tapSeconds → `BlockStart`, `is_blocking`; release → `BlockEnd`, no shove. Drain: stamina drops by drainPerSecond × dt; at 0 → `GuardBroken`, then holding again inside breakSeconds does not block, after it does. `can_guard` false while blocking → `BlockEnd`. `filter_bite`: yaw 0 (facing −Z), attacker at −Z while blocking → damage × damageScale, blocked, stamina −staminaPerHit; attacker at +Z → full damage, not blocked; not blocking → full; unknown attacker → full; not enough stamina for the hit → full damage, block broken. `shove_targets`: targets in front within range in order nearest first, capped at maxTargets; target behind or too far is skipped; push points away from the player with length `push`; `out` is cleared each call.

## Worker notes

Hand-applied (Haiku, 2026-11-24):
- game/tests/all/melee_guard_tests.rs: Stamina import path fixed (game::melee::MeleeTarget); Stamina built with struct update syntax (clippy).
- rotland/src/zombies/combat_glue.rs: immune check uses immune_kinds.contains (clippy).
- game/src/melee_guard.rs: data tags changed from shove.X / block.X to the leaf key X (the gate checks the leaf name in the JSON).
- data/stances.json: NOT applied. aimClip on melee_1h/melee_2h (and gripWhenAiming on melee_2h) broke existing humanoid stance tests (aim_is_ignored_for_melee_and_rod, weapon_swap_resets_aim_and_clears_old_clips), which encode the DECISIONS 2026-11-23 rule that melee ignores aim. Owner/orchestrator decision needed; until then the block pose is not wired and aim-block only drives the guard logic.
- aimHold.rot for melee_2h: not changed (no verified value).
- Model rebuilt (build_characters.py) and baked (cargo run --release -p baker) after wiring.
Gate: clippy PASS, tests PASS (998). Scope FAIL for the same shared-checkout reason as A13a; numbers and unused PASS.

## Neon notes

Note: `data/stances.json` was not included in the Read section of the prompt; see step 7 below for `aimHold.rot`.

Fixed wiring edits to apply:

1. `game/src/lib.rs`: `pub mod melee_guard;` and `pub use melee_guard::{GuardConfig, GuardEvent, GuardInput, GuardState, shove_targets};` (next to the `melee_combo` lines).
2. `game/tests/all/main.rs`: `mod melee_guard_tests;`.
3. `game/src/zombie/brain.rs`, in `impl ZombieBrain`, after `pub fn last_seen(...)` add:

```rust
    /// Staggers the zombie for at least `seconds` (a shove): no moving or attacking until it ends.
    pub fn stagger_for(&mut self, seconds: f32) {
        self.stagger_timer = self.stagger_timer.max(seconds);
    }

```

4. `rotland/src/zombies/combat_glue.rs`, in `impl Zombies` after the `hit` function add:

```rust
    /// A shove from the player: flat push and stagger, no damage. Returns false for dead or immune zombies.
    pub fn shove(&mut self, id: u64, push: Vec3, stagger_seconds: f32, immune_kinds: &[String]) -> bool {
        let Some(z) = self.list.iter_mut().find(|z| z.id == id) else {
            return false;
        };
        if z.health.is_dead() || immune_kinds.iter().any(|k| *k == z.kind) {
            return false;
        }
        let flat = Vec3::new(push.x, 0.0, push.z);
        z.knock += flat;
        z.last_knock = flat;
        z.brain.stagger_for(stagger_seconds);
        true
    }

```

5. `rotland/src/combat.rs`, in `impl Combat` after `pub fn is_dead(...)` add:

```rust
    /// True while a melee swing or bash is winding up.
    pub fn swing_busy(&self) -> bool {
        self.swing.state() == SwingState::WindingUp
    }

```

6. `rotland/src/app/state.rs`: in `pub struct ActorsState`, next to `pub vault: game::VaultState,` add `pub guard: game::GuardState,` and `pub shove_scratch: Vec<(u64, Vec3)>,` (use `glam::Vec3` if `Vec3` is not imported there).
7. `data/stances.json`: in `classes.melee_1h` add `"aimClip": "block_1h",`; in `classes.melee_2h` add `"aimClip": "block_2h",` and set its `"gripWhenAiming": true`.
* `aimHold.rot` for `melee_2h`: set `"aimHold": { "rot": [0.0, 0.0, 90.0] }` (or add +90 deg roll relative to `melee_2h`'s existing `hold.rot` in `data/stances.json`) so the 2H haft turns sideways to lie horizontally across both hands in `block_2h`. (Verify the shaft axis against `classes.melee_2h.hold.rot` in `data/stances.json`, since `data/stances.json` was not in the Read context.)


8. `tools/blender/characters/poses.py`: after the `_MELEE_V2` import add `from poses_guard import CLIPS as _GUARD  # A13b: shove and block`; find `**_WOUND, **_MELEE_V2` replace `**_WOUND, **_MELEE_V2, **_GUARD`.
9. `rotland/src/app/frame_actors.rs`:

* Find `            let ev = c.update(dt, attack, p, &mut prey, &world.physics);` replace `            let ev = c.update(dt, attack && !actors.guard.is_blocking(), p, &mut prey, &world.physics);`
* Find:

```rust
        let captured = ui.cursor_captured && !ui.screen_open();
        let right_pressed = captured && input.mouse_pressed(MouseButton::Right);

```

replace:

```rust
        let captured = ui.cursor_captured && !ui.screen_open();
        let right_pressed = captured && input.mouse_pressed(MouseButton::Right);
        static GUARD_CFG: std::sync::OnceLock<game::GuardConfig> = std::sync::OnceLock::new();
        let guard_cfg = GUARD_CFG.get_or_init(|| game::GuardConfig::from_repo().unwrap_or_default());
        let guard_weapon = actors.combat.as_ref().and_then(|c| c.weapon_def()).is_some_and(|w| {
            !w.is_gun() && !w.is_bow() && !w.throwable && w.id != "fishing_rod"
        });
        let swing_busy = actors.combat.as_ref().is_some_and(|c| c.swing_busy());
        let guard_in = game::GuardInput {
            held: captured && input.mouse_held(MouseButton::Right),
            can_guard: !dead && !in_car && !climbing && !play.build_mode.active && guard_weapon && !swing_busy,
        };
        match actors.guard.update(dt, guard_in, &mut p.stamina, guard_cfg) {
            game::GuardEvent::Shove => {
                if let Some(z) = actors.zombies.as_mut() {
                    let facing = Vec3::new(-p.yaw.sin(), 0.0, -p.yaw.cos());
                    game::shove_targets(p.position, facing, &z.targets(), guard_cfg, &mut actors.shove_scratch);
                    for &(id, push) in &actors.shove_scratch {
                        z.shove(id, push, guard_cfg.shove.stagger_seconds, &guard_cfg.shove.immune_kinds);
                    }
                }
                if let Some(pv) = actors.player_visual.as_mut() {
                    pv.animator.play_action(&guard_cfg.shove.clip, guard_cfg.shove.clip_speed);
                }
            }
            game::GuardEvent::GuardBroken => {
                if let Some(play_state) = play.play.as_mut() {
                    play_state.show_toast("Guard broken!".to_string());
                }
            }
            _ => {}
        }

```

* Find `        if let Some(pv) = actors.player_visual.as_mut() { pv.animator.set_aiming(aiming); pv.animator.set_look_pitch(rig.pitch); }` replace:

```rust
        let melee_in_hand = actors.combat.as_ref().and_then(|c| c.weapon_def()).is_some_and(|w| !w.is_gun() && !w.is_bow() && !w.throwable);
        let anim_aiming = if melee_in_hand { actors.guard.is_blocking() } else { aiming };
        if let Some(pv) = actors.player_visual.as_mut() { pv.animator.set_aiming(anim_aiming); pv.animator.set_look_pitch(rig.pitch); }

```

10. `rotland/src/app/frame_sim.rs`, find `        bites.extend(frame.bites.iter().map(|b| b.damage));` replace:

```rust
        {
            static GUARD_CFG: std::sync::OnceLock<game::GuardConfig> = std::sync::OnceLock::new();
            let guard_cfg = GUARD_CFG.get_or_init(|| game::GuardConfig::from_repo().unwrap_or_default());
            // frame.bites are pushed in z.list order, so the biters line up with them
            let mut biters = z
                .list
                .iter()
                .filter(|zz| zz.last_out.as_ref().is_some_and(|o| o.attack_hit.is_some()))
                .map(|zz| zz.position);
            let mut blocked_any = false;
            for b in &frame.bites {
                let attacker = biters.next();
                let dmg = if let Some(pc) = actors.player_ctrl.as_mut() {
                    let (d, blocked) = actors.guard.filter_bite(b.damage, pc.position, pc.yaw, attacker, &mut pc.stamina, guard_cfg);
                    blocked_any |= blocked;
                    d
                } else {
                    b.damage
                };
                bites.push(dmg);
            }
            if blocked_any && let Some(pv) = actors.player_visual.as_mut() {
                pv.animator.play_action(&guard_cfg.block.hit_clip, 1.0);
            }
        }

```

11. After applying: rebuild the player model (`blender -b --python tools/blender/characters/build_characters.py`) and bake (`cargo run --release -p baker`) under the heavy lock, then run the gate.

## Gate report
- date: 2026-11-24
- base: main
- files changed (38): assets/models/buildings/settlement.glb, assets/models/characters/human_player.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/melee_guard.json, data/melee_moves.json, docs/DECISIONS.md, game/src/lib.rs, game/src/melee_combo.rs, game/src/melee_guard.rs, game/src/zombie/brain.rs, game/tests/all/main.rs, game/tests/all/melee_combo_tests.rs, game/tests/all/melee_guard_tests.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_sim.rs, rotland/src/app/state.rs, rotland/src/combat.rs, rotland/src/zombies/combat_glue.rs, tasks/A12-actions.md, tasks/A13a-melee-combos.md, tasks/A13b-shove-block.md, tasks/A14-hit-reactions.md, tools/blender/buildings/settlement.py, tools/blender/characters/poses_guard.py, tools/blender/characters/poses_melee_v2.py, tools/blender/characters/poses.py, tools/blender/vehicles/car_parts.py
- tests passed: 998
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
