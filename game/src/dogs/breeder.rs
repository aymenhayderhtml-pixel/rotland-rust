//! Dog breeder interaction at camp: purchasing a puppy with dollars.
// Godot: DogBreeder.cs

use glam::Vec3;
use crate::economy::Wallet;
use super::config::DogConfigData;
use super::manager::Dogs;

/// Camp breeder interactable object for buying a puppy.
// Godot: DogBreeder.cs
#[derive(Clone, Debug, PartialEq)]
pub struct DogBreeder {
    pub position: Vec3,
}

impl DogBreeder {
    pub const GROUP_NAME: &str = "dog_breeder";
    pub const ROOT_LIFT: f32 = 1.1;

    pub fn new(position: Vec3) -> Self {
        Self { position }
    }

    /// Interaction prompt text.
    // Godot: DogBreeder.cs GetPrompt
    pub fn prompt(&self, has_dog: bool, cfg: &DogConfigData) -> String {
        if has_dog {
            "Dog breeder (you already have a dog)".to_string()
        } else {
            format!("Buy a puppy (${})", cfg.price)
        }
    }

    /// Attempts to purchase a puppy from the breeder.
    // Godot: DogBreeder.cs Interact
    pub fn interact(
        &self,
        dogs: &mut Dogs,
        wallet: &mut Wallet,
        spawn_at: Vec3,
        day: i32,
        now_hours: f64,
    ) -> Result<u64, String> {
        dogs.buy_from_breeder(wallet, spawn_at, day, now_hours)
    }
}
