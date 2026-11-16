//! A group of animals that spooks together.
// Godot: AnimalHerd.cs

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimalHerd {
    pub id: u64,
    pub member_ids: Vec<u64>,
}

impl AnimalHerd {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            member_ids: Vec::new(),
        }
    }

    // Godot: AnimalHerd.cs Add
    pub fn add(&mut self, animal_id: u64) {
        if !self.member_ids.contains(&animal_id) {
            self.member_ids.push(animal_id);
        }
    }

    // Godot: AnimalHerd.cs Remove
    pub fn remove(&mut self, animal_id: u64) {
        self.member_ids.retain(|&id| id != animal_id);
    }

    pub fn len(&self) -> usize {
        self.member_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.member_ids.is_empty()
    }
}
