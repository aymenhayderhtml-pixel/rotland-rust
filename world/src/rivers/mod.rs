pub mod carve;
pub mod mesh;
pub mod path;

use std::path::Path;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::biome::layout::WorldLayout;
use crate::rivers::carve::{SegmentGrid, carve_point, find_crossings, water_level_at};
use crate::rivers::mesh::build_river_water_mesh_near;
use crate::rivers::path::trace_river;
use crate::roads::RoadNetwork;

// new: DECISIONS 2026-11-25
const RIVER_SEED_STRIDE: i32 = 131;
// data: rivers.json max_turn_deg
const DEFAULT_MAX_TURN_DEG: f32 = 12.0;

fn default_max_turn_deg() -> f32 {
    DEFAULT_MAX_TURN_DEG
}

/// Starting sector and downhill target direction for one river in `data/rivers.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiverSeedSpec {
    pub name: String,
    pub start_angle_deg: f32,
    pub start_radius_m: f32,
    pub target_angle_deg: f32,
    pub target_radius_m: f32,
}

/// River pathing, carving, mesh, and bridge configuration loaded from `data/rivers.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiversData {
    pub step_m: f32,
    pub max_length_m: f32,
    pub core_margin_m: f32,
    pub sample_count: usize,
    pub sample_spread_deg: f32,
    #[serde(default = "default_max_turn_deg")]
    pub max_turn_deg: f32,
    pub wiggle_deg: f32,
    pub wiggle_frequency: f32,
    pub target_pull: f32,
    pub width_start_m: f32,
    pub width_end_m: f32,
    pub depth_m: f32,
    pub bed_depth_factor: f32,
    pub bank_m: f32,
    pub mesh_bank_overlap_m: f32,
    pub grid_cell_m: f32,
    pub bridge_spans_m: Vec<f32>,
    pub crossing_merge_dist_m: f32,
    pub rivers: Vec<RiverSeedSpec>,
}

impl RiversData {
    /// Loads `data/rivers.json` via `paths::try_repo_path`.
    pub fn load() -> Result<Self, String> {
        let path = paths::try_repo_path("data/rivers.json")
            .map_err(|e| format!("data/rivers.json: {e}"))?;
        Self::load_from_file(path)
    }

    /// Loads river configuration from an explicit file path.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&json)
    }

    /// Parses and validates river configuration from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let data: Self = serde_json::from_str(json)
            .map_err(|e| format!("failed to parse rivers JSON: {e}"))?;
        if data.step_m <= 0.0 || data.max_length_m <= 0.0 {
            return Err("rivers.json step_m and max_length_m must be positive".to_string());
        }
        if data.sample_count == 0 {
            return Err("rivers.json sample_count must be at least 1".to_string());
        }
        if data.max_turn_deg <= 0.0 {
            return Err("rivers.json max_turn_deg must be positive".to_string());
        }
        if data.grid_cell_m <= 0.0 {
            return Err("rivers.json grid_cell_m must be positive".to_string());
        }
        if data.bridge_spans_m.is_empty() {
            return Err("rivers.json bridge_spans_m must not be empty".to_string());
        }
        Ok(data)
    }
}

/// One traced river polyline with per-vertex channel width and monotonic water surface height.
#[derive(Debug, Clone, PartialEq)]
pub struct River {
    pub points: Vec<Vec2>,
    pub widths: Vec<f32>,
    pub water_levels: Vec<f32>,
}

/// Generated outer-world river network with spatial indexing for carving, water queries, meshes, and crossings.
#[derive(Debug, Clone)]
pub struct RiverNetwork {
    data: RiversData,
    rivers: Vec<River>,
    grid: SegmentGrid,
}

impl RiverNetwork {
    /// Traces all rivers defined in `data` outside the preserved core using `base_height`.
    pub fn build(
        data: &RiversData,
        layout: &WorldLayout,
        seed: i32,
        origin: Vec2,
        base_height: &dyn Fn(f32, f32) -> f32,
    ) -> RiverNetwork {
        let rivers: Vec<River> = data
            .rivers
            .iter()
            .enumerate()
            .map(|(idx, spec)| {
                let river_seed = seed.wrapping_add((idx as i32 + 1).wrapping_mul(RIVER_SEED_STRIDE));
                trace_river(spec, data, layout, river_seed, origin, base_height)
            })
            .collect();
        Self::from_rivers(data, rivers)
    }

    /// Builds an indexed river network from pre-constructed rivers.
    pub fn from_rivers(data: &RiversData, rivers: Vec<River>) -> RiverNetwork {
        let grid = SegmentGrid::build(&rivers, data.bank_m, data.grid_cell_m);
        RiverNetwork {
            data: data.clone(),
            rivers,
            grid,
        }
    }

