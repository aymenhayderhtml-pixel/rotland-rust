//! Configuration and validation for New Halden survivor traces (`traces` in `data/metro.json`).

use std::sync::OnceLock;

use serde::Deserialize;

/// Settings for street-end car barricades (`traces.carWalls`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CarWallsConfig {
    pub count: [usize; 2],
    pub min_district: String,
    pub in_m: [f32; 2],
}

impl Default for CarWallsConfig {
    fn default() -> Self {
        Self {
            count: [2, 3], // data: metro.json traces
            min_district: "edge".into(), // data: metro.json traces
            in_m: [20.0, 40.0], // data: metro.json traces
        }
    }
}

/// One candidate interior dressing piece for a tower lobby (`traces.lobbyDressing`).
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LobbyDressingEntry {
    pub model: String,
    pub chance: f32,
    pub y: f32,
    pub x: f32,
    pub z: f32,
    pub yaw: f32,
}

/// Survivor trace settings (`traces` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TraceConfig {
    pub car_walls: CarWallsConfig,
    pub wall_width_m: f32,
    pub gap_m: f32,
    pub wall_offset_m: f32,
    pub graffiti_lobby_chance: f32,
    pub graffiti_street_wall_chance: f32,
    pub graffiti_height_m: [f32; 2],
    pub poster_chance: f32,
    pub poster_height_m: [f32; 2],
    pub poster_x_share: [f32; 2],
    pub camp_chance: f32,
    pub camp_floors: [u32; 2],
    pub camp_size_m: f32,
    pub camp_margin_m: f32,
    pub body_sidewalk_chance: f32,
    pub lobby_ceiling_y: f32,
    pub lobby_dressing: Vec<LobbyDressingEntry>,
}

impl Default for TraceConfig {
    fn default() -> Self {
        let e = |model: &str, chance: f32, y: f32, x: f32, z: f32, yaw: f32| LobbyDressingEntry {
            model: model.into(),
            chance,
            y,
            x,
            z,
            yaw,
        };
        Self {
            car_walls: CarWallsConfig::default(),
            wall_width_m: 10.0, // data: metro.json wallWidthM
            gap_m: 2.0,
            wall_offset_m: 0.02, // data: metro.json traces
            graffiti_lobby_chance: 0.35, // data: metro.json traces
            graffiti_street_wall_chance: 0.12, // data: metro.json traces
            graffiti_height_m: [1.0, 3.0],
            poster_chance: 0.3, // data: metro.json traces
            poster_height_m: [1.2, 2.0], // data: metro.json traces
            poster_x_share: [0.3, 0.4], // data: metro.json traces
            camp_chance: 0.15, // data: metro.json traces
            camp_floors: [4, 8], // data: metro.json traces
            camp_size_m: 6.0, // data: metro.json traces
            camp_margin_m: 1.0,
            body_sidewalk_chance: 0.35, // data: metro.json traces
            lobby_ceiling_y: 7.3, // data: metro.json traces
            lobby_dressing: vec![
                e("lobby_column", 0.65, 0.1, -0.22, 0.18, 0.0), // data: metro.json traces
                e("lobby_column", 0.65, 0.1, 0.22, -0.18, 0.0), // data: metro.json traces
                e("lobby_ceiling_hang", 0.45, 5.8, 0.0, -0.10, 0.2), // data: metro.json traces
                e("lobby_light_broken", 0.50, 5.6, 0.12, 0.08, -0.3), // data: metro.json traces
                e("lobby_rug", 0.55, 0.1, 0.0, -0.22, 0.0), // data: metro.json traces
                e("lobby_chair_tipped", 0.40, 0.1, 0.16, -0.15, 1.1), // data: metro.json traces
                e("lobby_debris", 0.50, 0.1, -0.14, -0.08, 0.7), // data: metro.json traces
                e("body_sheet_b", 0.18, 0.1, 0.20, -0.24, 0.5), // data: metro.json traces
            ],
        }
    }
}

impl TraceConfig {
    /// Checks all chances, ranges and lobby bounds.
    pub fn validate(&self) -> Result<(), String> {
        let chances = [
            ("graffitiLobbyChance", self.graffiti_lobby_chance),
            ("graffitiStreetWallChance", self.graffiti_street_wall_chance),
            ("posterChance", self.poster_chance),
            ("campChance", self.camp_chance),
            ("bodySidewalkChance", self.body_sidewalk_chance),
        ];
        for (name, c) in chances {
            if !(0.0..=1.0).contains(&c) {
                return Err(format!("metro.json traces.{name} ({c}) must be within 0..1"));
            }
        }
        if self.car_walls.count[0] > self.car_walls.count[1]
            || self.car_walls.in_m[0] <= 0.0
            || self.car_walls.in_m[0] > self.car_walls.in_m[1]
            || !matches!(self.car_walls.min_district.as_str(), "edge" | "midtown" | "core")
        {
            return Err("metro.json traces.carWalls is invalid".into());
        }
        if self.wall_width_m <= 0.0
            || self.gap_m <= 0.0
            || self.wall_offset_m <= 0.0
            || self.graffiti_height_m[0] < 0.0
            || self.graffiti_height_m[0] > self.graffiti_height_m[1]
            || self.poster_height_m[0] < 0.0
            || self.poster_height_m[0] > self.poster_height_m[1]
            || self.poster_x_share[0] <= 0.0
            || self.poster_x_share[0] > self.poster_x_share[1]
            || self.poster_x_share[1] >= 0.5
            || self.camp_floors[0] > self.camp_floors[1]
            || self.camp_size_m <= 0.0
            || self.camp_margin_m < 0.0
            || self.lobby_ceiling_y <= 0.0
        {
            return Err("metro.json traces: dimensions or ranges are invalid".into());
        }
        for e in &self.lobby_dressing {
            if e.model.trim().is_empty()
                || !(0.0..=1.0).contains(&e.chance)
                || e.y < 0.0
                || e.y > self.lobby_ceiling_y
                || e.x.abs() >= 0.5
                // new: DECISIONS 2026-11-26 (keep lobby props clear of front door at z < 0.3)
                || e.z >= 0.3
                || e.z <= -0.5
            {
                return Err(format!("metro.json traces.lobbyDressing entry '{}' is invalid", e.model));
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct TracesWrapper {
    traces: TraceConfig,
}

/// Parses and validates the top-level `traces` object from a `metro.json` string.
pub fn load_traces(json: &str) -> Result<TraceConfig, String> {
    let wrapper: TracesWrapper =
        serde_json::from_str(json).map_err(|e| format!("metro.json traces: {e}"))?;
    wrapper.traces.validate()?;
    Ok(wrapper.traces)
}

/// Cached `TraceConfig` loaded from `data/metro.json` (uses `TraceConfig::default()` if the
/// `"traces"` key is not yet in the file).
pub fn default_traces() -> &'static TraceConfig {
    static CACHE: OnceLock<TraceConfig> = OnceLock::new();
    CACHE.get_or_init(|| {
        let path = paths::repo_path("data/metro.json");
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let root: serde_json::Value = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));
        if root.get("traces").is_none() {
            return TraceConfig::default();
        }
        load_traces(&json).expect("data/metro.json traces must be valid")
    })
}
