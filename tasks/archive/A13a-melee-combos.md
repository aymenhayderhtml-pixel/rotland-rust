# A13a (Neon): melee combos and weapon weight

Files: game/src/melee_combo.rs, game/tests/all/melee_combo_tests.rs, data/melee_moves.json, tools/blender/characters/poses_melee_v2.py, tasks/A13a-melee-combos.md, game/src/lib.rs, game/tests/all/main.rs, rotland/src/combat.rs, rotland/src/app/frame_actors.rs, tools/blender/characters/poses.py
Read: game/src/melee.rs, game/src/vault.rs, tools/blender/characters/poses_combat.py, humanoid/src/swing.rs
API: game/src/weapons.rs, game/src/player.rs
Data: data/weapons.json

Rotland is a low-poly 3D zombie survival game in Rust (custom engine, glam math). This is job 13a of a deep character animation system (job 13b, shove and block, comes next and builds on this one). Copy the config style of `game/src/vault.rs` (serde camelCase config with `Default`, `from_json`, `from_repo` via `paths::repo_path`).

**The problem:** all 11 melee weapons (knife, dagger, machete, pipe, katana, bat, crowbar, axe, sledgehammer `hammer`, fire axe `fire_axe`, spear) play one of only two swings (`melee_1h` or `melee_2h`), the same swing every click. The clip also plays at speed 1.0, so the hit lands long before the clip's impact frame (bat: hit at 0.15 s, the bat visually connects at 0.40 s). Heavy weapons feel the same as knives.