    pub fn rivers(&self) -> &[River] {
        &self.rivers
    }

    pub fn data(&self) -> &RiversData {
        &self.data
    }

    /// Returns `(bed_height, blend)` within `width/2 + bank_m` of any river, else `None`.
    pub fn carve(&self, x: f32, z: f32) -> Option<(f32, f32)> {
        carve_point(&self.rivers, &self.grid, &self.data, Vec2::new(x, z))
    }

    /// Returns the river water surface level when `(x, z)` lies over water, else `None`.
    pub fn water_at(&self, x: f32, z: f32) -> Option<f32> {
        water_level_at(&self.rivers, &self.grid, Vec2::new(x, z))
    }

    /// Builds a non-indexed `+Y`-facing triangle list for river surfaces within `radius` of `center`.
    pub fn water_mesh_near(&self, center: Vec2, radius: f32) -> Vec<Vec3> {
        build_river_water_mesh_near(&self.rivers, self.data.mesh_bank_overlap_m, center, radius)
            .to_triangle_list()
    }

    /// Finds every road-river crossing as `(position, road_heading_yaw_rad, bridge_span_m)`.
    ///
    /// `road_heading_yaw_rad` uses the Godot yaw convention where forward is `(-sin yaw, 0, -cos yaw)`.
    pub fn crossings(&self, roads: &RoadNetwork) -> Vec<(Vec2, f32, f32)> {
        let polylines: Vec<&[Vec2]> = roads.roads().iter().map(|r| r.points.as_slice()).collect();
        self.crossings_for_polylines(&polylines)
    }

    /// Finds every crossing between arbitrary XZ road polylines and this river network.
    pub fn crossings_for_polylines(&self, polylines: &[&[Vec2]]) -> Vec<(Vec2, f32, f32)> {
        find_crossings(&self.rivers, &self.grid, &self.data, polylines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biome::layout::WorldLayoutData;

    fn hilly_terrain(x: f32, z: f32) -> f32 {
        let slope = 60.0 - 0.004 * x - 0.002 * z;
        let hills =
            6.0 * (x * 0.003).sin() * (z * 0.003).cos() + 3.0 * ((x + z) * 0.007).sin();
        slope + hills
    }

    fn cross2(a: Vec2, b: Vec2) -> f32 {
        a.x * b.y - a.y * b.x
    }

    fn segments_intersect(a0: Vec2, a1: Vec2, b0: Vec2, b1: Vec2) -> bool {
        let da = a1 - a0;
        let db = b1 - b0;
        let denom = cross2(da, db);
        if denom.abs() <= 1e-6 {
            return false;
        }
        let diff = b0 - a0;
        let t = cross2(diff, db) / denom;
        let u = cross2(diff, da) / denom;
        (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
    }

    #[test]
    fn seed_42_rivers_are_long_reach_target_never_self_cross_and_never_rise() {
        let data = RiversData::load().expect("load data/rivers.json");
        let layout_data = WorldLayoutData::load().expect("load data/world_layout.json");
        let layout = WorldLayout::new(&layout_data, 42, Vec2::ZERO, Vec2::X);
        let net = RiverNetwork::build(&data, &layout, 42, Vec2::ZERO, &hilly_terrain);

        assert_eq!(net.rivers().len(), data.rivers.len());

        for (spec, river) in data.rivers.iter().zip(net.rivers().iter()) {
            let total_len: f32 = river
                .points
                .windows(2)
                .map(|w| (w[1] - w[0]).length())
                .sum();
            assert!(
                total_len >= 2500.0,
                "{} too short: {total_len} m",
                spec.name
            );

            let target_pos = layout.direction_at_deg(spec.target_angle_deg) * spec.target_radius_m;
            let end_pos = *river.points.last().expect("river has points");
            let end_dist = (end_pos - target_pos).length();
            assert!(
                end_dist <= 100.0,
                "{} ended {end_dist} m from target",
                spec.name
            );

            let seg_count = river.points.len().saturating_sub(1);
            for i in 0..seg_count {
                for j in (i + 2)..seg_count {
                    assert!(
                        !segments_intersect(
                            river.points[i],
                            river.points[i + 1],
                            river.points[j],
                            river.points[j + 1],
                        ),
                        "{} self-crossed between segments {i} and {j}",
                        spec.name
                    );
                }
            }

            for (i, w) in river.water_levels.windows(2).enumerate() {
                assert!(
                    w[1] <= w[0] + 1e-5,
                    "{} water rose at {i}: {} -> {}",
                    spec.name,
                    w[0],
                    w[1]
                );
            }
        }
    }
}
