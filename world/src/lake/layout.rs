use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::color::Color;
use crate::lake::config::{LakeConfigData, VillageConfig};
use crate::math::{lerp, smooth_step};
use crate::noise::GodotNoise;
use crate::random::DotnetRandom;

/// Zone classification relative to the lake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LakeZone {
    Land,
    Beach,
    Shallow,
    Deep,
}

/// A pier or dock starting on land and extending into the water.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dock {
    pub start: Vec2,
    pub end: Vec2,
    pub yaw: f32,
}

impl Dock {
    pub fn length(&self) -> f32 {
        self.start.distance(self.end)
    }

    pub fn forward(&self) -> Vec2 {
        Vec2::new(-self.yaw.sin(), -self.yaw.cos())
    }
}

/// A building lot in the fishing village along the shore.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VillageLot {
    pub center: Vec2,
    pub size: Vec2,
    pub yaw: f32,
    pub building_id: String,
}

impl VillageLot {
    pub fn corners(&self) -> [Vec2; 4] {
        let forward = Vec2::new(-self.yaw.sin(), -self.yaw.cos());
        let right = Vec2::new(-forward.y, forward.x);
        let hx = right * (self.size.x * 0.5);
        let hy = forward * (self.size.y * 0.5);
        [
            self.center - hx - hy,
            self.center + hx - hy,
            self.center + hx + hy,
            self.center - hx + hy,
        ]
    }

    pub fn aabb(&self) -> (Vec2, Vec2) {
        let corners = self.corners();
        let mut min = corners[0];
        let mut max = corners[0];
        for &c in &corners[1..] {
            min = min.min(c);
            max = max.max(c);
        }
        (min, max)
    }

    pub fn overlaps(&self, other: &VillageLot) -> bool {
        let corners_a = self.corners();
        let corners_b = other.corners();
        let axes = [
            (corners_a[1] - corners_a[0]).normalize(),
            (corners_a[3] - corners_a[0]).normalize(),
            (corners_b[1] - corners_b[0]).normalize(),
            (corners_b[3] - corners_b[0]).normalize(),
        ];
        for axis in axes {
            let (min_a, max_a) = project_onto_axis(&corners_a, axis);
            let (min_b, max_b) = project_onto_axis(&corners_b, axis);
            if max_a < min_b || max_b < min_a {
                return false;
            }
        }
        true
    }
}

fn project_onto_axis(corners: &[Vec2; 4], axis: Vec2) -> (f32, f32) {
    let mut min = corners[0].dot(axis);
    let mut max = min;
    for &c in &corners[1..] {
        let d = c.dot(axis);
        if d < min {
            min = d;
        }
        if d > max {
            max = d;
        }
    }
    (min, max)
}

/// The kind of fishing spot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FishingSpotKind {
    Dock,
    Beach,
}

/// A placeable fishing spot on a dock or beach.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FishingSpot {
    pub position: Vec2,
    pub kind: FishingSpotKind,
}

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

