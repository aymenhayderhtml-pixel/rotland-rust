//! Top-level Animals system managing active animals, pool, spawning, hits, and butchering.
// Godot: AnimalSpawner.cs, Animal.cs

use glam::Vec3;
use physics::CharacterMover;
use game_core::damage::DamageInfo;
use game_core::items::{Inventory, ItemStack};
use crate::melee::MeleeTarget;

use super::combat::AnimalAttackHit;
use super::config::AnimalConfigData;
use super::hunting::AnimalHunting;
use super::loot::AnimalLoot;
use super::instance::AnimalInstance;
use super::motion::AnimalMotion;
use super::pool::AnimalPool;
use super::rng::AnimalRng;

/// Output state per animal per frame for rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimalDrawState {
    pub id: u64,
    pub species: String,
    pub model: String,
    pub position: Vec3,
    pub yaw: f32,
    pub anim: String,
    pub alive: bool,
}

/// What occurred during the animal simulation frame.
#[derive(Clone, Debug, Default)]
pub struct AnimalFrame {
    pub draws: Vec<AnimalDrawState>,
    pub attacks: Vec<AnimalAttackHit>,
    pub sounds: Vec<(String, Vec3)>,
}

/// Context provided to the animals system for each simulation tick.
#[derive(Clone, Copy, Default)]
pub struct AnimalUpdateContext<'a> {
    pub player_position: Option<Vec3>,
    pub player_forward: Vec3,
    pub player_is_crouching: bool,
    pub player_is_sprinting: bool,
    pub player_is_dead: bool,
    pub player_hunting_stealth_bonus: f32,
    pub player_hunting_level: u32,
    pub is_night: bool,
    pub biome_name: Option<&'a str>,
    pub noises: &'a [(Vec3, f32)],
    pub zombies: &'a [(u64, Vec3, bool)],
    pub in_no_spawn_zone: Option<&'a dyn Fn(Vec3) -> bool>,
}

impl<'a> AnimalUpdateContext<'a> {
    pub fn new(player_position: Vec3) -> Self {
        Self {
            player_position: Some(player_position),
            player_forward: Vec3::NEG_Z,
            ..Default::default()
        }
    }

    pub fn with_night(mut self, is_night: bool) -> Self {
        self.is_night = is_night;
        self
    }

    pub fn with_biome(mut self, biome: &'a str) -> Self {
        self.biome_name = Some(biome);
        self
    }
}

pub struct Animals {
    pub config: AnimalConfigData,
    pub list: Vec<AnimalInstance>,
    pub pool: AnimalPool,
    pub enabled: bool,
    pub(super) mover: CharacterMover,
    pub(super) next_id: u64,
    pub(super) next_herd_id: u64,
    pub(super) rng: AnimalRng,
    pub(super) kills: Vec<String>,
    pub(super) play_time: f32,
    pub(super) timer: f32,
}

impl Animals {
    pub fn new(config: AnimalConfigData, seed: u64) -> Self {
        Self {
            config,
            list: Vec::new(),
            pool: AnimalPool::new(),
            enabled: true,
            mover: CharacterMover::default(),
            next_id: 1000,
            next_herd_id: 1,
            rng: AnimalRng::new(seed),
            kills: Vec::new(),
            play_time: 0.0,
            timer: 0.0,
        }
    }

    pub fn from_repo() -> Result<Self, String> {
        let config = AnimalConfigData::from_repo()?;
        Ok(Self::new(config, 0x9E37_79B9_7F4A_7C15))
    }

    /// Ids are handed out from here on (keeps them apart from zombie ids).
    pub fn set_first_id(&mut self, id: u64) {
        self.next_id = id;
    }

    pub fn active_count(&self) -> usize {
        self.list.len()
    }

    pub fn live_count(&self) -> usize {
        self.list.iter().filter(|a| !a.is_dead()).count()
    }

    pub fn take_kills(&mut self) -> Vec<String> {
        std::mem::take(&mut self.kills)
    }

