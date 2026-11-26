//! Parses `data/metro.json` (New Halden) and checks it before the city is built.

use std::collections::HashMap;
use std::path::Path;

use glam::Vec2;
use serde::Deserialize;

use crate::metro::layout::Reserved;

fn default_reference_dir() -> Vec2 {
    Vec2::X
}

/// Ring sizes of the city, counted in blocks from the outer edge inward.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistrictsConfig {
    pub edge_rings: i32,
    pub midtown_rings: i32,
    pub core_blocks: i32,
}

/// A rectangle of blocks (from block `ix`, `iz`, `w` wide, `h` high) held for one reserved kind.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ReservedArea {
    pub kind: Reserved,
    pub ix: i32,
    pub iz: i32,
    pub w: i32,
    pub h: i32,
}

/// Tower floors (min, max) and footprint (min, max metres) for one district.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DistrictTowers {
    pub floors: [u32; 2],
    pub footprint: [f32; 2],
}

/// Tower heights, footprints and landmark counts (`towers` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TowersConfig {
    pub edge: DistrictTowers,
    pub midtown: DistrictTowers,
    pub core: DistrictTowers,
    pub floor_height: f32,
    pub landmark_count: [usize; 2],
    pub landmark_floors: [u32; 2],
    pub lot_gap: f32,
    pub max_lots_per_block: usize,
}

/// The wide highway from the nearest town into the city (`highway` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HighwayConfig {
    pub highway_width: f32,
    pub median_width: f32,
    pub lanes: u32,
    pub shoulder_width: f32,
    pub height_smooth_m: f32,
    pub flatten_edge_m: f32,
    pub town_setback_m: f32,
    /// Length of the highway's last stretch that blends into the flat city ground.
    pub city_blend_m: f32,
}

/// A model id with its relative pick weight.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WeightedModel {
    pub id: String,
    pub weight: u32,
}

/// The evacuation traffic jam on the highway (`jam` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JamConfig {
    pub jam_length: f32,
    pub car_spacing_m: [f32; 2],
    pub car_models: Vec<WeightedModel>,
    pub bus_count: [usize; 2],
    pub army_truck_count: [usize; 2],
    pub body_chance: f32,
    pub luggage_chance: f32,
    pub burnt_chance: f32,
    pub inbound_chance: f32,
    pub avenue_jam_chance: f32,
    pub shoulder_body_chance: f32,
    /// Width of one traffic lane in metres (`laneWidthM`).
    pub lane_width_m: f32,
    /// Maximum random yaw turn of a jammed car, in degrees (`yawJitterDeg`).
    pub yaw_jitter_deg: f32,
    /// How far luggage sits outward from a car's centre line, metres (`luggageOffsetM`).
    pub luggage_offset_m: [f32; 2],
    /// Chance that a luggage spot also has clothes on the asphalt (`clothesSpillChance`).
    pub clothes_spill_chance: f32,
    /// Seat points per car model id, in model-local metres (R38-05).
    #[serde(default)]
    pub seat_offsets: HashMap<String, Vec<[f32; 3]>>,
}

/// Far skyline settings (`skyline` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkylineConfig {
    pub skyline_distance: f32,
    pub tris_per_tower: usize,
}

/// Everything about New Halden, read from `data/metro.json`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetroConfig {
    pub key: String,
    pub name: String,
    pub center_distance: f32,
    pub angle_jitter_deg: f32,
    pub size_blocks: i32,
    pub block_size: f32,
    pub street_width: f32,
    pub avenue_width: f32,
    pub avenue_every: i32,
    pub corner_drop_chance: f32,
    pub skirt_width: f32,
    pub districts: DistrictsConfig,
    pub reserved: Vec<ReservedArea>,
    pub towers: TowersConfig,
    pub highway: HighwayConfig,
    pub jam: JamConfig,
    pub skyline: SkylineConfig,
    /// Camp-to-Ashford direction the sector angles turn from. Set by the lead after
    /// `WorldLayout::reference_dir()` is known; not stored in the JSON.
    #[serde(skip, default = "default_reference_dir")]
    pub reference_dir: Vec2,
}

