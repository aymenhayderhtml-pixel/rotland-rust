//! Combat behaviour for dogs: scanning, barking, biting, and taking hits.
// Godot: DogCombat.cs

use glam::Vec3;
use crate::damage::DamageInfo;
use crate::health::Health;
use super::config::DogConfigData;

/// Represents a bite attack landed by a dog on a target entity.
#[derive(Clone, Debug, PartialEq)]
pub struct DogBite {
    pub target_id: u64,
    pub damage: DamageInfo,
    pub knockback: Vec3,
}

/// The dog's combat logic: scans for zombies/animals, barks at close enemies,
/// closes the distance and delivers bite attacks.
// Godot: DogCombat.cs
#[derive(Clone, Debug)]
pub struct DogCombat {
    pub(super) scan_left: f32,
    pub(super) bark_cooldown: f32,
    pub(super) bite_cooldown: f32,
    pub(super) hurt_cooldown: f32,
    pub(super) target_id: Option<u64>,
    pub(super) target_pos: Option<Vec3>,
    pub(super) engaged: bool,
    pub(super) bark_count: usize,
    pub(super) bite_count: usize,
}

impl Default for DogCombat {
    fn default() -> Self {
        Self::new()
    }
}

impl DogCombat {
    pub fn new() -> Self {
        Self {
            scan_left: 0.0,
            bark_cooldown: 0.0,
            bite_cooldown: 0.0,
            hurt_cooldown: 0.0,
            target_id: None,
            target_pos: None,
            engaged: false,
            bark_count: 0,
            bite_count: 0,
        }
    }

    pub fn target_id(&self) -> Option<u64> {
        self.target_id
    }

    pub fn is_engaged(&self) -> bool {
        self.engaged
    }

    pub fn bark_count(&self) -> usize {
        self.bark_count
    }

    pub fn bite_count(&self) -> usize {
        self.bite_count
    }

    pub fn clear(&mut self) {
        self.target_id = None;
        self.target_pos = None;
        self.engaged = false;
    }

    /// Where the dog should run to fight, or None when it has no target or is already biting.
    // Godot: DogCombat.cs FightPoint
    pub fn fight_point(&mut self, dog_pos: Vec3, cfg: &DogConfigData) -> Option<Vec3> {
        let Some(pos) = self.target_pos else {
            self.engaged = false;
            return None;
        };
        let dist = Self::flat(pos - dog_pos).length();
        self.engaged = dist <= cfg.bite_range;
        if self.engaged {
            None
        } else {
            Some(pos)
        }
    }

