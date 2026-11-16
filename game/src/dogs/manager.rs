//! Dogs manager owning the pet dog and wild strays, coordinating commands and state.
// Godot: DogManager.cs

use std::collections::HashSet;
use glam::Vec3;
use physics::CharacterMover;

use crate::damage::DamageInfo;
use crate::economy::Wallet;
use crate::items::Inventory;
use crate::melee::MeleeTarget;

use super::config::DogConfigData;
use super::dog::Dog;
use super::food::DogFood;
use super::growth::DogGrowth;
use super::mode::DogMode;
use super::stray::Stray;

/// The Dogs manager: owns the one live pet dog, tracks strays and dead count.
// Godot: DogManager.cs
#[derive(Debug)]
pub struct Dogs {
    pub config: DogConfigData,
    pub(super) dog: Option<Dog>,
    pub(super) strays: Vec<Stray>,
    pub tamed: HashSet<String>,
    pub dead_count: i32,
    pub(super) next_id: u64,
    pub(super) stray_timer: f32,
    pub(super) spawner_enabled: bool,
    pub(super) mover: CharacterMover,
}

impl Dogs {
    pub fn new(config: DogConfigData) -> Self {
        Self {
            config,
            dog: None,
            strays: Vec::new(),
            tamed: HashSet::new(),
            dead_count: 0,
            next_id: 1,
            stray_timer: 0.0,
            spawner_enabled: true,
            mover: CharacterMover::new(0.3, 0.8),
        }
    }

    pub fn from_repo() -> Result<Self, String> {
        let config = DogConfigData::from_repo()?;
        Ok(Self::new(config))
    }

    pub fn has_dog(&self) -> bool {
        self.dog.as_ref().is_some_and(|d| !d.is_dead())
    }

    pub fn dog(&self) -> Option<&Dog> {
        self.dog.as_ref()
    }

    pub fn dog_mut(&mut self) -> Option<&mut Dog> {
        self.dog.as_mut()
    }

    pub fn strays(&self) -> &[Stray] {
        &self.strays
    }

    pub fn dead_count(&self) -> i32 {
        self.dead_count
    }

    pub fn is_tamed(&self, lot_id: &str) -> bool {
        self.tamed.contains(lot_id)
    }

    /// Orders the dog to sit where it is or follow again (matching G key).
    // Godot: DogManager.cs ToggleStay
    pub fn toggle_stay(&mut self) {
        if let Some(dog) = self.dog.as_mut() && !dog.is_dead() {
            let next = if dog.mode == DogMode::Stay { DogMode::Follow } else { DogMode::Stay };
            dog.set_mode(next);
        }
    }

    /// Orders the dog to walk back to camp (matching Shift+G key).
    // Godot: DogManager.cs GoHome
    pub fn go_home(&mut self) {
        if let Some(dog) = self.dog.as_mut() && !dog.is_dead() {
            dog.set_mode(DogMode::Home);
        }
    }

    /// Sets order mode on the active pet dog.
    // Godot: Dog.cs SetMode
    pub fn set_mode(&mut self, mode: DogMode) {
        if let Some(dog) = self.dog.as_mut() {
            dog.set_mode(mode);
        }
    }

    /// Feeds the dog from bag: heals feedHeal and removes 1 food item.
    // Godot: Dog.cs FeedFromBag
    pub fn feed(&mut self, dog_id: u64, bag: &mut Inventory) -> Result<String, String> {
        let Some(dog) = self.dog.as_mut() else {
            return Err("Dog not found.".to_string());
        };
        if dog.id != dog_id {
            return Err("Dog id mismatch.".to_string());
        }
        if dog.is_dead() {
            return Err("Dog is dead.".to_string());
        }
        if !dog.feed_from_bag(bag, &self.config) {
            return Err("No dog food in inventory.".to_string());
        }
        Ok(format!("Fed {}.", dog.name))
    }

