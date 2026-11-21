use glam::{IVec2, Vec2};
use crate::city::config::{CityConfigData, District};
use crate::city::lots::{generate_lots, CityLot};
use crate::city::streets::{generate_streets, StreetSegment};
use crate::random::DotnetRandom;

// new: DECISIONS 2026-11-21
const SAMPLE_GRID: usize = 5;
// new: DECISIONS 2026-11-21
const SAMPLES_COUNT: f32 = 25.0;

/// One chunk block in the city grid.
#[derive(Debug, Clone, PartialEq)]
pub struct CityBlock {
    pub coord: IVec2,
    pub district: District,
    pub present: bool,
}

/// The complete city layout.
#[derive(Debug, Clone, PartialEq)]
pub struct CityLayout {
    pub origin: Vec2,
    pub size: f32,
    pub blocks: Vec<CityBlock>,
    pub streets: Vec<StreetSegment>,
    pub lots: Vec<CityLot>,
    pub entry: Vec2,
}

impl CityLayout {
    /// Generates a complete city layout deterministically from a seed and configuration.
    pub fn generate(
        seed: i32,
        camp: Vec2,
        cfg: &CityConfigData,
        forest_at: impl Fn(Vec2) -> f32,
    ) -> Self {
        let size = cfg.blocks as f32 * cfg.block_size;
        let half_size = size * 0.5;

        let mut rng = DotnetRandom::new(seed);
        let start_angle = (rng.next_double() * std::f64::consts::TAU) as f32;

        let mut best_score = f32::INFINITY;
        let mut best_origin = Vec2::ZERO;

        // Step 1: Direction finding avoiding forest.
        for i in 0..cfg.direction_tries {
            let angle = start_angle + (i as f32) * (std::f32::consts::TAU / cfg.direction_tries as f32);
            let u = Vec2::new(angle.cos(), angle.sin());

            let d = cfg.distance_from_camp;
            let h = half_size;
            let abs_ux = u.x.abs();
            let abs_uy = u.y.abs();

            let t = if abs_ux < 1e-5 {
                (d + h) / abs_uy.max(1e-5)
            } else if abs_uy < 1e-5 {
                (d + h) / abs_ux.max(1e-5)
            } else {
                let u_sum = abs_ux + abs_uy;
                let disc = d * d - h * h * (2.0 - u_sum * u_sum);
                let t_corner = h * u_sum + disc.max(0.0).sqrt();
                if t_corner * abs_ux >= h && t_corner * abs_uy >= h {
                    t_corner
                } else if t_corner * abs_uy < h {
                    (d + h) / abs_ux
                } else {
                    (d + h) / abs_uy
                }
            };

            let center = camp + u * t;
            let raw_origin = center - Vec2::splat(h);
            let origin = (raw_origin / cfg.block_size).round() * cfg.block_size;

            let mut score_sum = 0.0;
            for gy in 0..SAMPLE_GRID {
                for gx in 0..SAMPLE_GRID {
                    let sample = origin + Vec2::new(
                        (gx as f32 + 0.5) / (SAMPLE_GRID as f32) * size,
                        (gy as f32 + 0.5) / (SAMPLE_GRID as f32) * size,
                    );
                    score_sum += forest_at(sample);
                }
            }
            let score = score_sum / SAMPLES_COUNT;
            if score < best_score {
                best_score = score;
                best_origin = origin;
            }
        }

        let origin = best_origin;
        let min_chunk_x = (origin.x / cfg.block_size).round() as i32;
        let min_chunk_y = (origin.y / cfg.block_size).round() as i32;

        // Step 2: Blocks, ragged corner drops, and district assignment.
        let mut blocks = Vec::with_capacity((cfg.blocks * cfg.blocks) as usize);
        for by in 0..cfg.blocks {
            for bx in 0..cfg.blocks {
                let coord = IVec2::new(min_chunk_x + bx, min_chunk_y + by);
                let is_corner = (bx == 0 || bx == cfg.blocks - 1) && (by == 0 || by == cfg.blocks - 1);
                let present = if is_corner {
                    rng.next_double() >= cfg.corner_drop_chance as f64
                } else {
                    true
                };

                let ring = bx.min(cfg.blocks - 1 - bx).min(by.min(cfg.blocks - 1 - by));
                let core_start = (cfg.blocks - cfg.core_blocks) / 2;
                let core_end = core_start + cfg.core_blocks;

                let district = if bx >= core_start && bx < core_end && by >= core_start && by < core_end {
                    District::Core
                } else if ring < cfg.outskirts_rings {
                    District::Outskirts
                } else {
                    District::Downtown
                };

                blocks.push(CityBlock {
                    coord,
                    district,
                    present,
                });
            }
        }

        // Industrial strip along one side inside the outskirts rings.
        let industrial_side = rng.next(4);
        let r = cfg.outskirts_rings;
        let inner_max = cfg.blocks - r;
        let strip_w = cfg.industrial_strip[0];
        let strip_len = cfg.industrial_strip[1];

        for by in r..inner_max {
            for bx in r..inner_max {
                let is_industrial = match industrial_side {
                    0 => bx < r + strip_w && by < r + strip_len,
                    1 => bx >= inner_max - strip_w && by < r + strip_len,
                    2 => by < r + strip_w && bx < r + strip_len,
                    _ => by >= inner_max - strip_w && bx < r + strip_len,
                };
                if is_industrial {
                    let idx = (by * cfg.blocks + bx) as usize;
                    blocks[idx].district = District::Industrial;
                }
            }
        }

        // Parks from random Downtown blocks.
        let num_parks = if cfg.parks[0] == cfg.parks[1] {
            cfg.parks[0]
        } else {
            cfg.parks[0] + rng.next(cfg.parks[1] - cfg.parks[0] + 1)
        };

        let mut downtown_indices: Vec<usize> = blocks
            .iter()
            .enumerate()
            .filter(|(_, b)| b.present && b.district == District::Downtown)
            .map(|(i, _)| i)
            .collect();

        let park_count = (num_parks as usize).min(downtown_indices.len());
        for i in 0..park_count {
            let pick_offset = rng.next((downtown_indices.len() - i) as i32) as usize;
            let chosen_idx = i + pick_offset;
            downtown_indices.swap(i, chosen_idx);
            blocks[downtown_indices[i]].district = District::Park;
        }

        // Step 3: Streets and connectivity.
        let present_grid: Vec<bool> = blocks.iter().map(|b| b.present).collect();
        let (streets, entry) = generate_streets(origin, &present_grid, camp, cfg, &mut rng);

        // Step 4: Lots inside non-park blocks.
        let lots = generate_lots(origin, &blocks, cfg, &mut rng);

        CityLayout {
            origin,
            size,
            blocks,
            streets,
            lots,
            entry,
        }
    }

    /// True if point p falls within a present city block or on a city street surface.
    pub fn contains(&self, p: Vec2) -> bool {
        self.district_at(p).is_some() || self.street_at(p).is_some()
    }

    /// District of the present block at point p, or None if outside or dropped.
    pub fn district_at(&self, p: Vec2) -> Option<District> {
        let rel = p - self.origin;
        if rel.x < 0.0 || rel.y < 0.0 || rel.x >= self.size || rel.y >= self.size {
            return None;
        }
        let blocks_dim = (self.blocks.len() as f64).sqrt().round() as i32;
        if blocks_dim <= 0 {
            return None;
        }
        let block_size = self.size / blocks_dim as f32;
        let bx = (rel.x / block_size).floor() as i32;
        let by = (rel.y / block_size).floor() as i32;
        if bx < 0 || bx >= blocks_dim || by < 0 || by >= blocks_dim {
            return None;
        }
        let idx = (by * blocks_dim + bx) as usize;
        self.blocks.get(idx).and_then(|b| if b.present { Some(b.district) } else { None })
    }

    /// Street segment surface covering point p, if any.
    pub fn street_at(&self, p: Vec2) -> Option<&StreetSegment> {
        self.streets.iter().find(|s| s.contains_point(p))
    }
}