impl MetroConfig {
    /// Checks every field that the layout code relies on.
    pub fn validate(&self) -> Result<(), String> {
        let positive = [
            self.center_distance,
            self.block_size,
            self.street_width,
            self.avenue_width,
            self.towers.floor_height,
            self.towers.lot_gap,
            self.highway.highway_width,
            self.jam.jam_length,
            self.skyline.skyline_distance,
        ];
        if positive.iter().any(|v| *v <= 0.0) {
            return Err("metro.json: distances, widths and heights must be positive".into());
        }
        if self.size_blocks <= 0 || self.avenue_every <= 0 || self.skirt_width < 0.0 {
            return Err("metro.json: sizeBlocks, avenueEvery must be positive; skirtWidth >= 0".into());
        }
        if !(0.0..=1.0).contains(&self.corner_drop_chance) {
            return Err("metro.json: cornerDropChance must be within 0..1".into());
        }
        self.validate_districts()?;
        self.validate_reserved()?;
        for (name, t) in [
            ("edge", &self.towers.edge),
            ("midtown", &self.towers.midtown),
            ("core", &self.towers.core),
        ] {
            if t.floors[0] > t.floors[1] || t.footprint[0] > t.footprint[1] || t.footprint[0] <= 0.0 {
                return Err(format!("metro.json: towers.{name} ranges are reversed or empty"));
            }
        }
        if self.towers.landmark_count[0] > self.towers.landmark_count[1]
            || self.towers.landmark_floors[0] > self.towers.landmark_floors[1]
            || self.towers.max_lots_per_block == 0
        {
            return Err("metro.json: towers landmark or lot ranges are invalid".into());
        }
        if self.highway.lanes == 0 || self.jam.car_models.is_empty() {
            return Err("metro.json: highway needs lanes and jam needs car models".into());
        }
        Ok(())
    }

    fn validate_districts(&self) -> Result<(), String> {
        let d = &self.districts;
        if d.edge_rings < 0 || d.midtown_rings < 0 || d.core_blocks <= 0 {
            return Err("metro.json: district ring sizes must be >= 0 (core >= 1)".into());
        }
        let total = d.core_blocks + 2 * d.midtown_rings + 2 * d.edge_rings;
        if total != self.size_blocks {
            return Err(format!(
                "metro.json: districts add up to {total} blocks but sizeBlocks is {}",
                self.size_blocks
            ));
        }
        Ok(())
    }

    fn validate_reserved(&self) -> Result<(), String> {
        let n = self.size_blocks;
        let mut occupied = vec![false; (n * n) as usize];
        for kind in Reserved::ALL {
            if !self.reserved.iter().any(|a| a.kind == kind) {
                return Err(format!("metro.json: reserved kind {kind:?} is missing"));
            }
        }
        for area in &self.reserved {
            let inside = area.w >= 1
                && area.h >= 1
                && area.ix >= 0
                && area.iz >= 0
                && area.ix + area.w <= n
                && area.iz + area.h <= n;
            if !inside {
                return Err(format!("metro.json: reserved {:?} is outside the grid", area.kind));
            }
            for z in area.iz..area.iz + area.h {
                for x in area.ix..area.ix + area.w {
                    let cell = &mut occupied[(z * n + x) as usize];
                    if *cell {
                        return Err(format!("metro.json: reserved {:?} overlaps another area", area.kind));
                    }
                    *cell = true;
                }
            }
        }
        Ok(())
    }

    /// Parses and validates a `metro.json` string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let cfg: Self = serde_json::from_str(json).map_err(|e| format!("metro.json: {e}"))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Reads and validates a `metro.json` file.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load() -> MetroConfig {
        let path = paths::repo_path("data/metro.json");
        MetroConfig::load_from_file(path).expect("data/metro.json must load")
    }

    #[test]
    fn loads_every_top_level_key_and_all_reserved_kinds() {
        let cfg = load();
        assert_eq!(cfg.key, "new_halden");
        assert_eq!(cfg.name, "New Halden");
        assert_eq!(cfg.size_blocks, 32);
        assert_eq!(cfg.block_size, 64.0);
        assert_eq!(cfg.districts.core_blocks, 10);
        assert_eq!(cfg.reserved.len(), 8);
        for kind in Reserved::ALL {
            assert!(cfg.reserved.iter().any(|a| a.kind == kind), "missing {kind:?}");
        }
        assert_eq!(cfg.reference_dir, Vec2::X, "reference_dir is runtime-only");
    }

    #[test]
    fn rejects_district_sum_mismatch() {
        let mut cfg = load();
        cfg.districts.core_blocks += 1;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_overlapping_reserved_areas() {
        let mut cfg = load();
        cfg.reserved[1].ix = cfg.reserved[0].ix;
        cfg.reserved[1].iz = cfg.reserved[0].iz;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_missing_reserved_kind() {
        let mut cfg = load();
        cfg.reserved.remove(0);
        assert!(cfg.validate().is_err());
    }
}