#[inline]
pub(crate) fn ellipse_radius(u: Vec2, radius: Vec2) -> f32 {
    let qx = u.x / radius.x;
    let qy = u.y / radius.y;
    1.0 / (qx * qx + qy * qy).max(1e-6).sqrt()
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
        );

        let docks = generate_docks(
            center,
            radius_v,
            base_angle,
            cfg.docks.count,
            cfg.docks.length,
            cfg.shore_noise.amplitude,
            &noise,
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

    /// Carves the lake bed and flat beach into the terrain, returning the adjusted height.
    pub fn ground_height(&self, p: Vec2, raw: f32) -> f32 {
        if !self.contains_region(p) {
            return raw;
        }
        let d = self.shore_distance(p);
        let blend_end = self.config.beach_width + self.config.beach_blend;
        if d >= blend_end {
            return raw;
        }
        if d < 0.0 {
            self.water_level - self.depth_at(p)
        } else if d <= self.config.beach_width {
            let beach_fraction = if self.config.beach_width > 1e-5 {
                d / self.config.beach_width
            } else {
                0.0
            };
            self.water_level + beach_fraction * self.config.water_level_above_shore
        } else {
            let beach_edge_height = self.water_level + self.config.water_level_above_shore;
            let t = smooth_step(self.config.beach_width, blend_end, d);
            lerp(beach_edge_height, raw, t)
        }
    }

    /// Axis-aligned bounding box of the lake water surface.
    pub fn water_bounds(&self) -> (Vec2, Vec2) {
        // data: lake.json shoreNoise
        let margin = self.config.shore_noise.amplitude + 4.0;
        let max_extent = self.radius + Vec2::splat(margin);
        (self.center - max_extent, self.center + max_extent)
    }

    // data: lake.json sand
    const COLOR_SCALE: f32 = 1.0 / 255.0;

    pub fn sand_color(&self) -> Color {
        Color::new(
            self.config.sand[0] as f32 * Self::COLOR_SCALE,
            self.config.sand[1] as f32 * Self::COLOR_SCALE,
            self.config.sand[2] as f32 * Self::COLOR_SCALE,
            1.0,
        )
    }

    pub fn wet_sand_color(&self) -> Color {
        Color::new(
            self.config.wet_sand[0] as f32 * Self::COLOR_SCALE,
            self.config.wet_sand[1] as f32 * Self::COLOR_SCALE,
            self.config.wet_sand[2] as f32 * Self::COLOR_SCALE,
            1.0,
        )
    }

    pub fn lake_bed_color(&self) -> Color {
        Color::new(
            self.config.lake_bed[0] as f32 * Self::COLOR_SCALE,
            self.config.lake_bed[1] as f32 * Self::COLOR_SCALE,
            self.config.lake_bed[2] as f32 * Self::COLOR_SCALE,
            1.0,
        )
    }

    /// Paints the terrain inside the lake region (lake bed, wet sand, dry sand beach).
    pub fn paint(&self, base_color: Color, wx: f32, wz: f32) -> Color {
        let p = Vec2::new(wx, wz);
        if !self.contains_region(p) {
            return base_color;
        }
        let d = self.shore_distance(p);
        let blend_end = self.config.beach_width + self.config.beach_blend;
        if d >= blend_end {
            return base_color;
        }

        if d < 0.0 {
            let t = if self.config.shallow_width > 1e-4 {
                (1.0 + d / self.config.shallow_width).clamp(0.0, 1.0)
            } else {
                0.0
            };
            self.lake_bed_color().lerp(self.wet_sand_color(), t)
        } else if d <= self.config.beach_width {
            // data: lake.json beachWidth
            let wet_width = (self.config.beach_width * 0.25).max(2.0);
            if d < wet_width {
                let t = d / wet_width;
                self.wet_sand_color().lerp(self.sand_color(), t)
            } else {
                self.sand_color()
            }
        } else {
            let t = smooth_step(self.config.beach_width, blend_end, d);
            self.sand_color().lerp(base_color, t)
        }
    }
}