    /// Renames the active dog (trimmed, at most max_name_length).
    // Godot: DogManager.cs SetDogName
    pub fn set_name(&mut self, name: &str) {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return;
        }
        let max_len = self.config.max_name_length;
        let final_name = if trimmed.len() > max_len {
            &trimmed[..max_len]
        } else {
            trimmed
        };
        if let Some(dog) = self.dog.as_mut() {
            dog.set_name(final_name.to_string());
        }
    }

    /// Tames a stray dog at lot_id using 1 food item from bag.
    // Godot: DogManager.cs TameStray
    pub fn adopt_stray(&mut self, lot_id: &str, bag: &mut Inventory, day: i32) -> Result<u64, String> {
        if self.has_dog() {
            return Err("You already have a dog. Only one at a time.".to_string());
        }
        if !DogFood::take(bag, &self.config.feed_items) {
            return Err("A stray dog. Raw or cooked meat might win it over.".to_string());
        }
        let at = if let Some(idx) = self.strays.iter().position(|s| s.lot_id == lot_id) {
            let s = self.strays.remove(idx);
            s.position
        } else {
            Vec3::ZERO
        };
        self.tamed.insert(lot_id.to_string());
        let name = self.config.default_name.clone();
        let id = self.spawn_dog(at, self.config.max_health, DogMode::Follow, &name, day, DogGrowth::NO_START);
        Ok(id)
    }

    /// Buys a puppy from the camp breeder with dollars from wallet.
    // Godot: DogManager.cs BuyPuppy
    pub fn buy_from_breeder(
        &mut self,
        wallet: &mut Wallet,
        spawn_at: Vec3,
        day: i32,
        now_hours: f64,
    ) -> Result<u64, String> {
        if self.has_dog() {
            return Err("You already have a dog. Only one at a time.".to_string());
        }
        let price = self.config.price;
        if wallet.dollars() < price {
            return Err(format!("A puppy costs ${price}. You have ${}.", wallet.dollars()));
        }
        if !wallet.try_spend(price) {
            return Err("Not enough dollars.".to_string());
        }
        let name = self.config.default_name.clone();
        let id = self.spawn_dog(spawn_at, self.config.max_health, DogMode::Follow, &name, day, now_hours);
        Ok(id)
    }

    /// Spawns the live dog under the manager. Replaces any existing dog.
    // Godot: DogManager.cs Spawn
    pub fn spawn_dog(
        &mut self,
        at: Vec3,
        health: f32,
        mode: DogMode,
        name: &str,
        bought_day: i32,
        growth_start: f64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut dog = Dog::new(id, name.to_string(), mode, at, &self.config);
        dog.bought_day = bought_day;
        dog.growth_start = growth_start;
        dog.health.set_current(health);
        dog.place_at(at);
        self.dog = Some(dog);
        id
    }

    /// Spawns a stray dog directly at a lot (tests and spawner).
    // Godot: StrayDogSpawner.cs SpawnAt
    pub fn spawn_stray_at(&mut self, lot_id: &str, at: Vec3) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut stray = Stray::new(id, lot_id.to_string(), at);
        stray.place_at(at);
        self.strays.push(stray);
        id
    }

    /// Applies damage to a dog or stray. Returns true if hit killed it.
    pub fn hit(&mut self, id: u64, damage: DamageInfo, knockback: Vec3) -> bool {
        if let Some(dog) = self.dog.as_mut() && dog.id == id {
            if dog.is_dead() {
                return false;
            }
            let res = dog.health.take_damage(damage);
            if res.dealt > 0.0 {
                dog.knock += knockback;
                dog.since_hurt = 0.0;
            }
            if res.died_now {
                self.dead_count += 1;
                dog.play_die();
                let _ = dog.drop_pack();
                return true;
            }
            return false;
        }

        // Stray dogs
        if let Some(pos) = self.strays.iter().position(|s| s.id == id) {
            self.strays.remove(pos);
            return true;
        }
        false
    }

    /// Targets list for melee combat query.
    pub fn targets(&self) -> Vec<MeleeTarget> {
        let mut list = Vec::new();
        if let Some(dog) = &self.dog
            && !dog.is_dead() {
                list.push(MeleeTarget {
                    id: dog.id,
                    position: dog.position,
                    radius: 0.4,
                });
            }
        for stray in &self.strays {
            list.push(MeleeTarget {
                id: stray.id,
                position: stray.position,
                radius: 0.4,
            });
        }
        list
    }

    /// True if the target entity is friendly to the player (the pet dog).
    pub fn is_friendly(&self, id: u64) -> bool {
        self.dog.as_ref().is_some_and(|d| d.id == id)
    }
}