    /// Ticks cooldowns and scans for combat targets every scan interval.
    // Godot: DogCombat.cs Step
    pub fn step(
        &mut self,
        dog_id: u64,
        dog_pos: Vec3,
        health: &mut Health,
        dt: f32,
        cfg: &DogConfigData,
        zombies: &[(u64, Vec3, bool)],
        animals: &[(u64, Vec3, bool)],
        bites: &mut Vec<DogBite>,
        sounds: &mut Vec<(String, Vec3)>,
        one_shot: &mut Option<(&'static str, f32)>,
    ) {
        self.bark_cooldown = (self.bark_cooldown - dt).max(0.0);
        self.bite_cooldown = (self.bite_cooldown - dt).max(0.0);
        self.hurt_cooldown = (self.hurt_cooldown - dt).max(0.0);

        self.scan_left -= dt;
        if self.scan_left > 0.0 {
            return;
        }
        self.scan_left = cfg.scan_interval;

        self.scan(dog_id, dog_pos, health, cfg, zombies, animals, bites, sounds, one_shot);
    }

    #[allow(clippy::too_many_arguments)]
    fn scan(
        &mut self,
        dog_id: u64,
        dog_pos: Vec3,
        health: &mut Health,
        cfg: &DogConfigData,
        zombies: &[(u64, Vec3, bool)],
        animals: &[(u64, Vec3, bool)],
        bites: &mut Vec<DogBite>,
        sounds: &mut Vec<(String, Vec3)>,
        one_shot: &mut Option<(&'static str, f32)>,
    ) {
        let mut nearest_any_dist = f32::MAX;
        let mut has_nearest_any = false;

        let mut fight_id: Option<u64> = None;
        let mut fight_pos: Option<Vec3> = None;
        let mut fight_dist = f32::MAX;

        // Scan zombies
        for &(z_id, z_pos, z_dead) in zombies {
            if z_dead {
                continue;
            }
            let dist = Self::flat(z_pos - dog_pos).length();
            if dist < nearest_any_dist {
                nearest_any_dist = dist;
                has_nearest_any = true;
            }
            if dist <= cfg.attack_range && dist < fight_dist {
                fight_id = Some(z_id);
                fight_pos = Some(z_pos);
                fight_dist = dist;
            }
        }

        // If no zombie fight target, check hostile/valid animals
        if fight_id.is_none() {
            for &(a_id, a_pos, a_dead) in animals {
                if a_dead {
                    continue;
                }
                let dist = Self::flat(a_pos - dog_pos).length();
                if dist <= cfg.attack_range && dist < fight_dist {
                    fight_id = Some(a_id);
                    fight_pos = Some(a_pos);
                    fight_dist = dist;
                }
            }
        }

        // Bark at enemies within barkRange
        if has_nearest_any && nearest_any_dist <= cfg.bark_range && self.bark_cooldown <= 0.0 {
            self.bark(dog_pos, cfg, sounds, one_shot);
        }

        self.target_id = fight_id;
        self.target_pos = fight_pos;
        if fight_pos.is_some() {
            let dist = Self::flat(fight_pos.unwrap() - dog_pos).length();
            self.engaged = dist <= cfg.bite_range;
        } else {
            self.engaged = false;
        }

        // Take bites from adjacent zombies
        self.take_bites_from_zombies(dog_pos, health, cfg, zombies);

        // Deliver bite if in range
        self.bite_target(dog_id, dog_pos, cfg, bites, sounds, one_shot);
    }

    fn bark(
        &mut self,
        dog_pos: Vec3,
        cfg: &DogConfigData,
        sounds: &mut Vec<(String, Vec3)>,
        one_shot: &mut Option<(&'static str, f32)>,
    ) {
        self.bark_cooldown = cfg.bark_cooldown;
        self.bark_count += 1;
        *one_shot = Some(("bark", 0.8));
        sounds.push(("dog_bark".to_string(), dog_pos));
    }

    fn bite_target(
        &mut self,
        dog_id: u64,
        dog_pos: Vec3,
        cfg: &DogConfigData,
        bites: &mut Vec<DogBite>,
        sounds: &mut Vec<(String, Vec3)>,
        one_shot: &mut Option<(&'static str, f32)>,
    ) {
        if !self.engaged || self.bite_cooldown > 0.0 {
            return;
        }
        let Some(t_id) = self.target_id else { return; };
        let Some(t_pos) = self.target_pos else { return; };

        self.bite_cooldown = cfg.bite_cooldown;
        self.bite_count += 1;
        *one_shot = Some(("bite", 0.6));
        sounds.push(("dog_bite".to_string(), dog_pos));

        let offset = Self::flat(t_pos - dog_pos);
        let dist = offset.length();
        let dir = if dist > 0.001 { offset / dist } else { Vec3::NEG_Z };

        let damage = DamageInfo {
            amount: cfg.bite_damage,
            source: Some(dog_id),
            hit_point: t_pos,
            direction: dir,
            damage_type: "dog_bite".to_string(),
            headshot: false,
            leg_hit: false,
        };

        bites.push(DogBite {
            target_id: t_id,
            damage,
            knockback: dir * 4.0,
        });
    }

    fn take_bites_from_zombies(
        &mut self,
        dog_pos: Vec3,
        health: &mut Health,
        cfg: &DogConfigData,
        zombies: &[(u64, Vec3, bool)],
    ) {
        if self.hurt_cooldown > 0.0 || health.is_dead() {
            return;
        }
        for &(z_id, z_pos, z_dead) in zombies {
            if z_dead {
                continue;
            }
            let dist = Self::flat(z_pos - dog_pos).length();
            if dist <= cfg.bite_range {
                self.hurt_cooldown = cfg.bite_cooldown;
                let offset = Self::flat(dog_pos - z_pos);
                let dir = if offset.length() > 0.001 { offset.normalize() } else { Vec3::Z };
                let info = DamageInfo {
                    amount: 10.0,
                    source: Some(z_id),
                    hit_point: dog_pos,
                    direction: dir,
                    damage_type: "bite".to_string(),
                    headshot: false,
                    leg_hit: false,
                };
                health.take_damage(info);
                return;
            }
        }
    }

    fn flat(v: Vec3) -> Vec3 {
        Vec3::new(v.x, 0.0, v.z)
    }
}
