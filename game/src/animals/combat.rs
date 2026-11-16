//! A boar's fighting logic.
// Godot: AnimalCombat.cs

use glam::Vec3;
use super::config::AnimalSpeciesData;

pub const FIGHT_COOLDOWN: f32 = 1.5;
pub const FIGHT_TICK: f32 = 0.5;

/// Hit dealt by an animal (boar) to the player or a zombie.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimalAttackHit {
    pub animal_id: u64,
    pub target_id: Option<u64>,
    pub damage: f32,
    pub knockback: Vec3,
    pub hit_point: Vec3,
    pub is_player: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimalCombat {
    pub attack_cooldown: f32,
    pub fight_cooldown: f32,
    pub fight_tick: f32,
}

impl AnimalCombat {
    pub fn new() -> Self {
        Self::default()
    }

    // Godot: AnimalCombat.cs AttackReady
    pub fn attack_ready(&self) -> bool {
        self.attack_cooldown <= 0.0
    }

    // Godot: AnimalCombat.cs Reset
    pub fn reset(&mut self) {
        self.attack_cooldown = 0.0;
        self.fight_cooldown = 0.0;
        self.fight_tick = 0.0;
    }

    // Godot: AnimalCombat.cs Tick
    pub fn tick(&mut self, dt: f32) {
        self.attack_cooldown -= dt;
        self.fight_cooldown -= dt;
        self.fight_tick -= dt;
    }

    /// One gore hit on the player.
    // Godot: AnimalCombat.cs Strike
    pub fn strike(
        &mut self,
        animal_id: u64,
        animal_pos: Vec3,
        animal_yaw: f32,
        player_pos: Vec3,
        sp: &AnimalSpeciesData,
    ) -> AnimalAttackHit {
        self.attack_cooldown = sp.attack_cooldown;
        let mut dir = Vec3::new(player_pos.x - animal_pos.x, 0.0, player_pos.z - animal_pos.z);
        if dir.length_squared() > 0.0001 {
            dir = dir.normalize();
        } else {
            dir = Vec3::new(-animal_yaw.sin(), 0.0, -animal_yaw.cos());
        }
        AnimalAttackHit {
            animal_id,
            target_id: None,
            damage: sp.attack_damage,
            knockback: dir * sp.attack_knockback,
            hit_point: player_pos + Vec3::Y,
            is_player: true,
        }
    }

    /// Checks if a zombie is within fight range and attacks it.
    // Godot: AnimalCombat.cs FightZombies
    pub fn fight_zombies(
        &mut self,
        animal_id: u64,
        animal_pos: Vec3,
        zombies: &[(u64, Vec3, bool)], // (id, position, is_dead)
        sp: &AnimalSpeciesData,
    ) -> Vec<AnimalAttackHit> {
        if self.fight_tick > 0.0 || self.fight_cooldown > 0.0 {
            return Vec::new();
        }
        self.fight_tick = FIGHT_TICK;
        let mut hits = Vec::new();
        for &(z_id, z_pos, z_dead) in zombies {
            if z_dead {
                continue;
            }
            let offset = Vec3::new(z_pos.x - animal_pos.x, 0.0, z_pos.z - animal_pos.z);
            if offset.length() > sp.fight_range {
                continue;
            }
            let dir = if offset.length_squared() > 0.0001 {
                offset.normalize()
            } else {
                Vec3::NEG_Z
            };
            hits.push(AnimalAttackHit {
                animal_id,
                target_id: Some(z_id),
                damage: sp.fight_damage,
                knockback: dir * 2.0,
                hit_point: z_pos + Vec3::Y,
                is_player: false,
            });
        }
        if !hits.is_empty() {
            self.fight_cooldown = FIGHT_COOLDOWN;
        }
        hits
    }
}
