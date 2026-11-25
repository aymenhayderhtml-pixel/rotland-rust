//! Gun bash melee attack for empty firearms (GunBash.cs).

use game::melee::gun_bash;
use game::{Stamina, WeaponDef};

use crate::combat::Combat;

impl Combat {
    /// Swings an empty gun as a club when out of ammo (GunBash.cs).
    /// Builds a melee WeaponDef via gun_bash(gun), deducting stamina and resolving
    /// the swing through the same MeleeWorld path as a melee weapon.
    pub fn bash(&mut self, gun: &WeaponDef, stamina: &mut Stamina) -> bool {
        if self.is_dead() || !gun.is_gun() {
            return false;
        }
        let bash_def = gun_bash(gun);
        if !self.swing.try_start(&bash_def, stamina) {
            return false;
        }
        self.pending_swing_started = bash_def.anim.attack.clone();
        self.bash_weapon = Some(bash_def);
        true
    }

    /// True if a gun bash is active (winding up or cooling down).
    #[cfg(test)]
    pub fn is_bashing(&self) -> bool {
        self.bash_weapon.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::test_helpers::{flat_world, load_weapons, MockMeleeWorld, MockZombie};
    use game::melee::SwingState;
    use game::{Health, Player, Stamina};
    use glam::Vec3;

    #[test]
    fn empty_pistol_click_starts_bash_with_weapons_json_numbers() {
        let registry = load_weapons();
        let pistol = registry.get("pistol").expect("pistol weapon def");

        let mut asserts = 0;
        // Verify numbers directly from data/weapons.json
        assert_eq!(pistol.bash_damage, 15.0);
        asserts += 1;
        assert_eq!(pistol.bash_impact, 0.30);
        asserts += 1;
        assert_eq!(pistol.anim.bash.as_deref(), Some("gun_bash"));
        asserts += 1;

        let mut combat = Combat::new("bat");
        // Holding a gun clears melee weapon
        combat.set_weapon(None);
        assert_eq!(combat.weapon_id(), None);
        asserts += 1;

        let mut stamina = Stamina::default();
        let initial_stamina = stamina.current; // 100.0
        assert_eq!(initial_stamina, 100.0);
        asserts += 1;

        // Start gun bash
        let started = combat.bash(pistol, &mut stamina);
        assert!(started);
        asserts += 1;
        assert!(combat.is_bashing());
        asserts += 1;
        assert_eq!(combat.swing_state(), SwingState::WindingUp);
        asserts += 1;
        assert_eq!(combat.weapon_id(), Some("pistol"));
        asserts += 1;

        // Stamina deducted by BASH_STAMINA_COST (10.0)
        assert_eq!(stamina.current, 90.0);
        asserts += 1;
        assert_eq!(stamina.time_since_use, 0.0);
        asserts += 1;

        // Verify bash weapon definition matches GunBash.cs and weapons.json specifications
        let bash_def = combat.bash_weapon.as_ref().expect("bash weapon def");
        assert_eq!(bash_def.id, "pistol");
        asserts += 1;
        assert_eq!(bash_def.damage, 15.0); // pistol.bash_damage
        asserts += 1;
        assert_eq!(bash_def.windup, 0.30); // pistol.bash_impact
        asserts += 1;
        assert_eq!(bash_def.cooldown, 0.7); // BASH_COOLDOWN
        asserts += 1;
        assert_eq!(bash_def.stamina_cost, 10.0); // BASH_STAMINA_COST
        asserts += 1;
        assert_eq!(bash_def.range, 1.9); // BASH_RANGE
        asserts += 1;
        assert_eq!(bash_def.arc_deg, 80.0); // BASH_ARC_DEG
        asserts += 1;
        assert_eq!(bash_def.knockback, 2.0); // BASH_KNOCKBACK
        asserts += 1;
        assert_eq!(bash_def.stagger, 0.3); // BASH_STAGGER
        asserts += 1;
        assert_eq!(bash_def.max_targets, 1);
        asserts += 1;
        assert_eq!(bash_def.noise_radius, 6.0); // BASH_NOISE_RADIUS
        asserts += 1;
        assert_eq!(bash_def.anim.attack.as_deref(), Some("gun_bash"));
        asserts += 1;

        // Cannot start another bash while winding up
        assert!(!combat.bash(pistol, &mut stamina));
        asserts += 1;
        assert!(asserts >= 20);
    }

    #[test]
    fn zombie_in_front_takes_bash_damage() {
        let registry = load_weapons();
        let pistol = registry.get("pistol").expect("pistol weapon def");
        let physics = flat_world();

        let mut combat = Combat::new("bat");
        combat.set_weapon(None);

        let mut player = Player::new(Vec3::ZERO);
        player.yaw = 0.0; // facing Vec3::NEG_Z (forward is (0, 0, -1))

        // Zombie 1.2m ahead at (0, 0, -1.2) with 100.0 health (in range: 1.2 < 1.9 BASH_RANGE)
        let mut world = MockMeleeWorld {
            zombies: vec![MockZombie {
                id: 42,
                position: Vec3::new(0.0, 0.0, -1.2),
                health: Health::new(100.0),
                knockback: Vec3::ZERO,
            }],
        };

        let mut asserts = 0;
        let started = combat.bash(pistol, &mut player.stamina);
        assert!(started);
        asserts += 1;

        // Frame 1: advance 0.15s (< 0.30s bash impact). No hit yet.
        let ev1 = combat.update(0.15, false, &mut player, &mut world, &physics);
        assert_eq!(combat.hits, 0);
        asserts += 1;
        assert_eq!(world.zombies[0].health.current(), 100.0);
        asserts += 1;
        assert_eq!(ev1.swing_started.as_deref(), Some("gun_bash"));
        asserts += 1;
        assert!(ev1.swung.is_none());
        asserts += 1;
        assert_eq!(combat.swing_state(), SwingState::WindingUp);
        asserts += 1;

        // Frame 2: advance 0.16s (total elapsed 0.31s >= 0.30s bash impact). Hit resolves!
        let ev2 = combat.update(0.16, false, &mut player, &mut world, &physics);
        assert_eq!(combat.hits, 1);
        asserts += 1;
        // Zombie takes exactly pistol.bash_damage (15.0). 100.0 - 15.0 = 85.0.
        assert_eq!(world.zombies[0].health.current(), 85.0);
        asserts += 1;
        // Knockback along attack direction (0, 0, -1) * 2.0 knockback
        assert_eq!(world.zombies[0].knockback, Vec3::new(0.0, 0.0, -2.0));
        asserts += 1;
        // Bash noise emitted at player position with radius 6.0
        assert_eq!(combat.noises.len(), 1);
        asserts += 1;
        assert_eq!(combat.noises[0].radius, 6.0);
        asserts += 1;
        assert!(ev2.swung.is_some());
        asserts += 1;
        // State transitions to CoolingDown
        assert_eq!(combat.swing_state(), SwingState::CoolingDown);
        asserts += 1;

        // Frame 3: advance past cooldown (0.7s total cooldown, 0.31s elapsed -> remaining ~0.39s)
        combat.update(0.40, false, &mut player, &mut world, &physics);
        assert_eq!(combat.swing_state(), SwingState::Idle);
        asserts += 1;
        assert!(!combat.is_bashing());
        asserts += 1;
        assert!(asserts >= 12);
    }

    #[test]
    fn ak47_bash_deals_30_damage_matching_godot_gun_bash_when_empty() {
        let registry = load_weapons();
        let ak47 = registry.get("ak47").expect("ak47 weapon def");
        let physics = flat_world();

        let mut combat = Combat::new("bat");
        combat.set_weapon(None);

        let mut player = Player::new(Vec3::ZERO);
        player.yaw = 0.0;

        // Zombie 1.5m in front with 60.0 health (exact setup from GunBashWhenEmpty.cs)
        let mut world = MockMeleeWorld {
            zombies: vec![MockZombie {
                id: 10,
                position: Vec3::new(0.0, 0.0, -1.5),
                health: Health::new(60.0),
                knockback: Vec3::ZERO,
            }],
        };

        let mut asserts = 0;
        assert_eq!(ak47.bash_damage, 30.0);
        asserts += 1;
        assert_eq!(ak47.bash_impact, 0.30);
        asserts += 1;

        assert!(combat.bash(ak47, &mut player.stamina));
        asserts += 1;

        // Advance 0.35s past impact (0.30s)
        let ev = combat.update(0.35, false, &mut player, &mut world, &physics);
        assert_eq!(combat.hits, 1);
        asserts += 1;
        // Exact assertion from GunBashWhenEmpty.cs: 60 - 30 = 30
        assert_eq!(world.zombies[0].health.current(), 30.0);
        asserts += 1;
        assert_eq!(combat.noises[0].radius, 6.0);
        asserts += 1;
        assert!(ev.swung.is_some());
        asserts += 1;
        assert!(asserts >= 7);
    }

    #[test]
    fn bash_fails_when_insufficient_stamina_or_dead() {
        let registry = load_weapons();
        let pistol = registry.get("pistol").expect("pistol weapon def");

        let mut combat = Combat::new("bat");
        combat.set_weapon(None);

        let mut asserts = 0;
        // Stamina less than 10.0 (BASH_STAMINA_COST)
        let mut stamina = Stamina { current: 5.0, ..Default::default() };
        assert!(!combat.bash(pistol, &mut stamina));
        asserts += 1;
        assert_eq!(stamina.current, 5.0); // stamina unchanged
        asserts += 1;
        assert!(!combat.is_bashing());
        asserts += 1;

        // Dead player cannot bash
        combat.kill();
        let mut full_stamina = Stamina::default();
        assert!(!combat.bash(pistol, &mut full_stamina));
        asserts += 1;
        assert_eq!(full_stamina.current, 100.0);
        asserts += 1;
        assert!(asserts >= 5);
    }
}
