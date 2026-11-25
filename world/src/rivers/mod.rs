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
