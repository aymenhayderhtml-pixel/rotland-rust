//! Object pooling for AnimalInstance by model id.
// Godot: AnimalPool.cs

use std::collections::HashMap;
use glam::Vec3;
use super::config::AnimalSpeciesData;
use super::instance::AnimalInstance;

#[derive(Clone, Debug, Default)]
pub struct AnimalPool {
    parked: HashMap<String, Vec<AnimalInstance>>,
}

impl AnimalPool {
    pub fn new() -> Self {
        Self::default()
    }

    // Godot: AnimalPool.cs Acquire
    pub fn acquire(
        &mut self,
        next_id: &mut u64,
        species_id: &str,
        model_id: &str,
        sp: &AnimalSpeciesData,
        seed: u64,
        feet: Vec3,
    ) -> AnimalInstance {
        if let Some(stack) = self.parked.get_mut(model_id)
            && let Some(mut animal) = stack.pop()
        {
            animal.species_id = species_id.to_string();
            animal.species_data = sp.clone();
            animal.reset(feet);
            animal
        } else {
            let id = *next_id;
            *next_id += 1;
            AnimalInstance::new(id, species_id, model_id, sp.clone(), seed, feet)
        }
    }

    // Godot: AnimalPool.cs Release
    pub fn release(&mut self, mut animal: AnimalInstance) {
        animal.park();
        self.parked.entry(animal.model_id.clone()).or_default().push(animal);
    }

    pub fn parked_count(&self) -> usize {
        self.parked.values().map(Vec::len).sum()
    }
}