fn generate_village_lots(
    center: Vec2,
    radius: Vec2,
    base_angle: f32,
    village_cfg: &VillageConfig,
    beach_width: f32,
    noise_amp: f32,
    noise: &GodotNoise,
) -> Vec<VillageLot> {
    let mut lots = Vec::with_capacity(village_cfg.count);
    let span = 1.0;
    for i in 0..village_cfg.count {
        let t = if village_cfg.count > 1 {
            (i as f32) / (village_cfg.count as f32 - 1.0) - 0.5
        } else {
            0.0
        };
        let angle = base_angle + t * span;
        let u = Vec2::new(angle.cos(), angle.sin());
        let r_base = ellipse_radius(u, radius);
        let noise_val = noise.get_noise_2d(center.x + u.x * r_base, center.y + u.y * r_base);
        let r_shore = r_base + noise_val * noise_amp;
        let shore_pt = center + u * r_shore;

        let size_arr = village_cfg.sizes[i % village_cfg.sizes.len()];
        let size = Vec2::new(size_arr[0], size_arr[1]);
        let distance_inland = beach_width + size.y * 0.5 + 4.0;
        let lot_center = shore_pt + u * distance_inland;

        let forward = -u;
        let yaw = (-forward.x).atan2(-forward.y);

        let touches_water = {
            let right = Vec2::new(-forward.y, forward.x);
            let hx = right * (size.x * 0.5);
            let hy = forward * (size.y * 0.5);
            let corners = [
                lot_center - hx - hy,
                lot_center + hx - hy,
                lot_center + hx + hy,
                lot_center - hx + hy,
            ];
            corners.iter().any(|&c| {
                let d = c - center;
                let dist = d.length();
                let dir_u = if dist > 1e-5 { d / dist } else { Vec2::new(1.0, 0.0) };
                let r_b = ellipse_radius(dir_u, radius);
                let n_val = noise.get_noise_2d(center.x + dir_u.x * r_b, center.y + dir_u.y * r_b);
                dist <= r_b + n_val * noise_amp
            })
        };

        const INLAND_BUILDINGS: [&str; 4] = ["fish_shack", "house_small", "cabin", "shed"];
        let building_id = if touches_water {
            "boathouse".to_string()
        } else {
            INLAND_BUILDINGS[i % INLAND_BUILDINGS.len()].to_string()
        };

        lots.push(VillageLot {
            center: lot_center,
            size,
            yaw,
            building_id,
        });
    }
    lots
}

fn generate_docks(
    center: Vec2,
    radius: Vec2,
    base_angle: f32,
    count: usize,
    length: f32,
    noise_amp: f32,
    noise: &GodotNoise,
) -> Vec<Dock> {
    let mut docks = Vec::with_capacity(count);
    for j in 0..count {
        let t = if count > 1 {
            (j as f32 / (count as f32 - 1.0) - 0.5) * 0.5
        } else {
            0.0
        };
        let dock_angle = base_angle + t;
        let u = Vec2::new(dock_angle.cos(), dock_angle.sin());
        let r_base = ellipse_radius(u, radius);
        let noise_val = noise.get_noise_2d(center.x + u.x * r_base, center.y + u.y * r_base);
        let r_shore = r_base + noise_val * noise_amp;
        let shore_pt = center + u * r_shore;

        let forward = -u;
        let start = shore_pt + u * 2.0;
        let end = start + forward * length;
        let yaw = (-forward.x).atan2(-forward.y);

        docks.push(Dock { start, end, yaw });
    }
    docks
}

fn generate_fishing_spots(
    center: Vec2,
    radius: Vec2,
    base_angle: f32,
    docks: &[Dock],
    beach_width: f32,
    noise_amp: f32,
    noise: &GodotNoise,
) -> Vec<FishingSpot> {
    let mut spots = Vec::new();
    for dock in docks {
        spots.push(FishingSpot {
            position: dock.end,
            kind: FishingSpotKind::Dock,
        });
    }

    let beach_angles = [
        base_angle - 0.5,
        base_angle - 0.25,
        base_angle + 0.25,
        base_angle + 0.5,
    ];
    for angle in beach_angles {
        let u = Vec2::new(angle.cos(), angle.sin());
        let r_base = ellipse_radius(u, radius);
        let noise_val = noise.get_noise_2d(center.x + u.x * r_base, center.y + u.y * r_base);
        let r_shore = r_base + noise_val * noise_amp;
        let shore_pt = center + u * r_shore;
        let beach_pos = shore_pt + u * (beach_width * 0.5);
        spots.push(FishingSpot {
            position: beach_pos,
            kind: FishingSpotKind::Beach,
        });
    }

    spots
}
