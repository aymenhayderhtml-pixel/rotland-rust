use std::path::Path;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::city::config::District;
use crate::city::layout::CityLayout;
use crate::color::Color;
use crate::math::{lerp, smooth_step};
use crate::noise::GodotNoise;
use crate::settlement::{SettlementConfigData, SettlementGround, SettlementSite};

// data: city_ground.json asphalt
const COLOR_SCALE: f32 = 1.0 / 255.0;

fn rgb(c: [u8; 3]) -> Color {
    Color::new(
        c[0] as f32 * COLOR_SCALE,
        c[1] as f32 * COLOR_SCALE,
        c[2] as f32 * COLOR_SCALE,
        1.0,
    )
}

/// Configuration data loaded from data/city_ground.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CityGroundData {
    pub edge_blend: f32,
    pub road_width: f32,
    pub road_blend: f32,
    pub road_smooth_distance: f32,
    pub sidewalk_width: f32,
    pub asphalt: [u8; 3],
    pub sidewalk: [u8; 3],
    pub lot_ground: [u8; 3],
    pub dirt: [u8; 3],
}

impl CityGroundData {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let data: Self = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        data.validate()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(data)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let data: Self = serde_json::from_str(json)?;
        Ok(data)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.edge_blend <= 0.0
            || self.road_width <= 0.0
            || self.road_blend <= 0.0
            || self.road_smooth_distance <= 0.0
            || self.sidewalk_width <= 0.0
        {
            return Err("CityGroundData has values out of range".to_string());
        }
        Ok(())
    }
}

/// Cached tilted plane approximating raw height across the city square.
#[derive(Debug, Clone, PartialEq)]
pub struct CityPlane {
    pub center: Vec2,
    pub half_size: f32,
    pub h_center: f32,
    pub slope_x: f32,
    pub slope_z: f32,
}

impl CityPlane {
    pub fn fit(
        origin: Vec2,
        size: f32,
        max_slope: f32,
        raw_height: &impl Fn(f32, f32) -> f32,
    ) -> Self {
        let half_size = size * 0.5;
        let center = origin + Vec2::splat(half_size);
        let h_center = raw_height(center.x, center.y);

        let h00 = raw_height(origin.x, origin.y);
        let h10 = raw_height(origin.x + size, origin.y);
        let h01 = raw_height(origin.x, origin.y + size);
        let h11 = raw_height(origin.x + size, origin.y + size);

        let mut slope_x = ((h10 - h00) + (h11 - h01)) / (2.0 * size);
        let mut slope_z = ((h01 - h00) + (h11 - h10)) / (2.0 * size);

        let total_change = (slope_x.abs() + slope_z.abs()) * size;
        if total_change > max_slope && total_change > 1e-6 {
            let factor = max_slope / total_change;
            slope_x *= factor;
            slope_z *= factor;
        }

        Self {
            center,
            half_size,
            h_center,
            slope_x,
            slope_z,
        }
    }

    #[inline]
    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        self.h_center + self.slope_x * (x - self.center.x) + self.slope_z * (z - self.center.y)
    }
}

/// City ground flattening and surface painting rules.
pub struct CityGround {
    pub plane: CityPlane,
    pub config: CityGroundData,
    pub patch: GodotNoise,
    pub asphalt: Color,
    pub sidewalk: Color,
    pub lot_ground: Color,
    pub dirt: Color,
}

impl CityGround {
    pub fn new(
        seed: i32,
        layout: &CityLayout,
        raw_height: &impl Fn(f32, f32) -> f32,
        max_slope: f32,
        config: CityGroundData,
    ) -> Self {
        let plane = CityPlane::fit(layout.origin, layout.size, max_slope, raw_height);
        // new: DECISIONS 2026-11-21
        let patch = GodotNoise::fbm(seed + 11, 0.03, 2);
        let asphalt = rgb(config.asphalt);
        let sidewalk = rgb(config.sidewalk);
        let lot_ground = rgb(config.lot_ground);
        let dirt = rgb(config.dirt);
        Self {
            plane,
            config,
            patch,
            asphalt,
            sidewalk,
            lot_ground,
            dirt,
        }
    }

    /// Flattens the ground across the city square, blending outside over edge_blend.
    pub fn flatten_city(&self, raw: f32, x: f32, z: f32) -> f32 {
        let d_box = (x - self.plane.center.x).abs().max((z - self.plane.center.y).abs());
        let half = self.plane.half_size;
        let edge_blend = self.config.edge_blend;
        if d_box >= half + edge_blend {
            return raw;
        }
        let plane_h = self.plane.height_at(x, z);
        let t = if d_box <= half {
            0.0
        } else {
            smooth_step(half, half + edge_blend, d_box)
        };
        lerp(plane_h, raw, t)
    }

