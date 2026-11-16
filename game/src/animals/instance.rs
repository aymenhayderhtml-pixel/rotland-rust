//! One wild animal instance.
// Godot: Animal.cs, AnimalVisual.cs

use glam::Vec3;
use crate::health::Health;
use super::ai::AnimalAI;
use super::butcher::AnimalButcher;
use super::config::AnimalSpeciesData;
use super::motion::AnimalMotion;
use super::state::AnimalState;

pub const PARKED_Y: f32 = -1000.0;

#[derive(Clone, Debug)]
pub struct AnimalInstance {
    pub id: u64,
    pub species_id: String,
    pub model_id: String,
    pub species_data: AnimalSpeciesData,
    pub health: Health,
    pub position: Vec3,
    pub velocity: Vec3,
    pub knock: Vec3,
    pub yaw: f32,
    pub on_floor: bool,
    pub ai: AnimalAI,
    pub motion: AnimalMotion,
    pub butcher: AnimalButcher,
    pub herd_id: Option<u64>,
    pub ai_paused: bool,
    pub retired: bool,
    pub anim_clip: String,
    pub anim_time: f32,
    pub anim_busy: f32,
    pub is_buck: bool,
    pub visible: bool,
}

impl AnimalInstance {
    pub fn new(
        id: u64,
        species_id: &str,
        model_id: &str,
        species_data: AnimalSpeciesData,
        seed: u64,
        feet: Vec3,
    ) -> Self {
        let is_buck = species_data.buck_model.as_deref() == Some(model_id);
        let max_hp = species_data.health;
        let mut ai = AnimalAI::new(&species_data, seed);
        ai.reset(feet);
        Self {
            id,
            species_id: species_id.to_string(),
            model_id: model_id.to_string(),
            species_data,
            health: Health::new(max_hp),
            position: feet,
            velocity: Vec3::ZERO,
            knock: Vec3::ZERO,
            yaw: 0.0,
            on_floor: true,
            ai,
            motion: AnimalMotion::new(),
            butcher: AnimalButcher::new(),
            herd_id: None,
            ai_paused: false,
            retired: false,
            anim_clip: "idle".to_string(),
            anim_time: 0.0,
            anim_busy: 0.0,
            is_buck,
            visible: true,
        }
    }

    // Godot: Animal.cs IsBuck
    pub fn is_buck(&self) -> bool {
        self.is_buck
    }

    // Godot: Animal.cs IsDead
    pub fn is_dead(&self) -> bool {
        self.health.is_dead() || self.ai.state == AnimalState::Dead
    }

    // Godot: Animal.cs IsValidAndAlive
    pub fn is_valid_and_alive(&self) -> bool {
        !self.is_dead() && !self.retired
    }

    // Godot: Animal.cs CanRelease
    pub fn can_release(&self) -> bool {
        self.retired || (self.is_dead() && self.ai.body_left <= 0.0)
    }

    // Godot: Animal.cs Reset
    pub fn reset(&mut self, feet: Vec3) {
        self.retired = false;
        self.health.revive(None);
        self.butcher.reset();
        self.ai.reset(feet);
        self.position = feet;
        self.velocity = Vec3::ZERO;
        self.knock = Vec3::ZERO;
        self.ai_paused = false;
        self.visible = true;
        self.anim_clip = "idle".to_string();
        self.anim_time = 0.0;
        self.anim_busy = 0.0;
    }

    // Godot: Animal.cs Park
    pub fn park(&mut self) {
        self.herd_id = None;
        self.visible = false;
        self.position = Vec3::new(0.0, PARKED_Y, 0.0);
    }

    // Godot: AnimalVisual.cs SetLocomotion
    pub fn set_locomotion(&mut self, clip: &str) {
        if self.is_dead() || self.anim_busy > 0.0 {
            return;
        }
        if self.anim_clip != clip {
            self.anim_clip = clip.to_string();
            self.anim_time = 0.0;
        }
    }

    // Godot: AnimalVisual.cs PlayOnce
    pub fn play_once(&mut self, clip: &str, duration: f32) {
        if self.is_dead() {
            return;
        }
        self.anim_clip = clip.to_string();
        self.anim_time = 0.0;
        self.anim_busy = duration;
    }

    // Godot: AnimalVisual.cs PlayDie
    pub fn play_die(&mut self) {
        self.anim_clip = "die".to_string();
        self.anim_busy = 0.0;
    }

    // Godot: AnimalVisual.cs Tick
    pub fn tick_visual(&mut self, dt: f32) {
        self.anim_time += dt;
        if self.anim_busy > 0.0 {
            self.anim_busy -= dt;
        }
    }

    pub fn current_clip(&self) -> &str {
        &self.anim_clip
    }
}
