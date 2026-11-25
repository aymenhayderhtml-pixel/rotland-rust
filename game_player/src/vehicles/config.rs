use std::path::Path;
use serde::{Deserialize, Serialize};

/// Shape of data/vehicles.json (port of scripts/Vehicles/VehicleConfigData.cs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleConfigData {
    #[serde(default = "default_mass")]
    pub mass: f32,
    #[serde(default = "default_engine_force")]
    pub engine_force: f32,
    #[serde(default = "default_reverse_force")]
    pub reverse_force: f32,
    #[serde(default = "default_brake_force")]
    pub brake_force: f32,
    #[serde(default = "default_top_speed_kmh")]
    pub top_speed_kmh: f32,
    #[serde(default = "default_reverse_top_kmh")]
    pub reverse_top_kmh: f32,
    #[serde(default = "default_max_steer_rad")]
    pub max_steer_rad: f32,
    #[serde(default = "default_steer_speed_rad")]
    pub steer_speed_rad: f32,
    #[serde(default = "default_suspension_rest")]
    pub suspension_rest: f32,
    #[serde(default = "default_suspension_travel")]
    pub suspension_travel: f32,
    #[serde(default = "default_spring_stiffness")]
    pub spring_stiffness: f32,
    #[serde(default = "default_spring_damping")]
    pub spring_damping: f32,
    #[serde(default = "default_grip_mu")]
    pub grip_mu: f32,
    #[serde(default = "default_handbrake_grip")]
    pub handbrake_grip: f32,
    #[serde(default = "default_tank_litres")]
    pub tank_litres: f32,
    #[serde(default = "default_fuel_can_litres")]
    pub fuel_can_litres: f32,
    #[serde(default = "default_litres_per_metre_cruise")]
    pub litres_per_metre_cruise: f32,
    #[serde(default = "default_litres_per_metre_full_throttle")]
    pub litres_per_metre_full_throttle: f32,
    #[serde(default = "default_health")]
    pub health: f32,
    #[serde(default = "default_run_over_min_kmh")]
    pub run_over_min_kmh: f32,
    #[serde(default = "default_run_over_damage_per_kmh")]
    pub run_over_damage_per_kmh: f32,
    #[serde(default = "default_run_over_car_damage")]
    pub run_over_car_damage: f32,
    #[serde(default = "default_run_over_knockback_mps")]
    pub run_over_knockback_mps: f32,
    #[serde(default = "default_hit_half_width")]
    pub hit_half_width: f32,
    #[serde(default = "default_hit_front")]
    pub hit_front: f32,
    #[serde(default = "default_hit_back")]
    pub hit_back: f32,
    #[serde(default = "default_noise_radius_idle")]
    pub noise_radius_idle: f32,
    #[serde(default = "default_noise_radius_full")]
    pub noise_radius_full: f32,
    #[serde(default = "default_horn_radius")]
    pub horn_radius: f32,
    #[serde(default = "default_exit_max_kmh")]
    pub exit_max_kmh: f32,
    #[serde(default = "default_trunk_slots")]
    pub trunk_slots: usize,
    #[serde(default = "default_trunk_max_weight")]
    pub trunk_max_weight: f32,
    #[serde(default = "default_spawn_chance")]
    pub spawn_chance: f32,
    #[serde(default)]
    pub spawn_buildings: Vec<String>,
    #[serde(default)]
    pub spawn_fuel_min_frac: f32,
    #[serde(default = "default_spawn_fuel_max_frac")]
    pub spawn_fuel_max_frac: f32,
    #[serde(default = "default_camp_fuel_frac")]
    pub camp_fuel_frac: f32,
    #[serde(default = "default_camp_colour")]
    pub camp_colour: String,
    #[serde(default)]
    pub colours: Vec<String>,
    #[serde(default = "default_zombie_hit_damage")]
    pub zombie_hit_damage: f32,
    #[serde(default = "default_zombie_hit_seconds")]
    pub zombie_hit_seconds: f32,
    #[serde(default = "default_zombie_hit_reach")]
    pub zombie_hit_reach: f32,
    #[serde(default = "default_car_half_width")]
    pub car_half_width: f32,
    #[serde(default = "default_car_half_height")]
    pub car_half_height: f32,
    #[serde(default = "default_car_half_length")]
    pub car_half_length: f32,
}

