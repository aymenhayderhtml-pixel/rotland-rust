//! Configuration and species data parsed from `data/animals.json`.
// Godot: AnimalConfig.cs, AnimalConfigData.cs, AnimalSpeciesData.cs, AnimalBehaviourData.cs, AnimalSpawnData.cs, AnimalButcherData.cs

use std::collections::HashMap;
use std::path::Path;
use serde::Deserialize;

fn default_body_radius() -> f32 { 0.35 }
fn default_body_height() -> f32 { 1.0 }
fn default_walk_speed() -> f32 { 1.0 }
fn default_run_speed() -> f32 { 6.0 }
fn default_detect_range() -> f32 { 12.0 }
fn default_herd() -> Vec<usize> { vec![1, 1] }
fn default_charge_speed() -> f32 { 9.0 }
fn default_attack_range() -> f32 { 1.8 }
fn default_attack_damage() -> f32 { 20.0 }
fn default_attack_cooldown() -> f32 { 1.5 }
fn default_attack_knockback() -> f32 { 6.0 }
fn default_fight_range() -> f32 { 2.0 }
fn default_fight_damage() -> f32 { 15.0 }

/// One wild species from the "species" block of data/animals.json.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimalSpeciesData {
    pub name: String,
    pub model: String,
    #[serde(default)]
    pub buck_model: Option<String>,
    #[serde(default)]
    pub buck_chance: f32,
    #[serde(default = "default_body_radius")]
    pub health: f32,
    #[serde(default = "default_body_radius")]
    pub body_radius: f32,
    #[serde(default = "default_body_height")]
    pub body_height: f32,
    #[serde(default = "default_walk_speed")]
    pub walk_speed: f32,
    #[serde(default = "default_run_speed")]
    pub run_speed: f32,
    #[serde(default = "default_detect_range")]
    pub detect_range: f32,
    #[serde(default = "default_herd")]
    pub herd: Vec<usize>,
    #[serde(default)]
    pub zigzag: bool,
    #[serde(default)]
    pub danger: bool,
    #[serde(default = "default_charge_speed")]
    pub charge_speed: f32,
    #[serde(default = "default_attack_range")]
    pub attack_range: f32,
    #[serde(default = "default_attack_damage")]
    pub attack_damage: f32,
    #[serde(default = "default_attack_cooldown")]
    pub attack_cooldown: f32,
    #[serde(default = "default_attack_knockback")]
    pub attack_knockback: f32,
    #[serde(default = "default_fight_range")]
    pub fight_range: f32,
    #[serde(default = "default_fight_damage")]
    pub fight_damage: f32,
    #[serde(default)]
    pub xp: u32,
    #[serde(default)]
    pub meat: u32,
    #[serde(default)]
    pub hide: u32,
    #[serde(default)]
    pub antler: u32,
    #[serde(default)]
    pub alert_sound: Option<String>,
    #[serde(default)]
    pub attack_sound: Option<String>,
}

/// The "spawn" block of data/animals.json.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimalSpawnData {
    #[serde(default = "default_start_delay")]
    pub start_delay: f32,
    #[serde(default = "default_interval")]
    pub interval: f32,
    #[serde(default = "default_min_dist")]
    pub min_dist: f32,
    #[serde(default = "default_max_dist")]
    pub max_dist: f32,
    #[serde(default = "default_despawn_dist")]
    pub despawn_dist: f32,
    #[serde(default = "default_out_of_view_angle_deg")]
    pub out_of_view_angle_deg: f32,
    #[serde(default = "default_day_cap")]
    pub day_cap: usize,
    #[serde(default = "default_night_cap")]
    pub night_cap: usize,
    #[serde(default)]
    pub biomes: HashMap<String, Vec<String>>,
}

fn default_start_delay() -> f32 { 10.0 }
fn default_interval() -> f32 { 4.0 }
fn default_min_dist() -> f32 { 40.0 }
fn default_max_dist() -> f32 { 120.0 }
fn default_despawn_dist() -> f32 { 150.0 }
fn default_out_of_view_angle_deg() -> f32 { 75.0 }
fn default_day_cap() -> usize { 6 }
fn default_night_cap() -> usize { 3 }

