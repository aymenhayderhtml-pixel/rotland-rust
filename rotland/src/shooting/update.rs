//! Per-frame shooting logic: cooldowns, reload, ammo use, jams, spread and hit resolution.

use game::guns::Gun;
use game::guns_fire::{fire_rays, resolve_hits, shot_noise};
use game::items::Inventory;
use game::WeaponDef;
use glam::Vec3;
use physics::PhysicsWorld;

use super::events::{ShootEvents, ShootInput};
use super::targets::TargetWorld;
use super::Shooting;

impl Shooting {
    /// One frame of shooting logic against any `TargetWorld` (enables pure tests without GPU).
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub fn update_targets<T: TargetWorld>(
        &mut self,
        dt: f32,
        input: ShootInput,
        weapon: &WeaponDef,
        eye: Vec3,
        aim_dir: Vec3,
        inventory: &mut Inventory,
        targets: &mut T,
        physics: &PhysicsWorld,
    ) -> ShootEvents {
        self.update_targets_with_wear(
            dt,
            input,
            weapon,
            eye,
            aim_dir,
            inventory,
            targets,
            physics,
            1.0,
            &game::WearConfig::default(),
        )
    }

    /// One frame of shooting logic with weapon condition wear, jams, broken gating, and spread scaling.
    #[allow(clippy::too_many_arguments)]
    pub fn update_targets_with_wear<T: TargetWorld>(
        &mut self,
        dt: f32,
        input: ShootInput,
        weapon: &WeaponDef,
        eye: Vec3,
        aim_dir: Vec3,
        inventory: &mut Inventory,
        targets: &mut T,
        physics: &PhysicsWorld,
        condition: f32,
        wear_cfg: &game::WearConfig,
    ) -> ShootEvents {
        if weapon.is_bow() {
            let spare_in_bag = inventory.count(&weapon.ammo_item);
            return ShootEvents {
                ammo: (spare_in_bag, u32::MAX),
                ..Default::default()
            };
        }

        if weapon.is_throwable() {
            let count = inventory.count(&weapon.id);
            return ShootEvents {
                ammo: (count, u32::MAX),
                ..Default::default()
            };
        }

        if !weapon.is_gun() {
            return ShootEvents::default();
        }

        // Switching weapons cancels any reload in progress on the previous weapon
        if let Some(prev_id) = &self.active_weapon
            && prev_id != &weapon.id
            && let Some(prev_gun) = self.guns.get_mut(prev_id)
        {
            prev_gun.cancel_reload();
        }
        self.active_weapon = Some(weapon.id.clone());

        let gun = self
            .guns
            .entry(weapon.id.clone())
            .or_insert_with(|| Gun::new(weapon.magazine_size));
        let cd = self.shot_cooldowns.entry(weapon.id.clone()).or_insert(0.0);
        *cd = (*cd - dt).max(0.0);

        // Advance cooldown and reload timer
        if let Some(done) = gun.update(dt)
            && done.rounds_taken > 0
        {
            let available = inventory.count(&weapon.ammo_item);
            let to_take = done.rounds_taken.min(available);
            if to_take > 0 {
                inventory.remove(&weapon.ammo_item, to_take);
            }
        }

        let mut events = ShootEvents::default();
        let spare_in_bag = inventory.count(&weapon.ammo_item);

        // 1. Manual reload requested
        if input.reload_pressed && gun.start_reload(weapon, spare_in_bag) {
            events.reload_started = true;
        }

        // 2. Firing request: auto weapons fire while held, semi only on trigger press
        let wants_fire = if weapon.auto {
            input.fire_pressed || input.fire_held
        } else {
            input.fire_pressed
        };

        if wants_fire && !gun.is_reloading() {
            if game::is_broken(condition) {
                events.broken = true;
                if input.fire_pressed {
                    events.empty_click = true;
                }
            } else if gun.rounds() == 0 {
                if spare_in_bag == 0 {
                    // Empty with no spares: swung like a club instead of clicking (GunHandler.cs line 263)
                    events.bash = true;
                } else {
                    // Dry fire click, automatically starts reload if ammo is in the bag (GunHandler.cs line 144)
                    events.empty_click = true;
                    if gun.start_reload(weapon, spare_in_bag) {
                        events.reload_started = true;
                    }
                }
            } else if *cd <= 0.0 && game::jam_roll(condition, wear_cfg, || self.rng.next_f32()) {
                *cd = weapon.fire_interval;
                events.jammed = true;
                events.empty_click = true;
            } else if gun.try_fire(weapon, input.fire_pressed, input.fire_held) {
                *cd = weapon.fire_interval;
                let mut shot_weapon = weapon.clone();
                let sm = game::spread_mult(condition, wear_cfg);
                shot_weapon.spread_hip_deg *= sm;
                shot_weapon.spread_aim_deg *= sm;

                events.shots_fired += 1;
                events.fired_weapon = Some(shot_weapon.id.clone());
                events.noise = Some(shot_noise(&shot_weapon, eye));

                let rays = fire_rays(&shot_weapon, eye, aim_dir, input.aiming, &mut self.rng);
                let target_list = targets.gun_targets();
                let hits = resolve_hits(&rays, &shot_weapon, &target_list, physics, shot_weapon.range);

                for hit in hits {
                    if let (Some(target_id), Some(damage)) = (hit.target, hit.damage) {
                        events.hits += 1;
                        let knockback = Vec3::new(damage.direction.x, 0.0, damage.direction.z)
                            .normalize_or_zero()
                            * shot_weapon.knockback;
                        if targets.hit(target_id, damage, knockback) {
                            events.kills += 1;
                        }
                    }
                }
            }
        }

        let final_spare = inventory.count(&weapon.ammo_item);
        events.ammo = (gun.rounds(), final_spare);

        events
    }
}
