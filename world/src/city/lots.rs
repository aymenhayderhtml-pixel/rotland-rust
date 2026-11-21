use glam::Vec2;
use crate::city::config::{CityConfigData, District};
use crate::random::DotnetRandom;

// new: DECISIONS 2026-11-21
const MAX_ATTEMPTS: usize = 20;

/// A placed building lot.
#[derive(Debug, Clone, PartialEq)]
pub struct CityLot {
    pub building_id: String,
    pub center: Vec2,
    pub size: Vec2,
    pub yaw: f32,
    pub district: District,
}

impl CityLot {
    /// Axis-aligned bounding box (min, max).
    /// When facing South (yaw ~ 0) or North (yaw ~ PI), size.x is width along X, size.y is depth along Y.
    /// When facing West or East, size.x is width along Y, size.y is depth along X.
    pub fn aabb(&self) -> (Vec2, Vec2) {
        let (half_x, half_y) = if self.yaw.cos().abs() > 0.5 {
            (self.size.x * 0.5, self.size.y * 0.5)
        } else {
            (self.size.y * 0.5, self.size.x * 0.5)
        };
        (
            self.center - Vec2::new(half_x, half_y),
            self.center + Vec2::new(half_x, half_y),
        )
    }
}

/// Generates lots along the edges of all non-park present blocks.
pub(crate) fn generate_lots(
    origin: Vec2,
    blocks: &[crate::city::layout::CityBlock],
    cfg: &CityConfigData,
    rng: &mut DotnetRandom,
) -> Vec<CityLot> {
    let mut lots = Vec::new();
    let n = cfg.blocks;

    for (idx, block) in blocks.iter().enumerate() {
        if !block.present || block.district == District::Park {
            continue;
        }
        let bx = (idx as i32) % n;
        let by = (idx as i32) / n;

        let block_min = origin + Vec2::new(bx as f32 * cfg.block_size, by as f32 * cfg.block_size);
        let block_max = block_min + Vec2::splat(cfg.block_size);

        let w_south = if by % cfg.avenue_every == 0 {
            cfg.avenue_width
        } else {
            cfg.street_width
        };
        let w_north = if (by + 1) % cfg.avenue_every == 0 {
            cfg.avenue_width
        } else {
            cfg.street_width
        };
        let w_west = if bx % cfg.avenue_every == 0 {
            cfg.avenue_width
        } else {
            cfg.street_width
        };
        let w_east = if (bx + 1) % cfg.avenue_every == 0 {
            cfg.avenue_width
        } else {
            cfg.street_width
        };

        let usable_min = Vec2::new(
            block_min.x + w_west * 0.5 + cfg.lot_gap,
            block_min.y + w_south * 0.5 + cfg.lot_gap,
        );
        let usable_max = Vec2::new(
            block_max.x - w_east * 0.5 - cfg.lot_gap,
            block_max.y - w_north * 0.5 - cfg.lot_gap,
        );

        if usable_max.x <= usable_min.x || usable_max.y <= usable_min.y {
            continue;
        }

        let weights = cfg.districts.weights_for(block.district);
        if weights.is_empty() {
            continue;
        }
        let total_weight: u32 = weights.iter().map(|w| w.weight).sum();
        if total_weight == 0 {
            continue;
        }

        let mut block_lots: Vec<CityLot> = Vec::new();
        let mut failures = 0;

        while failures < MAX_ATTEMPTS {
            let mut pick = rng.next(total_weight as i32) as u32;
            let mut building_id = &weights[0].id;
            for bw in weights {
                if pick < bw.weight {
                    building_id = &bw.id;
                    break;
                }
                pick -= bw.weight;
            }

            let size_pair = match cfg.buildings.get(building_id) {
                Some(s) => *s,
                None => {
                    failures += 1;
                    continue;
                }
            };
            let width = size_pair[0];
            let depth = size_pair[1];

            let start_side = rng.next(4);
            let mut placed = false;

            for side_offset in 0..4 {
                let side = (start_side + side_offset) % 4;
                match side {
                    0 => {
                        // South side (faces -Y, yaw = 0)
                        let yaw = 0.0;
                        let half_x = width * 0.5;
                        let half_y = depth * 0.5;
                        let lot_y = usable_min.y + half_y;
                        if lot_y + half_y > usable_max.y {
                            continue;
                        }
                        let min_x = usable_min.x + half_x;
                        let max_x = usable_max.x - half_x;
                        if min_x > max_x {
                            continue;
                        }

                        let mut candidates = vec![min_x];
                        for ex in &block_lots {
                            let (_, ex_max) = ex.aabb();
                            let c_x = ex_max.x + cfg.lot_gap + half_x;
                            if c_x >= min_x && c_x <= max_x {
                                candidates.push(c_x);
                            }
                        }

                        for cx in candidates {
                            let cand_min = Vec2::new(cx - half_x, usable_min.y);
                            let cand_max = Vec2::new(cx + half_x, usable_min.y + depth);
                            if !lot_collides(cand_min, cand_max, &block_lots, cfg.lot_gap) {
                                block_lots.push(CityLot {
                                    building_id: building_id.clone(),
                                    center: Vec2::new(cx, lot_y),
                                    size: Vec2::new(width, depth),
                                    yaw,
                                    district: block.district,
                                });
                                placed = true;
                                break;
                            }
                        }
                    }
                    1 => {
                        // North side (faces +Y, yaw = PI)
                        let yaw = std::f32::consts::PI;
                        let half_x = width * 0.5;
                        let half_y = depth * 0.5;
                        let lot_y = usable_max.y - half_y;
                        if lot_y - half_y < usable_min.y {
                            continue;
                        }
                        let min_x = usable_min.x + half_x;
                        let max_x = usable_max.x - half_x;
                        if min_x > max_x {
                            continue;
                        }

                        let mut candidates = vec![min_x];
                        for ex in &block_lots {
                            let (_, ex_max) = ex.aabb();
                            let c_x = ex_max.x + cfg.lot_gap + half_x;
                            if c_x >= min_x && c_x <= max_x {
                                candidates.push(c_x);
                            }
                        }

                        for cx in candidates {
                            let cand_min = Vec2::new(cx - half_x, usable_max.y - depth);
                            let cand_max = Vec2::new(cx + half_x, usable_max.y);
                            if !lot_collides(cand_min, cand_max, &block_lots, cfg.lot_gap) {
                                block_lots.push(CityLot {
                                    building_id: building_id.clone(),
                                    center: Vec2::new(cx, lot_y),
                                    size: Vec2::new(width, depth),
                                    yaw,
                                    district: block.district,
                                });
                                placed = true;
                                break;
                            }
                        }
                    }
                    2 => {
                        // West side (faces -X, yaw = PI/2)
                        let yaw = std::f32::consts::FRAC_PI_2;
                        let half_x = depth * 0.5;
                        let half_y = width * 0.5;
                        let lot_x = usable_min.x + half_x;
                        if lot_x + half_x > usable_max.x {
                            continue;
                        }
                        let min_y = usable_min.y + half_y;
                        let max_y = usable_max.y - half_y;
                        if min_y > max_y {
                            continue;
                        }

                        let mut candidates = vec![min_y];
                        for ex in &block_lots {
                            let (_, ex_max) = ex.aabb();
                            let c_y = ex_max.y + cfg.lot_gap + half_y;
                            if c_y >= min_y && c_y <= max_y {
                                candidates.push(c_y);
                            }
                        }

                        for cy in candidates {
                            let cand_min = Vec2::new(usable_min.x, cy - half_y);
                            let cand_max = Vec2::new(usable_min.x + depth, cy + half_y);
                            if !lot_collides(cand_min, cand_max, &block_lots, cfg.lot_gap) {
                                block_lots.push(CityLot {
                                    building_id: building_id.clone(),
                                    center: Vec2::new(lot_x, cy),
                                    size: Vec2::new(width, depth),
                                    yaw,
                                    district: block.district,
                                });
                                placed = true;
                                break;
                            }
                        }
                    }
                    _ => {
                        // East side (faces +X, yaw = -PI/2)
                        let yaw = -std::f32::consts::FRAC_PI_2;
                        let half_x = depth * 0.5;
                        let half_y = width * 0.5;
                        let lot_x = usable_max.x - half_x;
                        if lot_x - half_x < usable_min.x {
                            continue;
                        }
                        let min_y = usable_min.y + half_y;
                        let max_y = usable_max.y - half_y;
                        if min_y > max_y {
                            continue;
                        }

                        let mut candidates = vec![min_y];
                        for ex in &block_lots {
                            let (_, ex_max) = ex.aabb();
                            let c_y = ex_max.y + cfg.lot_gap + half_y;
                            if c_y >= min_y && c_y <= max_y {
                                candidates.push(c_y);
                            }
                        }

                        for cy in candidates {
                            let cand_min = Vec2::new(usable_max.x - depth, cy - half_y);
                            let cand_max = Vec2::new(usable_max.x, cy + half_y);
                            if !lot_collides(cand_min, cand_max, &block_lots, cfg.lot_gap) {
                                block_lots.push(CityLot {
                                    building_id: building_id.clone(),
                                    center: Vec2::new(lot_x, cy),
                                    size: Vec2::new(width, depth),
                                    yaw,
                                    district: block.district,
                                });
                                placed = true;
                                break;
                            }
                        }
                    }
                }
                if placed {
                    break;
                }
            }

            if placed {
                failures = 0;
            } else {
                failures += 1;
            }
        }

        lots.extend(block_lots);
    }

    lots
}

fn lot_collides(cand_min: Vec2, cand_max: Vec2, existing: &[CityLot], lot_gap: f32) -> bool {
    for ex in existing {
        let (ex_min, ex_max) = ex.aabb();
        let overlap_x = cand_min.x - ex_max.x < lot_gap && ex_min.x - cand_max.x < lot_gap;
        let overlap_y = cand_min.y - ex_max.y < lot_gap && ex_min.y - cand_max.y < lot_gap;
        if overlap_x && overlap_y {
            return true;
        }
    }
    false
}