**After this job (owner's choices, DECISIONS 2026-11-24):**
- **Combos.** Clicking again soon after a swing plays the **next swing of a 2-3 step chain** for that weapon's class (e.g. right-to-left, left-to-right, overhead). The **last step hits harder** (`damageScale`). If the player stops clicking for longer than `comboWindow` after a swing's impact, the chain resets to step 1.
- **Input buffer.** A click up to `bufferSeconds` before the current swing can start again is remembered and starts the next step the moment it can.
- **Weight.** Heavy weapons wind up much slower (`windupScale` > 1), push harder (`knockbackScale`) and **leave the player stuck for `missRecover` seconds after a swing that hits nothing**. Knives are fast. **Base damage numbers in `weapons.json` do not change.**
- **The clip lines up with the hit:** play speed = `impact / effective windup`, clamped to `minClipSpeed..maxClipSpeed`, so the impact frame lands on the hit (this is what Godot did: MeleeHandler.cs "the clip is sped up so its impact frame lands on the windup").
- A weapon not listed in `melee_moves.json` behaves exactly as today (its own `anim.attack` clip, speed 1.0, no scaling, no combo). Gun bash is unchanged.

## `data/melee_moves.json` (use exactly)
```json
{
  "version": 1,
  "bufferSeconds": 0.25,
  "minClipSpeed": 0.6,
  "maxClipSpeed": 2.2,
  "weapons": {
    "knife": "light", "dagger": "light",
    "machete": "oneHand", "pipe": "oneHand", "katana": "oneHand",
    "bat": "twoHand", "crowbar": "twoHand", "axe": "twoHand",
    "hammer": "heavy", "fire_axe": "heavy",
    "spear": "spear"
  },
  "classes": {
    "light": { "comboWindow": 0.45, "missRecover": 0.0, "steps": [
      { "clip": "stab_a",     "impact": 0.12, "windupScale": 1.0, "cooldownScale": 0.8,  "damageScale": 1.0,  "knockbackScale": 1.0, "arcScale": 1.0 },
      { "clip": "stab_b",     "impact": 0.12, "windupScale": 1.0, "cooldownScale": 0.8,  "damageScale": 1.0,  "knockbackScale": 1.0, "arcScale": 1.0 },
      { "clip": "melee_1h",   "impact": 0.30, "windupScale": 1.3, "cooldownScale": 1.0,  "damageScale": 1.25, "knockbackScale": 1.5, "arcScale": 1.3 } ] },
    "oneHand": { "comboWindow": 0.5, "missRecover": 0.1, "steps": [
      { "clip": "melee_1h",   "impact": 0.30, "windupScale": 1.0, "cooldownScale": 0.85, "damageScale": 1.0,  "knockbackScale": 1.0, "arcScale": 1.0 },
      { "clip": "melee_1h_b", "impact": 0.25, "windupScale": 1.0, "cooldownScale": 0.85, "damageScale": 1.0,  "knockbackScale": 1.0, "arcScale": 1.0 },
      { "clip": "melee_1h_c", "impact": 0.35, "windupScale": 1.4, "cooldownScale": 1.0,  "damageScale": 1.25, "knockbackScale": 1.4, "arcScale": 0.8 } ] },
    "twoHand": { "comboWindow": 0.55, "missRecover": 0.2, "steps": [
      { "clip": "melee_2h",   "impact": 0.40, "windupScale": 1.2, "cooldownScale": 0.9,  "damageScale": 1.0,  "knockbackScale": 1.0, "arcScale": 1.0 },
      { "clip": "melee_2h_b", "impact": 0.35, "windupScale": 1.2, "cooldownScale": 0.9,  "damageScale": 1.0,  "knockbackScale": 1.0, "arcScale": 1.0 },
      { "clip": "melee_2h_c", "impact": 0.45, "windupScale": 1.6, "cooldownScale": 1.1,  "damageScale": 1.3,  "knockbackScale": 1.5, "arcScale": 0.7 } ] },
    "heavy": { "comboWindow": 0.6, "missRecover": 0.45, "steps": [
      { "clip": "heavy_overhead", "impact": 0.50, "windupScale": 1.6, "cooldownScale": 1.0, "damageScale": 1.0, "knockbackScale": 1.4, "arcScale": 0.8 },
      { "clip": "melee_2h_b",     "impact": 0.35, "windupScale": 1.4, "cooldownScale": 1.0, "damageScale": 1.3, "knockbackScale": 1.6, "arcScale": 1.2 } ] },
    "spear": { "comboWindow": 0.5, "missRecover": 0.15, "steps": [
      { "clip": "spear_thrust", "impact": 0.22, "windupScale": 1.0, "cooldownScale": 0.85, "damageScale": 1.0, "knockbackScale": 1.0, "arcScale": 1.0 },
      { "clip": "spear_thrust", "impact": 0.22, "windupScale": 1.0, "cooldownScale": 0.85, "damageScale": 1.0, "knockbackScale": 1.0, "arcScale": 1.0 },
      { "clip": "melee_2h_b",   "impact": 0.35, "windupScale": 1.3, "cooldownScale": 1.0,  "damageScale": 1.2, "knockbackScale": 1.5, "arcScale": 2.0 } ] }
  }
}
```
`melee_1h` (impact 0.30) and `melee_2h` (impact 0.40, the procedural swing in `humanoid/src/swing.rs`) already exist. The `impact` of each new clip must match the clip table you write below.

## Logic: `game/src/melee_combo.rs`
```rust
pub struct MeleeMovesConfig { ... }   // serde camelCase. Default = no weapons, no classes (every weapon falls back). from_json, from_repo ("data/melee_moves.json")
pub struct ComboStep { ... }           // one entry of "steps"

/// The swing to start now: a scaled copy of the weapon, the clip and its play speed.
#[derive(Clone, Debug)]
pub struct ComboMove {
    pub weapon: WeaponDef,  // damage, knockback, arc_deg, windup, cooldown scaled by the step (stamina_cost, range unchanged)
    pub clip: String,
    pub clip_speed: f32,    // impact / scaled windup, clamped; 1.0 for a fallback weapon
    pub step: usize,        // 0-based step that was used
}

#[derive(Clone, Debug, Default)]
pub struct MeleeComboState { /* next step, combo window left, buffered click left, miss recover left, whether the last started swing used a class */ }
impl MeleeComboState {
    pub fn tick(&mut self, dt: f32);
    /// True when a swing should start this frame: `pressed` now (or a click buffered within bufferSeconds),
    /// `swing_idle` is true, and no miss recovery is running. A press that cannot start now is buffered.
    pub fn wants_swing(&mut self, pressed: bool, swing_idle: bool, cfg: &MeleeMovesConfig) -> bool;
    /// The move for the next step (the chain resets to step 0 if the window ran out). Fallback for unknown weapons.
    pub fn next_move(&self, weapon: &WeaponDef, cfg: &MeleeMovesConfig) -> ComboMove;
    /// The swing really started (stamina paid): consume the buffer, advance the step (wrap to 0 after the last).
    pub fn on_started(&mut self, mv: &ComboMove, cfg: &MeleeMovesConfig);
    /// The swing's hit moment passed: open the combo window; if it hit nothing, start missRecover.
    pub fn on_resolved(&mut self, hit_something: bool, weapon_id: &str, cfg: &MeleeMovesConfig);
    pub fn reset(&mut self);   // weapon switch, death
}
```
Rules: after the window runs out the next swing is step 0. After the last step the chain goes back to step 0 (the window still applies). The window counts from `on_resolved`, not from the click. Allocation only when a swing starts (cloning the weapon), never per frame.

## Clips: `tools/blender/characters/poses_melee_v2.py`
`CLIPS = {...}` in the format of `poses_combat.py` (bones: spine, chest, neck, head, upper_arm_R/L, forearm_R/L, hand_R, (pitch, yaw, roll) degrees; same signs as `melee_1h`/`gun_bash`). Use the compact one-line-per-frame style of `gun_bash`. Upper body only. Each starts and ends at the `melee_1h` frame 0.0 rest pose (spear and two-hand clips may start/end at the `gun_bash` hold instead, both hands forward). Make each swing readable from behind the player: a clear windup, a fast strike around `impact`, then a slower recover.
- `stab_a` 0.35 s, impact 0.12: short right-hand jab straight forward at chest height, chest twists a little left.
- `stab_b` 0.35 s, impact 0.12: second jab, slightly lower, chest twists right.
- `melee_1h_b` 0.5 s, impact 0.25: backhand: wind up across the body to the left (spine/chest yaw +), cut out to the right (yaw −).
- `melee_1h_c` 0.6 s, impact 0.35: overhead chop: right arm high (upper_arm_R ≈ 150-160), chop down, spine bends forward (−10 to −15) at impact.
- `melee_2h_b` 0.7 s, impact 0.35: two-hand return swing, the mirror of `melee_2h` (wind up to the left, yaw ≈ +24/+30, strike to the right, yaw ≈ −28/−34), both arms.
- `melee_2h_c` 0.8 s, impact 0.45: two-hand overhead slam: both arms up (≈ 150), lean back a little, then down hard, spine −20 at impact.
- `heavy_overhead` 0.9 s, impact 0.50: like `melee_2h_c` but heavier: longer windup (both arms ≈ 165, spine +10, chest +12 lean back by 0.35 s), slam to spine −25, slow recover.
- `spear_thrust` 0.55 s, impact 0.22: both hands on the shaft: pull back (forearms bent ≈ 110), thrust forward (forearms ≈ 10, upper arms ≈ 85), chest leans forward −8.

## Wiring (fixed, repeat in NOTES unchanged; the helper applies them literally)
1. `game/src/lib.rs`: `pub mod melee_combo;` and `pub use melee_combo::{ComboMove, MeleeComboState, MeleeMovesConfig};` (next to the `vault` lines).
2. `game/tests/all/main.rs`: `mod melee_combo_tests;`.
3. `tools/blender/characters/poses.py`: after `from poses_wound import CLIPS as _WOUND` add `from poses_melee_v2 import CLIPS as _MELEE_V2  # A13a: combo swings`; find `**_HURT, **_WOUND` replace `**_HURT, **_WOUND, **_MELEE_V2`.
4. `rotland/src/combat.rs`:
   - In `pub struct Combat`, after `swing: MeleeSwing,` add `combo: game::MeleeComboState,`; in `Combat::new` after `swing: MeleeSwing::new(),` add `combo: game::MeleeComboState::default(),`.
   - In `pub struct CombatEvents`, after the `swing_started` field add `/// Play speed for the swing clip (0 = use 1.0).\n    pub swing_speed: f32,`.
   - In `set_weapon` and in `kill`, after `self.swing.cancel();` add `self.combo.reset();`.
   - Add above `impl Combat`:
```rust
fn melee_moves() -> &'static game::MeleeMovesConfig {
    static CFG: std::sync::OnceLock<game::MeleeMovesConfig> = std::sync::OnceLock::new();
    CFG.get_or_init(|| game::MeleeMovesConfig::from_repo().unwrap_or_default())
}
```
   - Find:
```rust
        if let Some(weapon) = &self.weapon
            && !weapon.is_bow()
            && attack_pressed
            && self.swing.try_start(weapon, &mut player.stamina)
        {
            ev.swing_started = weapon.anim.attack.clone();
        }
```
replace:
```rust
        let moves = melee_moves();
        self.combo.tick(dt);
        if let Some(weapon) = &self.weapon
            && !weapon.is_bow()
            && self.combo.wants_swing(attack_pressed, self.swing.state() == SwingState::Idle, moves)
        {
            let mv = self.combo.next_move(weapon, moves);
            if self.swing.try_start(&mv.weapon, &mut player.stamina) {
                self.combo.on_started(&mv, moves);
                ev.swing_started = Some(mv.clip);
                ev.swing_speed = mv.clip_speed;
            } else {
                self.combo.reset();
            }
        }
```
   - Find:
```rust
            if let Some(noise) = self.swing.take_noise() {
                self.noises.push(noise);
```
replace:
```rust
            if let Some(noise) = self.swing.take_noise() {
                self.noises.push(noise);
                if self.bash_weapon.is_none()
                    && let Some(w) = self.weapon.as_ref()
                {
                    self.combo.on_resolved(self.hits > 0, &w.id, moves);
                }
```
   - In test `dagger_and_spear_swing_as_melee`: find `assert_eq!(ev1.swing_started.as_deref(), dagger.anim.attack.as_deref());` replace `assert!(ev1.swing_started.is_some());`; find `assert_eq!(ev2.swing_started.as_deref(), spear.anim.attack.as_deref());` replace `assert!(ev2.swing_started.is_some());`.
5. `rotland/src/app/frame_actors.rs`, find:
```rust
            if let (Some(clip), Some(pv)) = (ev.swing_started, actors.player_visual.as_mut()) {
                pv.animator.play_action(&clip, 1.0);
            }
```
replace:
```rust
            let swing_speed = if ev.swing_speed > 0.0 { ev.swing_speed } else { 1.0 };
            if let (Some(clip), Some(pv)) = (ev.swing_started, actors.player_visual.as_mut()) {
                pv.animator.play_action(&clip, swing_speed);
            }
```
(If the borrow checker complains, the helper reorders.)
6. Helper after applying: rebuild the player model (`blender -b --python tools/blender/characters/build_characters.py`) and bake (`cargo run --release -p baker`) under the heavy lock, then the gate.

## Output rules (important)
- Output **complete files only for**: `game/src/melee_combo.rs`, `game/tests/all/melee_combo_tests.rs`, `data/melee_moves.json`, `tools/blender/characters/poses_melee_v2.py`.
- **Do not output** other files. In `=== NOTES ===`, repeat the fixed wiring above.
- **Keep the whole answer under 30,000 characters.** `melee_combo.rs` under 230 lines, tests under 180 lines. Finish with the final line `END OF EDITS`.
- No new crates. No sounds. No allocation per frame (only when a swing starts).
- Every new number in Rust code needs a source tag (`// data: melee_moves.json key` or `// new: DECISIONS 2026-11-24`).
- Answer format: each full file as `=== FILE: path ===`, a code fence, `=== END ===`. Then `=== NOTES ===` with the edits, then `=== END ===`, then `END OF EDITS`.

## Tests: `game/tests/all/melee_combo_tests.rs`
Build `WeaponDef`s in the test (no file reads except one test that loads `data/melee_moves.json` via `MeleeMovesConfig::from_repo()` and checks every class has 2-3 steps and every weapon maps to a known class). Use a small inline JSON config for the rest. Check: unknown weapon → its own clip, speed 1.0, unscaled; first click → step 0; second click inside the window → step 1; third → step 2 with scaled damage; after the last step → step 0; window runs out → step 0; a click while not idle is buffered and starts once idle (inside bufferSeconds), but not after the buffer runs out; a miss on a class with missRecover blocks `wants_swing` until it runs out, a hit does not; clip speed = impact / (windup × windupScale) and is clamped to both ends; `reset` → step 0 and no buffer.

## Worker notes

Hand-applied (Haiku, 2026-11-24): wiring from the task file applied literally. No code changes by hand.
Gate: clippy PASS, tests PASS (998). Scope FAIL only because the shared checkout holds the owner's uncommitted files (assets/models/_previews, settlement.glb, vehicles, car_parts, DECISIONS.md, A12/A14 tasks) and the A13b files; not touched.
Known risk: Combat::take_bites (death by bite) does not call combo.reset(); the wiring only covers set_weapon and kill. Combo state can carry over after a bite death until the next swing resets it.

## Neon notes

Fixed wiring to apply:

1. `game/src/lib.rs`: `pub mod melee_combo;` and `pub use melee_combo::{ComboMove, MeleeComboState, MeleeMovesConfig};` (next to the `vault` lines).
2. `game/tests/all/main.rs`: `mod melee_combo_tests;`.
3. `tools/blender/characters/poses.py`: after `from poses_wound import CLIPS as _WOUND` add `from poses_melee_v2 import CLIPS as _MELEE_V2  # A13a: combo swings`; find `**_HURT, **_WOUND` replace `**_HURT, **_WOUND, **_MELEE_V2`.
4. `rotland/src/combat.rs`:

* In `pub struct Combat`, after `swing: MeleeSwing,` add `combo: game::MeleeComboState,`; in `Combat::new` after `swing: MeleeSwing::new(),` add `combo: game::MeleeComboState::default(),`.
* In `pub struct CombatEvents`, after the `swing_started` field add `/// Play speed for the swing clip (0 = use 1.0).\n    pub swing_speed: f32,`.
* In `set_weapon` and in `kill`, after `self.swing.cancel();` add `self.combo.reset();`.
* Add above `impl Combat`:

```rust
fn melee_moves() -> &'static game::MeleeMovesConfig {
    static CFG: std::sync::OnceLock<game::MeleeMovesConfig> = std::sync::OnceLock::new();
    CFG.get_or_init(|| game::MeleeMovesConfig::from_repo().unwrap_or_default())
}

```

* Find:

```rust
        if let Some(weapon) = &self.weapon
            && !weapon.is_bow()
            && attack_pressed
            && self.swing.try_start(weapon, &mut player.stamina)
        {
            ev.swing_started = weapon.anim.attack.clone();
        }

```

replace:

```rust
        let moves = melee_moves();
        self.combo.tick(dt);
        if let Some(weapon) = &self.weapon
            && !weapon.is_bow()
            && self.combo.wants_swing(attack_pressed, self.swing.state() == SwingState::Idle, moves)
        {
            let mv = self.combo.next_move(weapon, moves);
            if self.swing.try_start(&mv.weapon, &mut player.stamina) {
                self.combo.on_started(&mv, moves);
                ev.swing_started = Some(mv.clip);
                ev.swing_speed = mv.clip_speed;
            } else {
                self.combo.reset();
            }
        }

```

* Find:

```rust
            if let Some(noise) = self.swing.take_noise() {
                self.noises.push(noise);

```

replace:

```rust
            if let Some(noise) = self.swing.take_noise() {
                self.noises.push(noise);
                if self.bash_weapon.is_none()
                    && let Some(w) = self.weapon.as_ref()
                {
                    self.combo.on_resolved(self.hits > 0, &w.id, moves);
                }

```

* In test `dagger_and_spear_swing_as_melee`: find `assert_eq!(ev1.swing_started.as_deref(), dagger.anim.attack.as_deref());` replace `assert!(ev1.swing_started.is_some());`; find `assert_eq!(ev2.swing_started.as_deref(), spear.anim.attack.as_deref());` replace `assert!(ev2.swing_started.is_some());`.

5. `rotland/src/app/frame_actors.rs`, find:

```rust
            if let (Some(clip), Some(pv)) = (ev.swing_started, actors.player_visual.as_mut()) {
                pv.animator.play_action(&clip, 1.0);
            }

```

replace:

```rust
            let swing_speed = if ev.swing_speed > 0.0 { ev.swing_speed } else { 1.0 };
            if let (Some(clip), Some(pv)) = (ev.swing_started, actors.player_visual.as_mut()) {
                pv.animator.play_action(&clip, swing_speed);
            }

```

6. Helper after applying: rebuild the player model (`blender -b --python tools/blender/characters/build_characters.py`) and bake (`cargo run --release -p baker`) under the heavy lock, then the gate.

## Gate report
- date: 2026-11-24
- base: main
- files changed (38): assets/models/buildings/settlement.glb, assets/models/characters/human_player.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/melee_guard.json, data/melee_moves.json, docs/DECISIONS.md, game/src/lib.rs, game/src/melee_combo.rs, game/src/melee_guard.rs, game/src/zombie/brain.rs, game/tests/all/main.rs, game/tests/all/melee_combo_tests.rs, game/tests/all/melee_guard_tests.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_sim.rs, rotland/src/app/state.rs, rotland/src/combat.rs, rotland/src/zombies/combat_glue.rs, tasks/A12-actions.md, tasks/A13a-melee-combos.md, tasks/A13b-shove-block.md, tasks/A14-hit-reactions.md, tools/blender/buildings/settlement.py, tools/blender/characters/poses_guard.py, tools/blender/characters/poses_melee_v2.py, tools/blender/characters/poses.py, tools/blender/vehicles/car_parts.py
- tests passed: 998
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
