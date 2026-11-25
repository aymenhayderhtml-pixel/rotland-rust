use std::collections::HashMap;
use std::path::Path;

use glam::{IVec2, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::biome::BiomeId;

/// Finite-difference step (metres) for ground slope estimation.
const SLOPE_STEP: f32 = 0.75; // new: DECISIONS 2026-11-25

/// One ground-detail model rule in `data/ground_detail.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetailRule {
    pub model: String,
    pub density_per_100m2: f32,
    pub scale: [f32; 2],
    pub radius_m: f32,
}

/// Configuration loaded from `data/ground_detail.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundDetailConfigData {
    #[serde(default = "default_cell_m")]
    pub cell_m: f32,
    #[serde(default = "default_radius_m")]
    pub radius_m: f32,
    #[serde(default = "default_fade_m")]
    pub fade_m: f32,
    #[serde(default = "default_max_instances")]
    pub max_instances: usize,
    #[serde(default = "default_max_slope")]
    pub max_slope: f32,
    #[serde(default = "default_ground_sink")]
    pub ground_sink: f32,
    #[serde(default)]
    pub biomes: HashMap<String, Vec<DetailRule>>,
}

fn default_cell_m() -> f32 {
    8.0 // data: ground_detail.json cell_m
}
fn default_radius_m() -> f32 {
    45.0 // data: ground_detail.json radius_m
}
fn default_fade_m() -> f32 {
    36.0 // data: ground_detail.json fade_m
}
fn default_max_instances() -> usize {
    25_000 // data: ground_detail.json max_instances
}
fn default_max_slope() -> f32 {
    0.7 // data: ground_detail.json max_slope
}
fn default_ground_sink() -> f32 {
    0.03 // data: ground_detail.json ground_sink
}

impl Default for GroundDetailConfigData {
    fn default() -> Self {
        Self {
            cell_m: default_cell_m(),
            radius_m: default_radius_m(),
            fade_m: default_fade_m(),
            max_instances: default_max_instances(),
            max_slope: default_max_slope(),
            ground_sink: default_ground_sink(),
            biomes: HashMap::new(),
        }
    }
}

impl GroundDetailConfigData {
    pub fn rules(&self, biome: BiomeId) -> &[DetailRule] {
        self.biomes.get(biome.key()).map_or(&[], Vec::as_slice)
    }
}

pub struct GroundDetailConfig;

impl GroundDetailConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<GroundDetailConfigData, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn from_json(json: &str) -> Result<GroundDetailConfigData, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// One placed ground-detail instance.
#[derive(Debug, Clone, PartialEq)]
pub struct DetailInstance {
    pub model: String,
    pub position: Vec3,
    pub yaw: f32,
    pub scale: f32,
    pub radius_m: f32,
}

/// Deterministic, jittered-grid ground detail placement for one `cell_m x cell_m` cell.
pub fn place(
    cell: IVec2,
    config: &GroundDetailConfigData,
    biome_at: &dyn Fn(Vec2) -> BiomeId,
    height_at: &dyn Fn(Vec2) -> f32,
    blocked: &dyn Fn(Vec2) -> bool,
    seed: i32,
) -> Vec<DetailInstance> {
    let cell_m = config.cell_m;
    if cell_m <= 0.0 {
        return Vec::new();
    }
    let origin = Vec2::new(cell.x as f32 * cell_m, cell.y as f32 * cell_m);
    let center = origin + Vec2::splat(cell_m * 0.5);
    let center_biome = biome_at(center);
    let rules = config.rules(center_biome);
    if rules.is_empty() {
        return Vec::new();
    }

    let cell_area_100m2 = (cell_m * cell_m) / 100.0; // new: DECISIONS 2026-11-25
    let mut out = Vec::new();

    for (rule_idx, rule) in rules.iter().enumerate() {
        if rule.density_per_100m2 <= 0.0 {
            continue;
        }
        let expected = rule.density_per_100m2 * cell_area_100m2;
        let n_side = (expected.ceil() as u32).max(1);
        let grid_n = (n_side as f32).sqrt().ceil() as u32;
        let sub = cell_m / grid_n as f32;
        let prob = (expected / (grid_n * grid_n) as f32).min(1.0);
        let mut rng = cell_rng(seed, cell, rule_idx as u32);

        for gz in 0..grid_n {
            for gx in 0..grid_n {
                if prob < 1.0 && rng.next_f32() >= prob {
                    continue;
                }
                let px = origin.x + (gx as f32 + rng.next_f32()) * sub;
                let pz = origin.y + (gz as f32 + rng.next_f32()) * sub;
                let pos2 = Vec2::new(px, pz);

                let yaw = rng.next_f32() * std::f32::consts::TAU;
                let scale = rule.scale[0] + rng.next_f32() * (rule.scale[1] - rule.scale[0]);

                if blocked(pos2) {
                    continue;
                }
                if biome_at(pos2) != center_biome {
                    continue;
                }
                if ground_normal_y(height_at, pos2) < config.max_slope {
                    continue;
                }
                let y = height_at(pos2) - config.ground_sink;
                out.push(DetailInstance {
                    model: rule.model.clone(),
                    position: Vec3::new(px, y, pz),
                    yaw,
                    scale,
                    radius_m: rule.radius_m,
                });
            }
        }
    }

    out
}

