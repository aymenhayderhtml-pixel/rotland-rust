# R9-02: Gun bash when out of ammo

Read `tasks/R9-common.md` first. Reference: `scripts/Combat/GunBash.cs`, `scripts/Combat/GunHandler.cs`. Rust already has `game::melee::gun_bash(&WeaponDef) -> WeaponDef` and the shooting code in `rust/rotland/src/shooting.rs` (read its events; a `bash` event may already exist).

You own `rust/rotland/src/shooting.rs` and `rust/rotland/src/combat.rs` this round.
- When the gun's magazine and spare ammo are both empty, a left click swings the gun as a club, exactly as GunBash.cs: its bash damage, impact time, clip, sound, cooldown.
- Add to `Combat` a method `bash(&mut self, gun: &WeaponDef, stamina..)` (or reuse the swing with `gun_bash(gun)`) so the swing resolves through the same `MeleeWorld` path as melee (zombies and animals are hit).
- Say in your notes the one call Claude adds in main.rs (where shooting events are handled).
Tests (`#[cfg(test)]` in shooting.rs or combat.rs, existing test style there): empty pistol + click starts a bash with the bash numbers from data/weapons.json; a gun with ammo fires instead; a zombie in front takes the bash damage.

## Worker notes

### Files changed
- `rust/rotland/src/combat.rs`:
  - Added `bash_weapon: Option<WeaponDef>` and `pending_swing_started: Option<String>` to `Combat`.
  - Added `pub fn bash(&mut self, gun: &WeaponDef, stamina: &mut Stamina) -> bool`: builds a melee `WeaponDef` via `game::melee::gun_bash(gun)`, checks if alive and is a gun, calls `self.swing.try_start(&bash_def, stamina)` to deduct stamina, sets `self.bash_weapon` and queues the attack animation clip (`bash_def.anim.attack`).
  - Added `pub fn is_bashing(&self) -> bool` and `pub fn swing_state(&self) -> SwingState` and `pub fn cancel_swing(&mut self)`.
  - Updated `Combat::update`: if `ev.swing_started` is None, consumes `self.pending_swing_started.take()`; uses `self.weapon.as_ref().or(self.bash_weapon.as_ref())` so the bash resolves through the identical `MeleeWorld` path as melee weapons (hitting zombies/animals, dealing damage, knockback, stagger, and emitting noise); resets `self.bash_weapon` to `None` once `self.swing.state()` returns to `Idle`.
  - Updated `Combat::weapon_id()`: returns the bash weapon ID when bashing so swing/hit audio lookups resolve.
  - Updated `Combat::set_weapon()`, `kill()`, and `take_bites()` to cancel any active bash.
  - Added 4 unit tests in `combat.rs`:
    - `empty_pistol_click_starts_bash_with_weapons_json_numbers`
    - `zombie_in_front_takes_bash_damage`
    - `ak47_bash_deals_30_damage_matching_godot_gun_bash_when_empty`
    - `bash_fails_when_insufficient_stamina_or_dead`
- `rust/rotland/src/shooting.rs`:
  - Confirmed `ShootEvents.bash: bool` triggers when gun magazine is 0 and spare ammo in inventory is 0 on fire input (retaining auto-reload dry fire when spare ammo > 0).
  - Added unit test in `shooting.rs`:
    - `gun_with_ammo_fires_instead_of_bashing`

### Public API
- `Combat::bash(&mut self, gun: &WeaponDef, stamina: &mut Stamina) -> bool`: starts a gun bash swing with stamina deduction.
- `Combat::is_bashing(&self) -> bool`: returns true while a bash swing is active (winding up or cooling down).
- `Combat::swing_state(&self) -> SwingState`: returns current swing state (`Idle`, `WindingUp`, or `CoolingDown`).
- `Combat::cancel_swing(&mut self)`: cancels an active melee or bash swing.
- `Combat::bash_weapon: Option<WeaponDef>`: public field for inspectability.