impl Default for AnimalSpawnData {
    fn default() -> Self {
        Self {
            start_delay: 10.0,
            interval: 4.0,
            min_dist: 40.0,
            max_dist: 120.0,
            despawn_dist: 150.0,
            out_of_view_angle_deg: 75.0,
            day_cap: 6,
            night_cap: 3,
            biomes: HashMap::new(),
        }
    }
}

/// The "behaviour" block of data/animals.json.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimalBehaviourData {
    #[serde(default = "default_crouch_factor")]
    pub crouch_factor: f32,
    #[serde(default = "default_sprint_factor")]
    pub sprint_factor: f32,
    #[serde(default = "default_min_stealth_factor")]
    pub min_stealth_factor: f32,
    #[serde(default = "default_sight_front_deg")]
    pub sight_front_deg: f32,
    #[serde(default = "default_close_sense_dist")]
    pub close_sense_dist: f32,
    #[serde(default = "default_gunshot_noise")]
    pub gunshot_noise: f32,
    #[serde(default = "default_gunshot_range")]
    pub gunshot_range: f32,
    #[serde(default = "default_alert_seconds")]
    pub alert_seconds: f32,
    #[serde(default = "default_flee_seconds")]
    pub flee_seconds: f32,
    #[serde(default = "default_flee_distance")]
    pub flee_distance: f32,
    #[serde(default = "default_zigzag_deg")]
    pub zigzag_deg: f32,
    #[serde(default = "default_zigzag_seconds")]
    pub zigzag_seconds: f32,
    #[serde(default = "default_graze_min")]
    pub graze_min: f32,
    #[serde(default = "default_graze_max")]
    pub graze_max: f32,
    #[serde(default = "default_wander_radius")]
    pub wander_radius: f32,
    #[serde(default = "default_boar_leash")]
    pub boar_leash: f32,
}

fn default_crouch_factor() -> f32 { 0.5 }
fn default_sprint_factor() -> f32 { 1.5 }
fn default_min_stealth_factor() -> f32 { 0.6 }
fn default_sight_front_deg() -> f32 { 90.0 }
fn default_close_sense_dist() -> f32 { 3.0 }
fn default_gunshot_noise() -> f32 { 30.0 }
fn default_gunshot_range() -> f32 { 60.0 }
fn default_alert_seconds() -> f32 { 1.5 }
fn default_flee_seconds() -> f32 { 8.0 }
fn default_flee_distance() -> f32 { 60.0 }
fn default_zigzag_deg() -> f32 { 40.0 }
fn default_zigzag_seconds() -> f32 { 0.5 }
fn default_graze_min() -> f32 { 4.0 }
fn default_graze_max() -> f32 { 8.0 }
fn default_wander_radius() -> f32 { 6.0 }
fn default_boar_leash() -> f32 { 25.0 }

impl Default for AnimalBehaviourData {
    fn default() -> Self {
        Self {
            crouch_factor: 0.5,
            sprint_factor: 1.5,
            min_stealth_factor: 0.6,
            sight_front_deg: 90.0,
            close_sense_dist: 3.0,
            gunshot_noise: 30.0,
            gunshot_range: 60.0,
            alert_seconds: 1.5,
            flee_seconds: 8.0,
            flee_distance: 60.0,
            zigzag_deg: 40.0,
            zigzag_seconds: 0.5,
            graze_min: 4.0,
            graze_max: 8.0,
            wander_radius: 6.0,
            boar_leash: 25.0,
        }
    }
}