    // Godot: AnimalSpawner.cs SpawnAt
    pub fn spawn_at(
        &mut self,
        species_id: &str,
        feet: Vec3,
        herd_id: Option<u64>,
        model: Option<&str>,
    ) -> Option<u64> {
        let sp = self.config.species(species_id)?.clone();
        let model_id = model.unwrap_or(&sp.model);
        let seed = self.rng.next_u64();
        let mut animal = self.pool.acquire(
            &mut self.next_id,
            species_id,
            model_id,
            &sp,
            seed,
            feet,
        );
        animal.herd_id = herd_id;
        let id = animal.id;
        self.list.push(animal);
        Some(id)
    }

    // Godot: Animal.cs OnDamaged & OnDied
    pub fn hit(&mut self, id: u64, damage: DamageInfo, knockback: Vec3) -> bool {
        let b = self.config.behaviour.clone();
        let butcher = self.config.butcher.clone();
        let Some(a) = self.list.iter_mut().find(|a| a.id == id) else {
            return false;
        };
        if a.is_dead() {
            return false;
        }
        let res = a.health.take_damage(damage.clone());
        if res.dealt > 0.0 {
            a.knock += Vec3::new(knockback.x, 0.0, knockback.z);
            let from = damage.hit_point;
            let _ = a.ai.on_hurt(from, &a.species_data, &b);
        }
        if res.died_now {
            a.ai.die(&butcher);
            a.play_die();
            self.kills.push(a.species_id.clone());
        }
        res.died_now
    }

    pub fn targets(&self) -> Vec<MeleeTarget> {
        self.list
            .iter()
            .filter(|a| !a.health.is_dead())
            .map(|a| MeleeTarget {
                id: a.id,
                position: a.position,
                radius: a.species_data.body_radius,
            })
            .collect()
    }

    // Godot: Animal.cs Finish & Interact
    pub fn butcher(&mut self, id: u64, bag: &mut Inventory) -> Vec<ItemStack> {
        self.butcher_with_hunting_level(id, bag, 1)
    }

    pub fn butcher_with_hunting_level(&mut self, id: u64, bag: &mut Inventory, hunting_level: u32) -> Vec<ItemStack> {
        let Some(a) = self.list.iter_mut().find(|a| a.id == id) else {
            return Vec::new();
        };
        if !a.is_dead() || a.butcher.done || a.retired {
            return Vec::new();
        }
        let extra = AnimalHunting::extra_meat(hunting_level, &self.config.butcher);
        let items = AnimalLoot::roll(&a.species_data, a.is_buck(), extra);
        let mut added = Vec::new();
        for (item_id, count) in items {
            let leftover = bag.add(&item_id, count);
            let fit = count.saturating_sub(leftover);
            if fit > 0 {
                added.push(ItemStack::new(item_id, fit));
            }
        }
        a.butcher.set_done();
        a.retired = true;
        added
    }

    // Godot: AnimalSpawner.cs Release
    pub fn despawn_far(&mut self, player_pos: Vec3) {
        let far = self.config.spawn.despawn_dist;
        let mut i = 0;
        while i < self.list.len() {
            let dist = AnimalMotion::flat(self.list[i].position - player_pos).length();
            if dist > far || self.list[i].can_release() {
                let animal = self.list.remove(i);
                self.pool.release(animal);
            } else {
                i += 1;
            }
        }
    }

    pub fn draw_states(&self) -> Vec<AnimalDrawState> {
        self.list
            .iter()
            .map(|a| AnimalDrawState {
                id: a.id,
                species: a.species_id.clone(),
                model: a.model_id.clone(),
                position: a.position,
                yaw: a.yaw,
                anim: a.current_clip().to_string(),
                alive: !a.is_dead(),
            })
            .collect()
    }

    pub fn to_save(&self) -> serde_json::Value {
        serde_json::json!({
            "animals": []
        })
    }

    pub fn from_save(&mut self, _value: &serde_json::Value) {
        // Godot does not save transient wild animals in save files
    }
}
