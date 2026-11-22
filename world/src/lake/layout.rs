use glam::Vec2;

use crate::lake::config::LakeConfigData;
use crate::lake::generate::{
    ellipse_radius, generate_docks, generate_fishing_spots, generate_village_lots,
};
use crate::lake::mesh::{build_lake_water_mesh, LakeWaterMesh};
pub use crate::lake::types::{Dock, FishingSpot, FishingSpotKind, LakeZone, VillageLot};
use crate::math::smooth_step;
use crate::noise::GodotNoise;
use crate::random::DotnetRandom;

/// Layout and geometry for the lake & coast region.
#[derive(Debug, Clone)]
pub struct LakeLayout {
    pub center: Vec2,
    pub radius: Vec2,
    pub water_level: f32,
    pub shore: Vec<Vec2>,
    pub docks: Vec<Dock>,
    pub village_lots: Vec<VillageLot>,
    pub fishing_spots: Vec<FishingSpot>,
    pub config: LakeConfigData,
    pub noise: GodotNoise,
}

impl PartialEq for LakeLayout {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center
            && self.radius == other.radius
            && (self.water_level - other.water_level).abs() < 1e-5
            && self.shore == other.shore
            && self.docks == other.docks
            && self.village_lots == other.village_lots
            && self.fishing_spots == other.fishing_spots
            && self.config == other.config
    }
}

impl LakeLayout {
    /// Generates a complete lake layout deterministically from a seed, camp, optional city center, and config.
    pub fn generate(
        seed: i32,
        camp: Vec2,
        city_center: Option<Vec2>,
        cfg: &LakeConfigData,
        raw_height: impl Fn(Vec2) -> f32,
    ) -> Self {
        let radius_v = Vec2::new(cfg.radius[0], cfg.radius[1]);

        let dir = match city_center {
            Some(city) => {
                let to_city = city - camp;
                if to_city.length_squared() > 1e-6 {
                    let to_city_norm = to_city.normalize();
                    let rad = cfg.angle_from_city.to_radians();
                    let cos = rad.cos();
                    let sin = rad.sin();
                    Vec2::new(
                        to_city_norm.x * cos - to_city_norm.y * sin,
                        to_city_norm.x * sin + to_city_norm.y * cos,
                    )
                    .normalize()
                } else {
                    let mut rng = DotnetRandom::new(seed);
                    let angle = (rng.next_double() * std::f64::consts::TAU) as f32;
                    Vec2::new(angle.cos(), angle.sin())
                }
            }
            None => {
                let mut rng = DotnetRandom::new(seed);
                let angle = (rng.next_double() * std::f64::consts::TAU) as f32;
                Vec2::new(angle.cos(), angle.sin())
            }
        };

        let r_towards_camp = ellipse_radius(-dir, radius_v);
        let center = camp + dir * (cfg.distance_from_camp + r_towards_camp);

        let noise = GodotNoise::fbm(seed + 100, cfg.shore_noise.frequency, 3);

        let samples = cfg.shore_samples;
        let mut shore = Vec::with_capacity(samples);
        let tau = std::f32::consts::TAU;
        for i in 0..samples {
            let theta = (i as f32) * (tau / samples as f32);
            let u = Vec2::new(theta.cos(), theta.sin());
            let r_base = ellipse_radius(u, radius_v);
            let noise_val = noise.get_noise_2d(center.x + u.x * r_base, center.y + u.y * r_base);
            let r_shore = r_base + noise_val * cfg.shore_noise.amplitude;
            shore.push(center + u * r_shore);
        }

        let shore_raw_sum: f32 = shore.iter().map(|&pt| raw_height(pt)).sum();
        let shore_raw_avg = if shore.is_empty() {
            raw_height(center)
        } else {
            shore_raw_sum / (shore.len() as f32)
        };
        let water_level = shore_raw_avg + cfg.water_level_above_shore;

        let to_camp = -dir;
        let base_angle = to_camp.y.atan2(to_camp.x);

        let village_lots = generate_village_lots(
            center,
            radius_v,
            base_angle,
            &cfg.village,
            cfg.beach_width,
            cfg.shore_noise.amplitude,
            &noise,
            water_level,
            cfg.docks.deck_height,
            &raw_height,
        );

        let docks = generate_docks(
            center,
            radius_v,
            base_angle,
            cfg.docks.count,
            cfg.docks.length,
            cfg.shore_noise.amplitude,
            &noise,
            water_level,
            cfg.docks.deck_height,
        );

        let fishing_spots = generate_fishing_spots(
            center,
            radius_v,
            base_angle,
            &docks,
            cfg.beach_width,
            cfg.shore_noise.amplitude,
            &noise,
        );

        Self {
            center,
            radius: radius_v,
            water_level,
            shore,
            docks,
            village_lots,
            fishing_spots,
            config: cfg.clone(),
            noise,
        }
    }

    /// Signed distance to the shoreline (negative in water, 0 at shore, positive on land).
    pub fn shore_distance(&self, p: Vec2) -> f32 {
        let d = p - self.center;
        let dist = d.length();
        let u = if dist > 1e-5 {
            d / dist
        } else {
            Vec2::new(1.0, 0.0)
        };
        let r_base = ellipse_radius(u, self.radius);
        let noise_val = self.noise.get_noise_2d(self.center.x + u.x * r_base, self.center.y + u.y * r_base);
        let r_shore = r_base + noise_val * self.config.shore_noise.amplitude;
        dist - r_shore
    }

    /// Depth below water level (0 at shore line, smoothly growing to maxDepth toward the middle).
    pub fn depth_at(&self, p: Vec2) -> f32 {
        let d = self.shore_distance(p);
        if d >= 0.0 {
            return 0.0;
        }
        let dist = (p - self.center).length();
        let u = if dist > 1e-5 {
            (p - self.center) / dist
        } else {
            Vec2::new(1.0, 0.0)
        };
        let r_base = ellipse_radius(u, self.radius);
        let noise_val = self.noise.get_noise_2d(self.center.x + u.x * r_base, self.center.y + u.y * r_base);
        let r_shore = (r_base + noise_val * self.config.shore_noise.amplitude).max(1.0);
        let fraction = (1.0 - dist / r_shore).clamp(0.0, 1.0);
        smooth_step(0.0, 1.0, fraction) * self.config.max_depth
    }

    /// Zone classification at position p.
    pub fn zone_at(&self, p: Vec2) -> LakeZone {
        let d = self.shore_distance(p);
        if d > self.config.beach_width {
            LakeZone::Land
        } else if d >= 0.0 {
            LakeZone::Beach
        } else if d >= -self.config.shallow_width {
            LakeZone::Shallow
        } else {
            LakeZone::Deep
        }
    }

    /// Bounding box check with margin for fast early-outs.
    pub fn contains_region(&self, p: Vec2) -> bool {
        let margin = self.config.beach_width + self.config.beach_blend + self.config.region_margin;
        let max_extent = self.radius + Vec2::splat(self.config.shore_noise.amplitude + margin);
        let min = self.center - max_extent;
        let max = self.center + max_extent;
        p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y
    }

    /// Axis-aligned bounding box of the lake water surface.
    pub fn water_bounds(&self) -> (Vec2, Vec2) {
        let margin = self.config.shore_noise.amplitude + 4.0;
        let max_extent = self.radius + Vec2::splat(margin);
        (self.center - max_extent, self.center + max_extent)
    }

    /// Builds a watertight triangle mesh of the lake water surface.
    pub fn build_water_mesh(&self) -> LakeWaterMesh {
        build_lake_water_mesh(self)
    }
}
