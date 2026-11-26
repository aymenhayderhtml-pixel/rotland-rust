# R36-03: revolver, hunting rifle, crossbow, grenade (data + logic + wiring)

Owns: data/weapons.json, data/items.json, data/loot_tables.json, game_core/src/weapons.rs, game_world/src/grenade.rs, game/tests/all/grenade.rs, game/tests/all/new_weapons.rs, rotland/src/grenade_play.rs, rotland/src/bow.rs, rotland/src/projectiles_view.rs, rotland/src/app/frame_actors*.rs (only the throwable/grenade lines)
Rules: `tasks/R36-common.md`. Owner decision: `docs/DECISIONS.md` row 2026-11-25 (weapons).

Copy the JSON shape of the existing entries exactly (`pistol`, `rifle`, `bow`, and the throwable `spear`). Starting numbers:
- **revolver** (copy pistol): damage 55, headshotMultiplier 2.0, range 50, fireInterval 0.45, magazineSize 6, ammoItem `revolver_ammo`, reloadTime 2.2, spreadHipDeg 2.5, spreadAimDeg 0.5, noiseRadius 45, knockback 1.0, stagger 0.3. Uses the pistol anims and pistol hold (tune `hold`/`fpHold` so it sits in the hand like the pistol).
- **hunting_rifle** (copy rifle): damage 110, headshotMultiplier 3.0, range 150, fireInterval 1.5, magazineSize 4, ammoItem `rifle_rounds`, reloadTime 2.8, spreadHipDeg 4.0, spreadAimDeg 0.05, scope fov 18 sensitivity 0.3, noiseRadius 70, knockback 2.0, stagger 0.5.
- **crossbow** (copy bow, and use the bow/arrow projectile code in `rotland/src/bow.rs`): damage 80, headshotMultiplier 2.5, range 70, arrowSpeed 60, gravity 6.0, pickupChance 0.75, ammoItem `crossbow_bolt`, noiseRadius 3. It is loaded instead of drawn: one bolt per shot, then a 2.0 s reload that plays `crossbow_reload` (anim key `reload`). It can stay loaded while carried, so firing is instant when loaded. Projectile model `crossbow_bolt`.
- **grenade** (throwable item, stack of up to 5, 1 used per throw): hold left mouse to aim with an arc preview (reuse how the spear/throwable aims, if there is an arc; otherwise a simple dotted arc), release to throw with the `grenade_throw` clip. Throw speed 14, gravity 9.8, bounces once on the ground (keeps 30% speed), fuse 2.5 s from the throw. **Explosion:** radius 6 m. Damage is 150 at the centre, down to 20% at the edge, to zombies AND the player (owner decision: your own grenade hurts you). Knockback 8 at the centre. Noise radius 80 (wakes zombies). Line of sight: a wall between them blocks the damage (use a physics ray). On explode, call `ExplosionView::spawn(pos, 6.0)` (R36-05 makes it).
- Pure grenade logic goes in `game_world/src/grenade.rs`: flight, bounce, fuse, damage falloff, and who is hit (given positions and a "blocked" callback). Tests in `game/tests/all/grenade.rs`. Data loading tests for the 4 weapons go in `game/tests/all/new_weapons.rs`.
- **Items:** `revolver_ammo` (stack 36), `crossbow_bolt` (stack 20), `grenade` (stack 5), plus the 3 weapons, with names and short descriptions like the others.
- **Loot:** revolver + revolver_ammo in house/police tables (uncommon). hunting_rifle in cabin/hunting tables (rare). crossbow + bolts in cabin/barn/shed (rare). grenade only in `police` and military-type tables (rare, count 1-2). Use the real table names in `data/loot_tables.json`.
- The HUD ammo counter must work for all four (grenade shows how many you carry).

## Worker notes
### Files changed (rotland wiring)
- `rotland/src/bow.rs`: added `CrossbowStep`, `crossbow_loaded` (starts `true`, preserved across `cancel()`), `crossbow_reload_timer`, `update_crossbow`, `is_crossbow_reloading`, plus unit tests.
- `rotland/src/projectiles_view.rs`: added `ProjectileKind::CrossbowBolt => "crossbow_bolt"` in `projectile_model_name`.
- `rotland/src/grenade_play.rs`: new module with `GrenadePlay` (left-mouse hold arc preview via `compute_arc_points`, release to throw consuming 1 `grenade` and playing `grenade_throw`, `step_grenade`, `compute_explosion_hits` + physics raycast LOS for zombies and player self-damage via `compute_explosion_damage`, noise radius 80, `fill_render_instances` for flying grenades and arc dots).
- `rotland/src/app/frame_actors.rs`: suppresses melee attack while holding `grenade`, passes `reload_pressed` to `step_ranged_and_projectiles`, and steps `actors.grenades`.
- `rotland/src/app/frame_actors_combat.rs`: excludes `grenade` from spear's instant right-click throw, wires `crossbow` instant fire + 2.0 s `crossbow_reload` + HUD ammo `(loaded, spare_bolts)`.

### Exact lines needed in shared files (for lead)
1. `rotland/src/main.rs`:
   Add `mod grenade_play;` in alphabetical order (after `mod fp_weapon;`).
2. `rotland/src/app/state.rs` (in `ActorsState`):
   Add `pub grenades: crate::grenade_play::GrenadePlay,`
3. `rotland/src/app/render_frame.rs` (around line 153, where `beds` collects dynamic props like `toxic_cloud_view`):
   ```rust
   if let Some(pack) = render.pack.as_ref() {
       actors.grenades.fill_render_instances(&gpu.device, pack, &mut render.library, &mut beds);
   }
   ```
4. `rotland/src/app/frame_actors_combat.rs` (line 277) & `rotland/src/grenade_play.rs` (line 169):
   - Wire `crate::gun_feel_view::GunFeelView::on_shot(&mut ..., "crossbow")` at the `NOTE(lead)` comment in `frame_actors_combat.rs`.
   - Wire `crate::explosion_view::ExplosionView::spawn(&mut ..., pos, EXPLOSION_RADIUS)` at the `NOTE(lead)` comment in `grenade_play.rs`.
5. `game_world/src/projectiles/types.rs` & `sim.rs` (for `crossbow_bolt` model + pickup):
   - Add `CrossbowBolt` variant to `ProjectileKind` in `types.rs`, `(0.75, 2.5)` in `Projectile::new`.
   - In `sim.rs`: `ProjectileKind::CrossbowBolt => Some("crossbow_bolt".to_string())` on `Hit::World`, and in `bow_shot`: `kind: if def.id == "crossbow" { ProjectileKind::CrossbowBolt } else { ProjectileKind::Arrow }`.
6. Check `game_world/src/grenade.rs` field names against `grenade_play.rs`: `ActiveGrenade { pos, vel, fuse, bounced }`, `GrenadeConfig { radius, max_damage, min_edge_frac, knockback, .. }`, `GrenadeHit { id, damage, knockback }`.

