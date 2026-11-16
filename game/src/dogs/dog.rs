//! The pet dog entity representation.
// Godot: Dog.cs

use glam::Vec3;

use crate::health::Health;
use crate::items::{Inventory, ItemStack};
use super::combat::DogCombat;
use super::config::DogConfigData;
use super::food::DogFood;
use super::growth::DogGrowth;
use super::mode::DogMode;
use super::motion::DogMotion;
use super::steering::DogSteering;

/// The pet dog entity. Follows the player, fights zombies, regenerates when not hurt,
/// and carries a small pack. At 0 health it dies for good and drops its pack.
// Godot: Dog.cs
#[derive(Debug)]
pub struct Dog {
    pub(super) id: u64,
    pub(super) name: String,
    pub bought_day: i32,
    pub growth_start: f64,
    pub(super) mode: DogMode,
    pub(super) health: Health,
    pub(super) pack: Inventory,
    pub(super) position: Vec3,
    pub(super) yaw: f32,
    pub(super) velocity: Vec3,
    pub(super) on_floor: bool,
    pub(super) knock: Vec3,
    pub(super) model: String,
    pub(super) is_friendly: bool,
    pub(super) current_clip: String,
    pub(super) busy: f32,
    pub(super) body_left: f32,
    pub(super) since_hurt: f32,
    pub(super) speed: f32,
    pub(super) motion: DogMotion,
    pub(super) combat: DogCombat,
    pub(super) steering: DogSteering,
}

impl Dog {
    pub fn new(id: u64, name: String, mode: DogMode, pos: Vec3, cfg: &DogConfigData) -> Self {
        Self {
            id,
            name,
            bought_day: 1,
            growth_start: DogGrowth::NO_START,
            mode,
            health: Health::new(cfg.max_health),
            pack: Inventory::with_capacity(cfg.pack_slots, cfg.pack_weight),
            position: pos,
            yaw: 0.0,
            velocity: Vec3::ZERO,
            on_floor: true,
            knock: Vec3::ZERO,
            model: "dog_tan".to_string(),
            is_friendly: true,
            current_clip: "idle".to_string(),
            busy: 0.0,
            body_left: cfg.body_seconds,
            since_hurt: 0.0,
            speed: 0.0,
            motion: DogMotion::new(),
            combat: DogCombat::new(),
            steering: DogSteering::new(id),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn mode(&self) -> DogMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: DogMode) {
        if !self.is_dead() {
            self.mode = mode;
        }
    }

    pub fn health(&self) -> &Health {
        &self.health
    }

    pub fn health_mut(&mut self) -> &mut Health {
        &mut self.health
    }

    pub fn pack(&self) -> &Inventory {
        &self.pack
    }

    pub fn pack_mut(&mut self) -> &mut Inventory {
        &mut self.pack
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn is_dead(&self) -> bool {
        self.health.is_dead()
    }

    pub fn is_friendly(&self) -> bool {
        self.is_friendly
    }

    pub fn idle_seconds(&self) -> f32 {
        self.motion.idle_seconds()
    }

    pub fn bark_count(&self) -> usize {
        self.combat.bark_count()
    }

    pub fn bite_count(&self) -> usize {
        self.combat.bite_count()
    }

    pub fn place_at(&mut self, pos: Vec3) {
        self.position = pos;
        self.velocity = Vec3::ZERO;
        self.motion.place_at();
    }

    /// Feeds one food item from the bag (heals feedHeal). False when dead or no food.
    // Godot: Dog.cs FeedFromBag
    pub fn feed_from_bag(&mut self, bag: &mut Inventory, cfg: &DogConfigData) -> bool {
        if self.is_dead() || !DogFood::take(bag, &cfg.feed_items) {
            return false;
        }
        self.health.heal(cfg.feed_heal);
        true
    }

    /// Plays a one-shot animation clip for the given duration.
    // Godot: DogVisual.cs PlayOnce
    pub fn play_once(&mut self, clip: &'static str, duration: f32) {
        if self.is_dead() {
            return;
        }
        self.current_clip = clip.to_string();
        self.busy = duration;
    }

    /// Plays die animation and marks dog dead.
    // Godot: DogVisual.cs PlayDie
    pub fn play_die(&mut self) {
        self.current_clip = "die".to_string();
        self.busy = 0.0;
        self.combat.clear();
    }

    /// Drops all items currently stored in the dog's pack.
    // Godot: Dog.cs DropPack
    pub fn drop_pack(&mut self) -> Vec<ItemStack> {
        let mut dropped = Vec::new();
        for i in 0..self.pack.slot_count() {
            if let Some(stack) = self.pack.take_slot(i) {
                dropped.push(stack);
            }
        }
        dropped
    }

    pub fn current_scale(&self, now_hours: f64, cfg: &DogConfigData) -> f32 {
        DogGrowth::scale(now_hours, self.growth_start, cfg)
    }

    pub fn current_clip(&self) -> &str {
        &self.current_clip
    }

    /// Regenerates health over time when not recently damaged.
    // Godot: Dog.cs Regenerate
    pub fn regenerate(&mut self, dt: f32, cfg: &DogConfigData) {
        self.since_hurt += dt;
        if self.since_hurt >= cfg.regen_delay_seconds && self.health.current() < self.health.max() {
            self.health.heal(cfg.regen_per_second * dt);
        }
    }

    /// Updates visual animation state based on locomotion and idle timers.
    // Godot: Dog.cs UpdateVisual
    pub(super) fn update_visual(&mut self, cfg: &DogConfigData) {
        if self.is_dead() {
            self.current_clip = "die".to_string();
            return;
        }
        if self.busy > 0.0 {
            return;
        }
        if self.motion.is_moving() {
            self.current_clip = if self.speed > cfg.walk_speed + 0.01 {
                "run".to_string()
            } else {
                "walk".to_string()
            };
            return;
        }
        if self.mode == DogMode::Stay {
            self.current_clip = "sit".to_string();
        } else if self.motion.idle_seconds() >= cfg.lie_after_seconds {
            self.current_clip = "lie".to_string();
        } else if self.motion.idle_seconds() >= cfg.sit_after_seconds {
            self.current_clip = "sit".to_string();
        } else {
            self.current_clip = "idle".to_string();
        }
    }
}
