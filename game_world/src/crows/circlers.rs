//! High-altitude circling crows over the city (slow loops, wing flap animation).

use std::f32::consts::TAU;

use glam::Vec3;

use crate::animals::AnimalRng;
use crate::crows::CrowsConfig;
use crate::crows::flock::{CrowPose, dir_to_yaw, flying_model};

#[derive(Clone, Debug)]
struct Circler {
    radius: f32,
    height: f32,
    angle: f32,
    angular_speed: f32,
    phase: f32,
    wing_time: f32,
}

/// High-altitude crows circling slowly over the city.
#[derive(Clone, Debug)]
pub struct Circlers {
    center: Vec3,
    items: Vec<Circler>,
}

impl Circlers {
    /// Spawns `cfg.circlers` circling crows above `city_center`.
    pub fn new(city_center: Vec3, seed: u64, cfg: &CrowsConfig) -> Self {
        let mut rng = AnimalRng::new(seed);
        let mut items = Vec::with_capacity(cfg.circlers);
        let r_lo = cfg.circle_radius[0].min(cfg.circle_radius[1]);
        let r_hi = cfg.circle_radius[0].max(cfg.circle_radius[1]);
        let h_lo = cfg.circle_height[0].min(cfg.circle_height[1]);
        let h_hi = cfg.circle_height[0].max(cfg.circle_height[1]);
        for _ in 0..cfg.circlers {
            let radius = rng.gen_range_f32(r_lo, r_hi).max(1.0);
            let height = rng.gen_range_f32(h_lo, h_hi);
            let angle = rng.gen_range_f32(0.0, TAU);
            let sign = if rng.next_f32() < 0.5 { 1.0 } else { -1.0 }; // new: DECISIONS 2026-11-27
            let angular_speed = sign * (cfg.fly_speed / radius);
            let phase = rng.next_f32();
            items.push(Circler {
                radius,
                height,
                angle,
                angular_speed,
                phase,
                wing_time: phase,
            });
        }
        Self { center: city_center, items }
    }

    /// Advances all circling crows by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        for c in &mut self.items {
            c.angle = (c.angle + c.angular_speed * dt).rem_euclid(TAU);
            c.wing_time += dt;
        }
    }

    /// Iterates over the poses of all circling crows.
    pub fn instances<'a>(&'a self, cfg: &'a CrowsConfig) -> impl Iterator<Item = CrowPose> + 'a {
        self.items.iter().map(move |c| {
            let (s, cs) = c.angle.sin_cos();
            let pos = self.center + Vec3::new(cs * c.radius, c.height, s * c.radius);
            let dir = Vec3::new(-s * c.angular_speed.signum(), 0.0, cs * c.angular_speed.signum());
            CrowPose {
                pos,
                yaw: dir_to_yaw(dir),
                model: flying_model(c.phase, c.wing_time, cfg),
            }
        })
    }
}
