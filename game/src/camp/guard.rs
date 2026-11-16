//! Guard behavior and state on settlement platforms (port of scripts/Settlement/SettlementGuard.cs).

use glam::Vec3;
use world::{DotnetRandom, SettlementConfigData};

use crate::camp::math::CampMath;
use crate::damage::DamageInfo;

/// Settlement guard behavior and combat state (Godot: SettlementGuard.cs).
#[derive(Debug, Clone, PartialEq)]
pub struct SettlementGuard {
    pub id: String,
    pub post: Vec3,
    pub yaw: f32,
    pub patrol_points: Vec<Vec3>,
    pub patrol_index: usize,
    pub alert: bool,
    pub target_pos: Option<Vec3>,
    pub scan_timer: f32,
    pub fire_timer: f32,
    pub shots_fired: u32,
    pub animation: String,
    pub sight_interval: f32,
    pub fire_interval: f32,
    pub range: f32,
    pub damage: f32,
    pub miss_chance: f32,
    pub eye_height: f32,
}

impl SettlementGuard {
    pub const READY_POSE: &'static str = "rifle_aim";
    pub const FIRE_ACTION: &'static str = "rifle_fire";

    pub fn new(
        id: impl Into<String>,
        post: Vec3,
        yaw: f32,
        cfg: &SettlementConfigData,
    ) -> Self {
        Self {
            id: id.into(),
            post,
            yaw,
            patrol_points: Vec::new(),
            patrol_index: 0,
            alert: false,
            target_pos: None,
            scan_timer: 0.0,
            fire_timer: 0.0,
            shots_fired: 0,
            animation: Self::READY_POSE.to_string(),
            sight_interval: if cfg.guard_sight_interval > 0.0 { cfg.guard_sight_interval } else { 0.5 },
            fire_interval: if cfg.guard_fire_interval > 0.0 { cfg.guard_fire_interval } else { 1.2 },
            range: if cfg.guard_range > 0.0 { cfg.guard_range } else { 40.0 },
            damage: if cfg.guard_damage > 0.0 { cfg.guard_damage } else { 60.0 },
            miss_chance: cfg.guard_miss_chance,
            eye_height: if cfg.guard_eye_height > 0.0 { cfg.guard_eye_height } else { 1.6 },
        }
    }

    pub fn with_patrol(mut self, points: Vec<Vec3>) -> Self {
        self.patrol_points = points;
        self
    }

    /// Eye point, measured from feet plus eye height.
    // Godot: SettlementGuard.cs Head
    pub fn eye_position(&self) -> Vec3 {
        self.post + Vec3::new(0.0, self.eye_height, 0.0)
    }

    /// Updates guard timers, targets nearest zombie within range, and fires.
    // Godot: SettlementGuard.cs _PhysicsProcess
    pub fn update(
        &mut self,
        dt: f32,
        zombies: &[Vec3],
        rng: &mut DotnetRandom,
    ) -> Option<DamageInfo> {
        self.scan_timer -= dt;
        if self.scan_timer <= 0.0 {
            self.scan_timer = self.sight_interval;
            self.target_pos = self.find_target(zombies);
        }

        let Some(target) = self.target_pos else {
            self.alert = false;
            self.fire_timer = 0.0;
            self.animation = Self::READY_POSE.to_string();
            return None;
        };

        self.alert = true;
        let offset = target - self.post;
        self.yaw = (-offset.x).atan2(-offset.z);

        self.fire_timer -= dt;
        if self.fire_timer <= 0.0 {
            self.fire_timer = self.fire_interval;
            self.shots_fired += 1;
            self.animation = Self::FIRE_ACTION.to_string();

            if (rng.next_double() as f32) < self.miss_chance {
                return None;
            }

            let head = self.eye_position();
            let chest = target + Vec3::new(0.0, 1.2, 0.0);
            let dir = (chest - head).normalize_or_zero();
            return Some(DamageInfo {
                amount: self.damage,
                source: None,
                hit_point: chest,
                direction: dir,
                damage_type: "bullet".to_string(),
                headshot: false,
                leg_hit: false,
            });
        }

        self.animation = Self::READY_POSE.to_string();
        None
    }

    fn find_target(&self, zombies: &[Vec3]) -> Option<Vec3> {
        let eye = self.eye_position();
        let mut best: Option<Vec3> = None;
        let mut best_dist = self.range;

        for &z_pos in zombies {
            let dist = CampMath::flat(eye, z_pos);
            if dist <= best_dist {
                best_dist = dist;
                best = Some(z_pos);
            }
        }
        best
    }
}
