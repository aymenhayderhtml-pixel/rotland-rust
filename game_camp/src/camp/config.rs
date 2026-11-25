//! Configuration and data loader for camp life (port of scripts/Settlement/CampLifeConfig.cs and CampLifeData.cs).

use std::path::Path;
use serde::{Deserialize, Serialize};

/// Shape of data/camp_life.json (Godot: CampLifeData.cs).
/// Points are [x, z] in camp-local metres (the settlement's own axes, centre at 0, 0).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampLifeData {
    #[serde(default = "default_resident_count", alias = "ResidentCount")]
    pub resident_count: usize,
    #[serde(default = "default_walk_speed", alias = "WalkSpeed")]
    pub walk_speed: f32,
    #[serde(default = "default_idle_min", alias = "IdleMinSeconds")]
    pub idle_min_seconds: f32,
    #[serde(default = "default_idle_max", alias = "IdleMaxSeconds")]
    pub idle_max_seconds: f32,
    #[serde(default = "default_chat_min", alias = "ChatMinSeconds")]
    pub chat_min_seconds: f32,
    #[serde(default = "default_chat_max", alias = "ChatMaxSeconds")]
    pub chat_max_seconds: f32,
    #[serde(default = "default_chat_swap_min", alias = "ChatSwapMinSeconds")]
    pub chat_swap_min_seconds: f32,
    #[serde(default = "default_chat_swap_max", alias = "ChatSwapMaxSeconds")]
    pub chat_swap_max_seconds: f32,
    #[serde(default = "default_chat_range", alias = "ChatRange")]
    pub chat_range: f32,
    #[serde(default = "default_chat_half_gap", alias = "ChatHalfGap")]
    pub chat_half_gap: f32,
    #[serde(default = "default_bubble_seconds", alias = "BubbleSeconds")]
    pub bubble_seconds: f32,
    #[serde(default = "default_bubble_fade", alias = "BubbleFadeSeconds")]
    pub bubble_fade_seconds: f32,
    #[serde(default = "default_bubble_range", alias = "BubbleRange")]
    pub bubble_range: f32,
    #[serde(default = "default_greet_range", alias = "GreetRange")]
    pub greet_range: f32,
    #[serde(default = "default_greet_seconds", alias = "GreetSeconds")]
    pub greet_seconds: f32,
    #[serde(default = "default_far_range", alias = "FarRange")]
    pub far_range: f32,
    #[serde(default = "default_fire_from", alias = "FireHourFrom")]
    pub fire_hour_from: f32,
    #[serde(default = "default_fire_to", alias = "FireHourTo")]
    pub fire_hour_to: f32,
    #[serde(default = "default_fire_seats", alias = "FireSeats")]
    pub fire_seats: usize,
    #[serde(default = "default_seat_radius", alias = "SeatRadius")]
    pub seat_radius: f32,
    #[serde(default = "default_day_sit_chance", alias = "DaySitChance")]
    pub day_sit_chance: f32,
    #[serde(default = "default_day_chat_chance", alias = "DayChatChance")]
    pub day_chat_chance: f32,
    #[serde(default = "default_day_work_chance", alias = "DayWorkChance")]
    pub day_work_chance: f32,
    #[serde(default = "default_fire_seconds", alias = "FireSeconds")]
    pub fire_seconds: [f32; 2],
    #[serde(default = "default_work_seconds", alias = "WorkSeconds")]
    pub work_seconds: [f32; 2],
    #[serde(default, alias = "WanderPoints")]
    pub wander_points: Vec<[f32; 2]>,
    #[serde(default, alias = "BedPoints")]
    pub bed_points: Vec<[f32; 2]>,
    #[serde(default = "default_chop_stand", alias = "ChopStand")]
    pub chop_stand: [f32; 2],
    #[serde(default = "default_chop_face", alias = "ChopFace")]
    pub chop_face: [f32; 2],
    #[serde(default, alias = "CarrySpots")]
    pub carry_spots: Vec<[f32; 2]>,
    #[serde(default, alias = "ChatLines")]
    pub chat_lines: Vec<String>,
}