fn ground_normal_y(height_at: &dyn Fn(Vec2) -> f32, p: Vec2) -> f32 {
    let dhdx = (height_at(Vec2::new(p.x + SLOPE_STEP, p.y))
        - height_at(Vec2::new(p.x - SLOPE_STEP, p.y)))
        / (2.0 * SLOPE_STEP);
    let dhdz = (height_at(Vec2::new(p.x, p.y + SLOPE_STEP))
        - height_at(Vec2::new(p.x, p.y - SLOPE_STEP)))
        / (2.0 * SLOPE_STEP);
    1.0 / (dhdx * dhdx + 1.0 + dhdz * dhdz).sqrt()
}

struct CellRng(u32);

impl CellRng {
    fn next_f32(&mut self) -> f32 {
        // SplitMix32-style mixing for fast, deterministic [0, 1) floats.
        self.0 = self.0.wrapping_add(0x9e37_79b9);
        let mut z = self.0;
        z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b); // new: DECISIONS 2026-11-25
        z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35); // new: DECISIONS 2026-11-25
        z ^= z >> 16; // new: DECISIONS 2026-11-25
        (z >> 8) as f32 / ((1u32 << 24) as f32) // new: DECISIONS 2026-11-25
    }
}

fn cell_rng(seed: i32, cell: IVec2, rule_idx: u32) -> CellRng {
    let mut h = 2166136261u32; // new: DECISIONS 2026-11-25
    for v in [seed as u32, cell.x as u32, cell.y as u32, rule_idx] {
        h ^= v;
        h = h.wrapping_mul(16777619); // new: DECISIONS 2026-11-25
    }
    CellRng(h)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(density: f32) -> GroundDetailConfigData {
        let mut biomes = HashMap::new();
        biomes.insert(
            "plains".to_string(),
            vec![DetailRule {
                model: "grass_tuft_a".to_string(),
                density_per_100m2: density,
                scale: [0.8, 1.2],
                radius_m: 45.0,
            }],
        );
        GroundDetailConfigData {
            biomes,
            ..Default::default()
        }
    }

    #[test]
    fn same_seed_produces_same_instances() {
        let cfg = test_config(50.0);
        let a = place(
            IVec2::new(3, -5),
            &cfg,
            &|_| BiomeId::Plains,
            &|p| (p.x * 0.1).sin(),
            &|_| false,
            42,
        );
        let b = place(
            IVec2::new(3, -5),
            &cfg,
            &|_| BiomeId::Plains,
            &|p| (p.x * 0.1).sin(),
            &|_| false,
            42,
        );
        assert_eq!(a, b);
        assert!(!a.is_empty());
    }

    #[test]
    fn density_roughly_matches_data() {
        let density = 80.0;
        let cfg = test_config(density);
        let mut total = 0usize;
        let _cells = 25; // 5x5 cells = 25 * 64 m2 = 1600 m2
        for cz in 0..5 {
            for cx in 0..5 {
                let items = place(
                    IVec2::new(cx, cz),
                    &cfg,
                    &|_| BiomeId::Plains,
                    &|_| 0.0,
                    &|_| false,
                    123,
                );
                total += items.len();
            }
        }
        let expected = density * (25.0 * 64.0 / 100.0);
        let ratio = total as f32 / expected;
        assert!(
            (0.8..=1.2).contains(&ratio),
            "expected ~{expected}, got {total} (ratio {ratio})"
        );
    }

    #[test]
    fn blocked_points_get_nothing() {
        let cfg = test_config(100.0);
        let all_blocked = place(
            IVec2::new(0, 0),
            &cfg,
            &|_| BiomeId::Plains,
            &|_| 0.0,
            &|_| true,
            42,
        );
        assert!(all_blocked.is_empty());

        let half_blocked = place(
            IVec2::new(0, 0),
            &cfg,
            &|_| BiomeId::Plains,
            &|_| 0.0,
            &|p| p.x < 4.0,
            42,
        );
        assert!(!half_blocked.is_empty());
        assert!(half_blocked.iter().all(|i| i.position.x >= 4.0));
    }

    #[test]
    fn steep_slopes_are_skipped() {
        let cfg = test_config(100.0);
        // Steep 2:1 ramp -> normal.y = 1 / sqrt(5) ~ 0.447 < 0.7
        let steep = place(
            IVec2::new(0, 0),
            &cfg,
            &|_| BiomeId::Plains,
            &|p| p.x * 2.0,
            &|_| false,
            42,
        );
        assert!(steep.is_empty());
    }

    #[test]
    fn loads_data_ground_detail_json() {
        let cfg = GroundDetailConfig::load_from_file(paths::repo_path("data/ground_detail.json"))
            .expect("data/ground_detail.json should parse");
        assert!(cfg.max_instances <= 25_000);
        for biome in BiomeId::ALL {
            assert!(
                !cfg.rules(biome).is_empty(),
                "missing ground_detail rules for {}",
                biome.key()
            );
        }
    }
}