fn default_mass() -> f32 { 1100.0 }
fn default_engine_force() -> f32 { 6500.0 }
fn default_reverse_force() -> f32 { 2000.0 }
fn default_brake_force() -> f32 { 6000.0 }
fn default_top_speed_kmh() -> f32 { 90.0 }
fn default_reverse_top_kmh() -> f32 { 25.0 }
fn default_max_steer_rad() -> f32 { 0.5 }
fn default_steer_speed_rad() -> f32 { 2.5 }
fn default_suspension_rest() -> f32 { 0.30 }
fn default_suspension_travel() -> f32 { 0.15 }
fn default_spring_stiffness() -> f32 { 27000.0 }
fn default_spring_damping() -> f32 { 3000.0 }
fn default_grip_mu() -> f32 { 1.1 }
fn default_handbrake_grip() -> f32 { 0.3 }
fn default_tank_litres() -> f32 { 40.0 }
fn default_fuel_can_litres() -> f32 { 10.0 }
fn default_litres_per_metre_cruise() -> f32 { 0.0005 }
fn default_litres_per_metre_full_throttle() -> f32 { 0.001 }
fn default_health() -> f32 { 300.0 }
fn default_run_over_min_kmh() -> f32 { 15.0 }
fn default_run_over_damage_per_kmh() -> f32 { 1.8 }
fn default_run_over_car_damage() -> f32 { 2.0 }
fn default_run_over_knockback_mps() -> f32 { 5.0 }
fn default_hit_half_width() -> f32 { 1.1 }
fn default_hit_front() -> f32 { 2.4 }
fn default_hit_back() -> f32 { 0.6 }
fn default_noise_radius_idle() -> f32 { 40.0 }
fn default_noise_radius_full() -> f32 { 80.0 }
fn default_horn_radius() -> f32 { 60.0 }
fn default_exit_max_kmh() -> f32 { 15.0 }
fn default_trunk_slots() -> usize { 20 }
fn default_trunk_max_weight() -> f32 { 150.0 }
fn default_spawn_chance() -> f32 { 0.35 }
fn default_spawn_fuel_max_frac() -> f32 { 0.3 }
fn default_camp_fuel_frac() -> f32 { 0.5 }
fn default_camp_colour() -> String { "red".to_string() }
fn default_zombie_hit_damage() -> f32 { 2.0 }
// data: vehicles.json zombieHitSeconds
fn default_zombie_hit_seconds() -> f32 { 1.2 }
// data: vehicles.json zombieHitReach
fn default_zombie_hit_reach() -> f32 { 1.5 }
// data: vehicles.json carHalfWidth
fn default_car_half_width() -> f32 { 0.8 }
// data: vehicles.json carHalfHeight
fn default_car_half_height() -> f32 { 1.2 }
// data: vehicles.json carHalfLength
fn default_car_half_length() -> f32 { 1.9 }

impl Default for VehicleConfigData {
    fn default() -> Self {
        Self {
            mass: default_mass(),
            engine_force: default_engine_force(),
            reverse_force: default_reverse_force(),
            brake_force: default_brake_force(),
            top_speed_kmh: default_top_speed_kmh(),
            reverse_top_kmh: default_reverse_top_kmh(),
            max_steer_rad: default_max_steer_rad(),
            steer_speed_rad: default_steer_speed_rad(),
            suspension_rest: default_suspension_rest(),
            suspension_travel: default_suspension_travel(),
            spring_stiffness: default_spring_stiffness(),
            spring_damping: default_spring_damping(),
            grip_mu: default_grip_mu(),
            handbrake_grip: default_handbrake_grip(),
            tank_litres: default_tank_litres(),
            fuel_can_litres: default_fuel_can_litres(),
            litres_per_metre_cruise: default_litres_per_metre_cruise(),
            litres_per_metre_full_throttle: default_litres_per_metre_full_throttle(),
            health: default_health(),
            run_over_min_kmh: default_run_over_min_kmh(),
            run_over_damage_per_kmh: default_run_over_damage_per_kmh(),
            run_over_car_damage: default_run_over_car_damage(),
            run_over_knockback_mps: default_run_over_knockback_mps(),
            hit_half_width: default_hit_half_width(),
            hit_front: default_hit_front(),
            hit_back: default_hit_back(),
            noise_radius_idle: default_noise_radius_idle(),
            noise_radius_full: default_noise_radius_full(),
            horn_radius: default_horn_radius(),
            exit_max_kmh: default_exit_max_kmh(),
            trunk_slots: default_trunk_slots(),
            trunk_max_weight: default_trunk_max_weight(),
            spawn_chance: default_spawn_chance(),
            spawn_buildings: vec!["gas_station".to_string(), "house_small".to_string()],
            spawn_fuel_min_frac: 0.0,
            spawn_fuel_max_frac: default_spawn_fuel_max_frac(),
            camp_fuel_frac: default_camp_fuel_frac(),
            camp_colour: default_camp_colour(),
            colours: vec!["red".to_string(), "blue".to_string()],
            zombie_hit_damage: default_zombie_hit_damage(),
            zombie_hit_seconds: default_zombie_hit_seconds(),
            zombie_hit_reach: default_zombie_hit_reach(),
            car_half_width: default_car_half_width(),
            car_half_height: default_car_half_height(),
            car_half_length: default_car_half_length(),
        }
    }
}

impl VehicleConfigData {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let data: Self = serde_json::from_str(json).map_err(|e| format!("Invalid vehicles.json: {e}"))?;
        data.validate()?;
        Ok(data)
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("Could not open {}: {e}", path.display()))?;
        Self::from_json(&text)
    }

    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/vehicles.json");
        Self::from_file(&path)
    }

    pub fn car_half_size(&self) -> glam::Vec3 {
        glam::Vec3::new(self.car_half_width, self.car_half_height, self.car_half_length)
    }

    // Godot: VehicleConfig.cs Validate / Problem
    pub fn validate(&self) -> Result<(), String> {
        let ok = self.mass > 0.0
            && self.engine_force > 0.0
            && self.brake_force > 0.0
            && self.top_speed_kmh > 0.0
            && self.max_steer_rad > 0.0
            && self.suspension_rest > 0.0
            && self.suspension_travel > 0.0
            && self.spring_stiffness > 0.0
            && self.tank_litres > 0.0
            && self.fuel_can_litres > 0.0
            && self.litres_per_metre_cruise > 0.0
            && self.health > 0.0
            && self.trunk_slots > 0
            && self.trunk_max_weight > 0.0
            && (0.0..=1.0).contains(&self.spawn_chance)
            && self.hit_half_width > 0.0
            && self.hit_front > 0.0;
        if !ok {
            return Err("vehicles.json has a value out of range".to_string());
        }
        for id in &self.spawn_buildings {
            if id.is_empty() {
                return Err("vehicles.json has an empty spawn building id".to_string());
            }
        }
        Ok(())
    }
}