fn default_resident_count() -> usize { 6 }
fn default_walk_speed() -> f32 { 1.2 }
fn default_idle_min() -> f32 { 3.0 }
fn default_idle_max() -> f32 { 8.0 }
fn default_chat_min() -> f32 { 15.0 }
fn default_chat_max() -> f32 { 30.0 }
fn default_chat_swap_min() -> f32 { 4.0 }
fn default_chat_swap_max() -> f32 { 6.0 }
fn default_chat_range() -> f32 { 15.0 }
fn default_chat_half_gap() -> f32 { 0.6 }
fn default_bubble_seconds() -> f32 { 4.0 }
fn default_bubble_fade() -> f32 { 0.6 }
fn default_bubble_range() -> f32 { 12.0 }
fn default_greet_range() -> f32 { 4.0 }
fn default_greet_seconds() -> f32 { 2.5 }
fn default_far_range() -> f32 { 60.0 }
fn default_fire_from() -> f32 { 19.0 }
fn default_fire_to() -> f32 { 6.0 }
fn default_fire_seats() -> usize { 4 }
fn default_seat_radius() -> f32 { 1.6 }
fn default_day_sit_chance() -> f32 { 0.1 }
fn default_day_chat_chance() -> f32 { 0.25 }
fn default_day_work_chance() -> f32 { 0.15 }
fn default_fire_seconds() -> [f32; 2] { [20.0, 40.0] }
fn default_work_seconds() -> [f32; 2] { [20.0, 40.0] }
fn default_chop_stand() -> [f32; 2] { [-13.0, 2.0] }
fn default_chop_face() -> [f32; 2] { [-16.0, 2.0] }

impl Default for CampLifeData {
    fn default() -> Self {
        Self {
            resident_count: default_resident_count(),
            walk_speed: default_walk_speed(),
            idle_min_seconds: default_idle_min(),
            idle_max_seconds: default_idle_max(),
            chat_min_seconds: default_chat_min(),
            chat_max_seconds: default_chat_max(),
            chat_swap_min_seconds: default_chat_swap_min(),
            chat_swap_max_seconds: default_chat_swap_max(),
            chat_range: default_chat_range(),
            chat_half_gap: default_chat_half_gap(),
            bubble_seconds: default_bubble_seconds(),
            bubble_fade_seconds: default_bubble_fade(),
            bubble_range: default_bubble_range(),
            greet_range: default_greet_range(),
            greet_seconds: default_greet_seconds(),
            far_range: default_far_range(),
            fire_hour_from: default_fire_from(),
            fire_hour_to: default_fire_to(),
            fire_seats: default_fire_seats(),
            seat_radius: default_seat_radius(),
            day_sit_chance: default_day_sit_chance(),
            day_chat_chance: default_day_chat_chance(),
            day_work_chance: default_day_work_chance(),
            fire_seconds: default_fire_seconds(),
            work_seconds: default_work_seconds(),
            wander_points: Vec::new(),
            bed_points: Vec::new(),
            chop_stand: default_chop_stand(),
            chop_face: default_chop_face(),
            carry_spots: Vec::new(),
            chat_lines: Vec::new(),
        }
    }
}

impl CampLifeData {
    /// Validates that points fall inside the safe zone and numbers are in range.
    // Godot: CampLifeConfig.cs Validate
    pub fn validate(&self) -> Result<(), String> {
        const LIMIT: f32 = 16.0;
        let ok = self.resident_count >= 2
            && self.walk_speed > 0.0
            && self.idle_min_seconds >= 0.0
            && self.idle_max_seconds >= self.idle_min_seconds
            && self.chat_max_seconds >= self.chat_min_seconds
            && self.chat_swap_max_seconds >= self.chat_swap_min_seconds
            && self.fire_seats >= 1
            && self.seat_radius > 0.0
            && self.fire_seconds.len() == 2
            && self.work_seconds.len() == 2
            && self.wander_points.len() >= self.resident_count
            && !self.bed_points.is_empty()
            && self.carry_spots.len() >= 2
            && self.chop_stand.len() == 2
            && self.chop_face.len() == 2
            && !self.chat_lines.is_empty();
        if !ok {
            return Err("camp_life data has values out of range".to_string());
        }

        let all_spots = self
            .wander_points
            .iter()
            .chain(self.bed_points.iter())
            .chain(self.carry_spots.iter())
            .chain(std::iter::once(&self.chop_stand));

        for p in all_spots {
            if p[0].abs() > LIMIT || p[1].abs() > LIMIT {
                return Err(format!("spot [{:.1}, {:.1}] is outside safe zone", p[0], p[1]));
            }
        }
        Ok(())
    }
}

/// Loads and validates data/camp_life.json (Godot: CampLifeConfig.cs).
pub struct CampLifeConfig;

impl CampLifeConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<CampLifeData, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let data: CampLifeData = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        data.validate()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(data)
    }

    /// Loads the real data/camp_life.json via paths::repo_path.
    pub fn load_default() -> Result<CampLifeData, String> {
        let path = paths::repo_path("data/camp_life.json");
        Self::load_from_file(&path).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn from_json(json: &str) -> Result<CampLifeData, serde_json::Error> {
        serde_json::from_str(json)
    }
}
