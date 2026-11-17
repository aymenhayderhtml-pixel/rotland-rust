//! Lifetime statistics for the player profile.
//! Port of `scripts/Player/PlayerStats.cs` (SaveId: "stats").

use std::collections::HashMap;

/// Maximum distance in metres a player can move in one physics frame before being considered a teleport.
// Godot: PlayerStats.cs MaxStepMetres
pub const STATS_MAX_STEP_METRES: f32 = 5.0;

/// Godot save system ID.
// Godot: PlayerStats.cs SaveId
pub const STATS_SAVE_ID: &str = "stats";

/// Godot load order.
// Godot: PlayerStats.cs LoadOrder
pub const STATS_LOAD_ORDER: i32 = 0;

/// Lifetime numbers for the hub's Profile tab.
/// Kills by type, headshots, deaths, highest day reached, play time and distance walked/driven.
#[derive(Clone, Debug, PartialEq)]
pub struct ProfileStats {
    pub kills: HashMap<String, u32>,
    pub headshots: u32,
    pub deaths: u32,
    pub highest_day: i32,
    pub play_time_seconds: f64,
    pub distance_walked: f32,
    pub distance_driven: f32,
    pub last_xz: Option<glam::Vec2>,
}

impl Default for ProfileStats {
    fn default() -> Self {
        Self {
            kills: HashMap::new(),
            headshots: 0,
            deaths: 0,
            highest_day: 1,
            play_time_seconds: 0.0,
            distance_walked: 0.0,
            distance_driven: 0.0,
            last_xz: None,
        }
    }
}

impl ProfileStats {
    /// Creates a new `ProfileStats` instance with default values (day 1, zero counters).
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a kill made by the player for a specific zombie type.
    // Godot: PlayerStats.cs RecordKill
    pub fn record_kill(&mut self, zombie_id: &str, headshot: bool) {
        *self.kills.entry(zombie_id.to_string()).or_insert(0) += 1;
        if headshot {
            self.headshots += 1;
        }
    }

    /// Convenience helper for recording an unknown/generic kill with headshot status.
    pub fn on_kill(&mut self, headshot: bool) {
        self.record_kill("unknown", headshot);
    }

    /// Records a death of the player.
    // Godot: PlayerStats.cs RecordDeath
    pub fn record_death(&mut self) {
        self.deaths += 1;
    }

    /// Convenience alias for `record_death`.
    pub fn on_death(&mut self) {
        self.record_death();
    }

    /// Advances the highest day reached by 1.
    pub fn on_day(&mut self) {
        self.highest_day = self.highest_day.max(1) + 1;
    }

    /// Updates the highest day reached if `day` is higher.
    // Godot: PlayerStats.cs RecordDay
    pub fn record_day(&mut self, day: i32) {
        self.highest_day = self.highest_day.max(day);
    }

    /// Adds play time in seconds (pausing stops the clock).
    // Godot: PlayerStats.cs AddPlayTime
    pub fn add_play_time(&mut self, seconds: f64) {
        if seconds > 0.0 {
            self.play_time_seconds += seconds;
        }
    }

    /// Adds walking distance in metres.
    // Godot: PlayerStats.cs AddDistance
    pub fn add_walked(&mut self, metres: f32) {
        if metres > 0.0 {
            self.distance_walked += metres;
        }
    }

    /// Adds driving distance in metres.
    pub fn add_driven(&mut self, metres: f32) {
        if metres > 0.0 {
            self.distance_driven += metres;
        }
    }

    /// Convenience alias for `add_walked`.
    pub fn add_distance(&mut self, metres: f32) {
        self.add_walked(metres);
    }

    /// Tracks walking distance from horizontal movement, filtering out jumps >= 5m as teleports.
    // Godot: PlayerStats.cs _PhysicsProcess
    pub fn update_position(&mut self, pos_xz: glam::Vec2) {
        if let Some(last) = self.last_xz {
            let dist = (pos_xz - last).length();
            if dist < STATS_MAX_STEP_METRES {
                self.add_walked(dist);
            }
        }
        self.last_xz = Some(pos_xz);
    }

    /// Resets the position tracker (e.g. after teleport or respawn).
    pub fn reset_position(&mut self) {
        self.last_xz = None;
    }

    /// Total kills across all zombie types.
    // Godot: PlayerStats.cs Kills
    pub fn kills(&self) -> u32 {
        self.kills.values().sum()
    }