### Traced test numbers and sources
- `data/weapons.json`:
  - `"pistol"`: `bashDamage` = 15.0, `bashImpact` = 0.30, `anim.bash` = `"gun_bash"`, `magazineSize` = 12.
  - `"ak47"`: `bashDamage` = 30.0, `bashImpact` = 0.30, `anim.bash` = `"gun_bash"`, `magazineSize` = 30.
- `scripts/Combat/GunBash.cs` / `rust/game/src/melee.rs:gun_bash`:
  - `BASH_RANGE` = 1.9 m
  - `BASH_ARC_DEG` = 80.0 deg
  - `BASH_COOLDOWN` = 0.7 s
  - `BASH_STAMINA_COST` = 10.0 (initial stamina 100.0 - 10.0 = 90.0)
  - `BASH_KNOCKBACK` = 2.0 (knockback vector = (0, 0, -2.0) along (0, 0, -1))
  - `BASH_STAGGER` = 0.3 s
  - `BASH_MAX_TARGETS` = 1
  - `BASH_NOISE_RADIUS` = 6.0 m
- `scripts/Debug/Tests/GunBashWhenEmpty.cs`:
  - Zombie spawned at 1.5 m in front of player with 60.0 health.
  - AK-47 bash impact is 0.30 s. After 0.35 s (> 0.30 s), 1 hit lands.
  - Zombie takes 30.0 damage: remaining health is 60.0 - 30.0 = 30.0.
- `scripts/Debug/Tests/NoBashWithAmmo.cs` / `shooting.rs:gun_with_ammo_fires_instead_of_bashing`:
  - Pistol with 12 rounds in mag: fire pressed results in `shots_fired` = 1, `bash` = false, `empty_click` = false, ammo = (11, 20).
  - Pistol with 0 in mag and 20 in inventory: `shots_fired` = 0, `bash` = false, `empty_click` = true, `reload_started` = true.
  - Pistol with 0 in mag and 0 in inventory: `shots_fired` = 0, `bash` = true, `empty_click` = false, `reload_started` = false, ammo = (0, 0).

### Quoted loop behavior (Rule 2)
In `rust/rotland/src/main.rs`:
Line 1100:
`if let (Some(play), Some(p)) = (self.play.as_mut(), self.player_ctrl.as_ref())`
and lines 1115-1125:
```rust
if let Some(z) = self.zombies.as_mut()
    && let Some(ev) = play.shoot_tick(
        dt,
        shoot,
        self.camera.position,
        self.camera.forward(),
        &mut wildlife::Prey { zombies: z, animals: self.wildlife.as_mut().map(|w| &mut w.sim), animal_bonus },
        &self.physics,
    )
    && let Some(c) = self.combat.as_mut()
{
```
Currently handles shooting events (`kills`, `shots_fired`, `reload_started`, `empty_click`, `noise`).

### Wiring for Claude
In `rust/rotland/src/main.rs`, where shooting events are handled in the frame loop (around line 1125):
Change `self.player_ctrl.as_ref()` on line 1100 to `self.player_ctrl.as_mut()` (or access `p_ctrl: &mut PlayerController`):
```rust
if let (Some(play), Some(p)) = (self.play.as_mut(), self.player_ctrl.as_mut())
```
Then inside the `if let Some(z) ... && let Some(c) = self.combat.as_mut()` block (around line 1126):
Add the one call:
```rust
if ev.bash && let Some(gun) = &play.gun {
    c.bash(gun, &mut p.stamina);
}
```
`c.bash()` starts the melee swing via `gun_bash(gun)` and queues the clip `gun_bash`. On the next frame, `c.update` emits `ev.swing_started = Some("gun_bash")` which plays the attack animation via `self.player_visual.animator.play_action`, resolves hits against `Prey` (zombies & animals), applies bash damage, knockback (2.0) and emits bash noise (radius 6.0).