    /// Flattens along city.road, blending to terrain. Skips inside camp's flat lot.
    #[allow(clippy::too_many_arguments)]
    pub fn flatten_road(
        &self,
        raw: f32,
        x: f32,
        z: f32,
        road: &[Vec2],
        settlements: &[SettlementSite],
        settlement_cfg: &SettlementConfigData,
        raw_height: &dyn Fn(f32, f32) -> f32,
    ) -> f32 {
        if road.len() < 2 {
            return raw;
        }
        if SettlementGround::nearest_box_distance(settlements, x, z) <= settlement_cfg.flat_radius {
            return raw;
        }

        let p = Vec2::new(x, z);
        let max_dist = self.config.road_width * 0.5 + self.config.road_blend;
        let mut best_dist = f32::MAX;
        let mut best_proj = Vec2::ZERO;
        let mut best_dir = Vec2::ZERO;

        for seg in road.windows(2) {
            let ab = seg[1] - seg[0];
            let len_sq = ab.length_squared();
            if len_sq < 1e-6 {
                continue;
            }
            let t = ((p - seg[0]).dot(ab) / len_sq).clamp(0.0, 1.0);
            let proj = seg[0] + ab * t;
            let d = p.distance(proj);
            if d < best_dist {
                best_dist = d;
                best_proj = proj;
                best_dir = ab / len_sq.sqrt();
            }
        }

        if best_dist >= max_dist {
            return raw;
        }

        let p_neg = best_proj - best_dir * self.config.road_smooth_distance;
        let p_pos = best_proj + best_dir * self.config.road_smooth_distance;
        let road_height = (raw_height(p_neg.x, p_neg.y) + raw_height(p_pos.x, p_pos.y)) * 0.5;

        let half_w = self.config.road_width * 0.5;
        let t = if best_dist <= half_w {
            0.0
        } else {
            smooth_step(half_w, max_dist, best_dist)
        };
        lerp(road_height, raw, t)
    }

    /// Paints streets, sidewalks, non-park blocks, and the camp road.
    pub fn paint(
        &self,
        base_color: Color,
        wx: f32,
        wz: f32,
        city: &CityLayout,
        settlements: &[SettlementSite],
        settlement_cfg: &SettlementConfigData,
    ) -> Color {
        if SettlementGround::nearest_box_distance(settlements, wx, wz) <= settlement_cfg.flat_radius {
            return base_color;
        }

        let p = Vec2::new(wx, wz);
        let half_size = city.size * 0.5;
        let city_center = city.origin + Vec2::splat(half_size);
        let d_box = (wx - city_center.x).abs().max((wz - city_center.y).abs());

        if d_box <= half_size + self.config.edge_blend {
            let mut on_road = false;
            let mut on_sidewalk = false;

            for s in &city.streets {
                if s.contains_point(p) {
                    let proj = s.closest_point(p);
                    let dist = p.distance(proj);
                    let half_w = s.width * 0.5;
                    if dist <= half_w - self.config.sidewalk_width {
                        on_road = true;
                        break;
                    } else {
                        on_sidewalk = true;
                    }
                }
            }

            if on_road {
                return self.asphalt;
            }
            if on_sidewalk {
                return self.sidewalk;
            }

            if let Some(district) = city.district_at(p) {
                if district == District::Park {
                    return base_color;
                }
                let noise_val = self.patch.get_noise_2d(wx, wz);
                // new: DECISIONS 2026-11-21
                let factor = (0.55 + 0.35 * noise_val).clamp(0.0, 1.0);
                // new: DECISIONS 2026-11-21
                let ground = self.lot_ground.lerp(self.dirt, smooth_step(-0.2, -0.6, noise_val) * 0.5);
                return base_color.lerp(ground, factor);
            }
        }

        if city.road.len() >= 2 {
            let mut best_dist = f32::MAX;
            for seg in city.road.windows(2) {
                let ab = seg[1] - seg[0];
                let len_sq = ab.length_squared();
                if len_sq < 1e-6 {
                    continue;
                }
                let t = ((p - seg[0]).dot(ab) / len_sq).clamp(0.0, 1.0);
                let proj = seg[0] + ab * t;
                best_dist = best_dist.min(p.distance(proj));
            }

            let half_w = self.config.road_width * 0.5;
            let max_blend = half_w + self.config.road_blend;
            if best_dist < max_blend {
                let noise_val = self.patch.get_noise_2d(wx, wz);
                // new: DECISIONS 2026-11-21
                let inner = half_w - 1.0;
                // new: DECISIONS 2026-11-21
                let crack_t = ((best_dist - inner) / 2.0 + noise_val * 0.3).clamp(0.0, 1.0);
                // new: DECISIONS 2026-11-21
                let road_surf = self.asphalt.lerp(self.dirt, crack_t * (0.6 + 0.4 * noise_val).clamp(0.0, 1.0));
                let t = if best_dist <= half_w {
                    0.0
                } else {
                    smooth_step(half_w, max_blend, best_dist)
                };
                return road_surf.lerp(base_color, t);
            }
        }

        base_color
    }
}