    /// Kills breakdown by zombie type.
    // Godot: PlayerStats.cs KillsByType
    pub fn kills_by_type(&self) -> &HashMap<String, u32> {
        &self.kills
    }

    /// Total headshot kills.
    // Godot: PlayerStats.cs Headshots
    pub fn headshots(&self) -> u32 {
        self.headshots
    }

    /// Total player deaths.
    // Godot: PlayerStats.cs Deaths
    pub fn deaths(&self) -> u32 {
        self.deaths
    }

    /// The highest day reached.
    // Godot: PlayerStats.cs HighestDay
    pub fn highest_day(&self) -> i32 {
        self.highest_day
    }

    /// Full days after day 1 that the player has survived through.
    // Godot: PlayerStats.cs DaysSurvived
    pub fn days_survived(&self) -> i32 {
        (self.highest_day - 1).max(0)
    }

    /// Total play time in seconds.
    // Godot: PlayerStats.cs PlayTimeSeconds
    pub fn play_time_seconds(&self) -> f64 {
        self.play_time_seconds
    }

    /// Total play time in seconds (convenience alias).
    pub fn play_time(&self) -> f64 {
        self.play_time_seconds
    }

    /// Total distance in metres (walked + driven).
    // Godot: PlayerStats.cs DistanceMetres
    pub fn distance_metres(&self) -> f32 {
        self.distance_walked + self.distance_driven
    }

    /// Distance walked in metres.
    pub fn distance_walked(&self) -> f32 {
        self.distance_walked
    }

    /// Distance driven in metres.
    pub fn distance_driven(&self) -> f32 {
        self.distance_driven
    }

    /// Saves stats to JSON using exact Godot save keys.
    // Godot: PlayerStats.cs Save
    pub fn to_save(&self) -> serde_json::Value {
        let mut kills_map = serde_json::Map::new();
        for (k, v) in &self.kills {
            kills_map.insert(k.clone(), serde_json::Value::from(*v));
        }
        serde_json::json!({
            "kills": kills_map,
            "headshots": self.headshots,
            "deaths": self.deaths,
            "highestDay": self.highest_day,
            "playTime": self.play_time_seconds,
            "distance": self.distance_metres(),
        })
    }

    /// Loads stats from JSON using exact Godot save keys.
    // Godot: PlayerStats.cs Load
    pub fn from_save(&mut self, data: &serde_json::Value) {
        let Some(obj) = data.as_object() else {
            return;
        };

        if let Some(kills) = obj.get("kills").and_then(|v| v.as_object()) {
            self.kills.clear();
            for (k, v) in kills {
                let count = v
                    .as_i64()
                    .map(|n| n.max(0) as u32)
                    .or_else(|| v.as_u64().map(|n| n as u32))
                    .unwrap_or(0);
                self.kills.insert(k.clone(), count);
            }
        }

        if let Some(hs) = obj.get("headshots")
            && let Some(val) = hs.as_i64().map(|n| n.max(0) as u32).or_else(|| hs.as_u64().map(|n| n as u32)) {
                self.headshots = val;
            }

        if let Some(d) = obj.get("deaths")
            && let Some(val) = d.as_i64().map(|n| n.max(0) as u32).or_else(|| d.as_u64().map(|n| n as u32)) {
                self.deaths = val;
            }

        if let Some(hd) = obj.get("highestDay")
            && let Some(val) = hd.as_i64().map(|n| (n as i32).max(1)) {
                self.highest_day = val;
            }

        if let Some(pt) = obj.get("playTime")
            && let Some(val) = pt.as_f64().map(|n| n.max(0.0)) {
                self.play_time_seconds = val;
            }

        if let Some(dist) = obj.get("distance")
            && let Some(val) = dist.as_f64().map(|n| (n as f32).max(0.0)) {
                self.distance_walked = val;
            }

        if let Some(driven) = obj.get("distanceDriven")
            && let Some(val) = driven.as_f64().map(|n| (n as f32).max(0.0)) {
                self.distance_driven = val;
            }
    }

    /// Constructs a `ProfileStats` instance from a saved JSON value.
    pub fn from_save_value(data: &serde_json::Value) -> Self {
        let mut stats = Self::default();
        stats.from_save(data);
        stats
    }
}
