use glam::{IVec2, Quat, Vec2, Vec3};
use world::DotnetRandom;

use crate::vehicles::config::VehicleConfigData;

pub const CAMP_CAR_ID: &str = "car_camp";
pub const SPAWN_SALT: i32 = 0x3C1A7D25;
pub const COLOUR_SALT: i32 = 0x0B17E5A3;
pub const FUEL_SALT: i32 = 0x5E2D9C41;
pub const PARKING_GAP: f32 = 3.5;

/// Stable FNV-1a hash over text, matching Godot C# CarSpawnRules.cs.
pub fn stable_hash(text: &str) -> u32 {
    let mut hash = 2166136261u32;
    for c in text.chars() {
        hash ^= c as u32;
        hash = hash.wrapping_mul(16777619u32);
    }
    hash
}

pub fn poi_car_id(poi_id: &str) -> String {
    format!("car_{poi_id}")
}

/// A deterministic number from 0 to 1 for this seed, id and salt.
pub fn roll(seed: i32, id: &str, salt: i32) -> f64 {
    let roll_seed = seed ^ (stable_hash(id) as i32) ^ salt;
    let mut rng = DotnetRandom::new(roll_seed);
    rng.next_double()
}

pub fn wants_car(seed: i32, poi_id: &str, building_id: &str, cfg: &VehicleConfigData) -> bool {
    cfg.spawn_buildings.iter().any(|b| b == building_id)
        && roll(seed, poi_id, SPAWN_SALT) < cfg.spawn_chance as f64
}

pub fn colour_for(seed: i32, id: &str, cfg: &VehicleConfigData) -> String {
    if cfg.colours.is_empty() {
        return "red".to_string();
    }
    let r = roll(seed, id, COLOUR_SALT);
    let index = (r * (cfg.colours.len() as f64)) as usize;
    cfg.colours[index.min(cfg.colours.len() - 1)].clone()
}

pub fn fuel_for(seed: i32, id: &str, cfg: &VehicleConfigData) -> f32 {
    let frac = cfg.spawn_fuel_min_frac
        + (cfg.spawn_fuel_max_frac - cfg.spawn_fuel_min_frac) * (roll(seed, id, FUEL_SALT) as f32);
    frac * cfg.tank_litres
}

/// Where a car parks beside a building: outside its east wall, facing along the wall.
pub fn poi_parking_xz(poi_pos: Vec3, poi_yaw: f32, def_half_width: f32) -> Vec2 {
    let local_x = def_half_width + PARKING_GAP;
    let c = poi_yaw.cos();
    let s = poi_yaw.sin();
    Vec2::new(poi_pos.x + local_x * c, poi_pos.z - local_x * s)
}

pub fn chunk_of(at: Vec3, chunk_size: f32) -> IVec2 {
    IVec2::new(
        (at.x / chunk_size).floor() as i32,
        (at.z / chunk_size).floor() as i32,
    )
}

pub fn new_record(
    id: &str,
    colour: &str,
    at: Vec3,
    yaw: f32,
    fuel: f32,
    health: f32,
) -> serde_json::Value {
    let q = Quat::from_rotation_y(yaw);
    serde_json::json!({
        "id": id,
        "colour": colour,
        "x": at.x,
        "y": at.y,
        "z": at.z,
        "qx": q.x,
        "qy": q.y,
        "qz": q.z,
        "qw": q.w,
        "fuel": fuel,
        "health": health,
        "trunk": {
            "stacks": []
        }
    })
}

/// Namespace struct mirroring CarSpawnRules.cs.
pub struct CarSpawnRules;

impl CarSpawnRules {
    pub const CAMP_CAR_ID: &'static str = CAMP_CAR_ID;

    pub fn stable_hash(text: &str) -> u32 {
        stable_hash(text)
    }

    pub fn poi_car_id(poi_id: &str) -> String {
        poi_car_id(poi_id)
    }

    pub fn roll(seed: i32, id: &str, salt: i32) -> f64 {
        roll(seed, id, salt)
    }

    pub fn wants_car(seed: i32, poi_id: &str, building_id: &str, cfg: &VehicleConfigData) -> bool {
        wants_car(seed, poi_id, building_id, cfg)
    }

    pub fn colour_for(seed: i32, id: &str, cfg: &VehicleConfigData) -> String {
        colour_for(seed, id, cfg)
    }

    pub fn fuel_for(seed: i32, id: &str, cfg: &VehicleConfigData) -> f32 {
        fuel_for(seed, id, cfg)
    }

    pub fn poi_parking_xz(poi_pos: Vec3, poi_yaw: f32, def_half_width: f32) -> Vec2 {
        poi_parking_xz(poi_pos, poi_yaw, def_half_width)
    }

    pub fn chunk_of(at: Vec3, chunk_size: f32) -> IVec2 {
        chunk_of(at, chunk_size)
    }

    pub fn new_record(
        id: &str,
        colour: &str,
        at: Vec3,
        yaw: f32,
        fuel: f32,
        health: f32,
    ) -> serde_json::Value {
        new_record(id, colour, at, yaw, fuel, health)
    }
}
