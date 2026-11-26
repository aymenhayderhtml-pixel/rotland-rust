//! Stray dogs sitting at house/cabin lots, fleeing if rushed, and their spawner.
// Godot: Stray.cs, StrayDogSpawner.cs

use glam::Vec3;
use physics::{CharacterMover, PhysicsWorld};

use game_core::loot::crate_seed;
use super::config::DogConfigData;
use super::motion::DogMotion;
use super::steering::DogSteering;

/// Lightweight description of a POI building lot for stray dog spawn checks.
#[derive(Clone, Debug, PartialEq)]
pub struct DogPoiSite {
    pub poi_id: String,
    pub building_id: String,
    pub position: Vec3,
    pub yaw: f32,
    pub half_depth: f32,
}

impl DogPoiSite {
    pub fn new(
        poi_id: impl Into<String>,
        building_id: impl Into<String>,
        position: Vec3,
        yaw: f32,
        half_depth: f32,
    ) -> Self {
        Self {
            poi_id: poi_id.into(),
            building_id: building_id.into(),
            position,
            yaw,
            half_depth,
        }
    }
}

/// A stray dog at a house or cabin lot. Not tamed yet: sits by the lot,
/// flees when rushed by a sprinting player, and can be tamed with meat.
// Godot: Stray.cs
#[derive(Debug)]
pub struct Stray {
    pub(super) id: u64,
    pub(super) lot_id: String,
    pub(super) position: Vec3,
    pub(super) yaw: f32,
    pub(super) velocity: Vec3,
    pub(super) on_floor: bool,
    pub(super) fleeing: bool,
    pub(super) flee_left: f32,
    pub(super) flee_target: Vec3,
    pub(super) flee_count: usize,
    pub(super) model: String,
    pub(super) motion: DogMotion,
    pub(super) steering: DogSteering,
}

impl Stray {
    pub fn new(id: u64, lot_id: String, pos: Vec3) -> Self {
        Self {
            id,
            lot_id,
            position: pos,
            yaw: 0.0,
            velocity: Vec3::ZERO,
            on_floor: true,
            fleeing: false,
            flee_left: 0.0,
            flee_target: pos,
            flee_count: 0,
            model: "dog_dark".to_string(),
            motion: DogMotion::new(),
            steering: DogSteering::new(id),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn lot_id(&self) -> &str {
        &self.lot_id
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn flee_count(&self) -> usize {
        self.flee_count
    }

    pub fn place_at(&mut self, pos: Vec3) {
        self.position = pos;
        self.motion.place_at();
    }

    /// Simulation step for a stray dog.
    // Godot: Stray.cs _PhysicsProcess
    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        dt: f32,
        player_pos: Option<Vec3>,
        player_sprinting: bool,
        player_vel: Vec3,
        cfg: &DogConfigData,
        mover: &CharacterMover,
        physics: &PhysicsWorld,
    ) {
        if self.fleeing {
            self.flee(dt, cfg, physics);
        } else if let Some(p_pos) = player_pos
            && self.is_rushed_at(p_pos, player_sprinting, player_vel, cfg) {
                self.start_flee(p_pos, cfg);
            } else {
                self.motion.idle(dt);
            }

        let walk_vel = if self.fleeing {
            let offset = DogMotion::flat(self.flee_target - self.position);
            let dist = offset.length();
            if dist > cfg.stop_distance {
                let dir = offset / dist;
                let heading = self.steering.choose(self.position, dir, dt, physics);
                self.yaw = DogMotion::face_direction(heading);
                heading * cfg.run_speed
            } else {
                Vec3::ZERO
            }
        } else {
            Vec3::ZERO
        };

        let moved = DogMotion::move_and_slide(
            mover,
            physics,
            self.position,
            walk_vel,
            Vec3::ZERO,
            self.velocity.y,
            self.on_floor,
            dt,
        );
        self.position = moved.position;
        self.velocity = moved.velocity;
        self.on_floor = moved.on_floor;
    }

    // Godot: Stray.cs IsRushedAt
    fn is_rushed_at(
        &self,
        player_pos: Vec3,
        player_sprinting: bool,
        player_vel: Vec3,
        cfg: &DogConfigData,
    ) -> bool {
        let to_stray = DogMotion::flat(self.position - player_pos);
        let moving = DogMotion::flat(player_vel);
        if !player_sprinting || to_stray.length() > cfg.stray_flee_range || moving.length() < 0.5 {
            return false;
        }
        moving.normalize().dot(to_stray.normalize()) > 0.5
    }

    // Godot: Stray.cs StartFlee
    fn start_flee(&mut self, player_pos: Vec3, cfg: &DogConfigData) {
        let offset = DogMotion::flat(self.position - player_pos);
        let away = if offset.length() > 0.001 { offset.normalize() } else { Vec3::Z };
        self.flee_target = self.position + away * cfg.stray_flee_distance;
        self.flee_left = cfg.stray_flee_seconds;
        self.fleeing = true;
        self.flee_count += 1;
    }

    // Godot: Stray.cs Flee
    fn flee(&mut self, dt: f32, cfg: &DogConfigData, physics: &PhysicsWorld) {
        self.flee_left -= dt;
        let (_vel, _yaw, moving) = self.motion.step(
            self.position,
            self.yaw,
            self.flee_target,
            cfg.run_speed,
            dt,
            cfg,
            &mut self.steering,
            physics,
        );
        if self.flee_left <= 0.0 || !moving {
            self.fleeing = false;
        }
    }
}

/// Helper methods for deterministic stray dog lot placement.
// Godot: StrayDogSpawner.cs
pub struct StrayDogSpawner;

impl StrayDogSpawner {
    pub const STRAY_SALT: &str = "/stray";

    /// True when this building gets a stray dog in this world. Same seed, same answer.
    // Godot: StrayDogSpawner.cs RollsStray
    pub fn rolls_stray(world_seed: i32, poi_id: &str, building_id: &str, cfg: &DogConfigData) -> bool {
        if !cfg.stray_buildings.iter().any(|b| b == building_id) {
            return false;
        }
        let key = format!("{poi_id}{}", Self::STRAY_SALT);
        let seed = crate_seed(world_seed, &key);
        let mut rng = world::DotnetRandom::new(seed);
        rng.next_double() < cfg.stray_chance as f64
    }

    /// Point just outside the building's front wall, so the dog sits beside the lot.
    // Godot: StrayDogSpawner.cs SitPoint
    pub fn sit_point(poi_pos: Vec3, poi_yaw: f32, half_depth: f32) -> Vec3 {
        let outside = glam::Quat::from_rotation_y(poi_yaw) * Vec3::new(0.0, 0.0, half_depth + 2.0);
        poi_pos + outside
    }
}
