//! Frame and draw types for dog rendering and simulation context.
// Godot: DogManager.cs, DogVisual.cs

use glam::Vec3;
use super::combat::DogBite;
use super::stray::DogPoiSite;

/// Per-dog draw state sent to the renderer.
#[derive(Clone, Debug, PartialEq)]
pub struct DogDrawState {
    pub id: u64,
    pub model: String,
    pub position: Vec3,
    pub yaw: f32,
    pub clip: String,
    pub alive: bool,
    pub scale: f32,
    pub is_stray: bool,
}

/// Output of a simulation frame.
#[derive(Clone, Debug, Default)]
pub struct DogFrame {
    pub draws: Vec<DogDrawState>,
    pub bites: Vec<DogBite>,
    pub sounds: Vec<(String, Vec3)>,
    pub toasts: Vec<String>,
}

/// Context passed to Dogs::update every tick.
#[derive(Clone, Copy, Default)]
pub struct DogUpdateContext<'a> {
    pub player_position: Option<Vec3>,
    pub player_forward: Vec3,
    pub player_dead: bool,
    pub player_sprinting: bool,
    pub player_velocity: Vec3,
    pub zombies: &'a [(u64, Vec3, bool)],
    pub animals: &'a [(u64, Vec3, bool)],
    pub is_night: bool,
    pub camp_position: Option<Vec3>,
    pub now_hours: f64,
    pub day: i32,
    pub poi_sites: &'a [DogPoiSite],
    pub world_seed: i32,
}

impl<'a> DogUpdateContext<'a> {
    pub fn new(player_position: Vec3) -> Self {
        Self {
            player_position: Some(player_position),
            player_forward: Vec3::NEG_Z,
            ..Default::default()
        }
    }

    pub fn with_forward(mut self, forward: Vec3) -> Self {
        self.player_forward = forward;
        self
    }

    #[allow(dead_code)]
    fn with_dead(mut self, dead: bool) -> Self {
        self.player_dead = dead;
        self
    }

    pub fn with_zombies(mut self, zombies: &'a [(u64, Vec3, bool)]) -> Self {
        self.zombies = zombies;
        self
    }

    #[allow(dead_code)]
    fn with_animals(mut self, animals: &'a [(u64, Vec3, bool)]) -> Self {
        self.animals = animals;
        self
    }

    #[allow(dead_code)]
    fn with_camp(mut self, camp_pos: Option<Vec3>) -> Self {
        self.camp_position = camp_pos;
        self
    }

    #[allow(dead_code)]
    fn with_time(mut self, day: i32, now_hours: f64) -> Self {
        self.day = day;
        self.now_hours = now_hours;
        self
    }

    #[allow(dead_code)]
    fn with_pois(mut self, world_seed: i32, poi_sites: &'a [DogPoiSite]) -> Self {
        self.world_seed = world_seed;
        self.poi_sites = poi_sites;
        self
    }
}