/// The "butcher" block of data/animals.json.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimalButcherData {
    #[serde(default = "default_seconds")]
    pub seconds: f32,
    #[serde(default = "default_tool_speed_factor")]
    pub tool_speed_factor: f32,
    #[serde(default = "default_tool_ids")]
    pub tool_ids: Vec<String>,
    #[serde(default = "default_tool_slots")]
    pub tool_slots: usize,
    #[serde(default = "default_range")]
    pub range: f32,
    #[serde(default = "default_give_up_range")]
    pub give_up_range: f32,
    #[serde(default = "default_xp")]
    pub xp: u32,
    #[serde(default = "default_extra_meat_levels")]
    pub extra_meat_levels: Vec<u32>,
    #[serde(default = "default_body_seconds")]
    pub body_seconds: f32,
}

fn default_seconds() -> f32 { 3.0 }
fn default_tool_speed_factor() -> f32 { 2.0 }
fn default_tool_ids() -> Vec<String> { vec!["axe".into(), "knife".into(), "machete".into()] }
fn default_tool_slots() -> usize { 6 }
fn default_range() -> f32 { 2.5 }
fn default_give_up_range() -> f32 { 3.5 }
fn default_xp() -> u32 { 5 }
fn default_extra_meat_levels() -> Vec<u32> { vec![3, 6] }
fn default_body_seconds() -> f32 { 300.0 }

impl Default for AnimalButcherData {
    fn default() -> Self {
        Self {
            seconds: 3.0,
            tool_speed_factor: 2.0,
            tool_ids: vec!["axe".into(), "knife".into(), "machete".into()],
            tool_slots: 6,
            range: 2.5,
            give_up_range: 3.5,
            xp: 5,
            extra_meat_levels: vec![3, 6],
            body_seconds: 300.0,
        }
    }
}

/// Root data shape of data/animals.json.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct AnimalConfigData {
    #[serde(default)]
    pub spawn: AnimalSpawnData,
    #[serde(default)]
    pub behaviour: AnimalBehaviourData,
    #[serde(default)]
    pub butcher: AnimalButcherData,
    #[serde(default)]
    pub species: HashMap<String, AnimalSpeciesData>,
}

impl AnimalConfigData {
    pub fn from_json(text: &str) -> Result<Self, String> {
        let data: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        data.validate()?;
        Ok(data)
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("Could not open {}: {e}", path.display()))?;
        Self::from_json(&text)
    }

    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/animals.json");
        Self::from_file(&path)
    }

    pub fn species(&self, id: &str) -> Option<&AnimalSpeciesData> {
        self.species.get(id)
    }

    // Godot: AnimalConfig.cs Validate
    pub fn validate(&self) -> Result<(), String> {
        let s = &self.spawn;
        if s.interval <= 0.0
            || s.min_dist < 0.0
            || s.max_dist < s.min_dist
            || s.despawn_dist <= s.max_dist
            || s.biomes.is_empty()
        {
            return Err("animals.json spawn block has a value out of range".into());
        }
        if self.species.is_empty() {
            return Err("animals.json needs at least one species".into());
        }
        for (biome, ids) in &s.biomes {
            for id in ids {
                if !self.species.contains_key(id) {
                    return Err(format!("animals.json biome '{biome}' lists unknown species '{id}'"));
                }
            }
        }
        for (id, sp) in &self.species {
            if sp.health <= 0.0
                || sp.walk_speed <= 0.0
                || sp.run_speed < sp.walk_speed
                || sp.detect_range <= 0.0
                || sp.model.is_empty()
                || sp.herd.len() != 2
                || sp.herd[0] < 1
                || sp.herd[1] < sp.herd[0]
            {
                return Err(format!("animals.json species '{id}' has a value out of range"));
            }
            if sp.danger && (sp.attack_range <= 0.0 || sp.charge_speed <= 0.0 || sp.attack_cooldown <= 0.0) {
                return Err(format!("animals.json species '{id}' is dangerous but has no attack numbers"));
            }
        }
        if self.butcher.seconds <= 0.0
            || self.butcher.tool_speed_factor < 1.0
            || self.butcher.give_up_range < self.butcher.range
        {
            return Err("animals.json butcher block has a value out of range".into());
        }
        Ok(())
    }
}


