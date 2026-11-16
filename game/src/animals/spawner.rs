//! Animal spawning logic matching AnimalSpawner.cs.
// Godot: AnimalSpawner.cs

use std::f32::consts::TAU;
use glam::{Quat, Vec3};
use physics::PhysicsWorld;
use super::config::{AnimalConfigData, AnimalSpeciesData};
use super::rng::AnimalRng;

pub const MAX_SLOPE_DEG: f32 = 35.0;
pub const SLOPE_PROBE_DIST: f32 = 1.0;
pub const FLOOR_OFFSET: f32 = 0.05;
pub const HERD_SPACING: f32 = 2.5;
pub const SPAWN_ATTEMPTS_PER_TICK: usize = 12;

/// Attempt to find valid ground coordinates for spawning.
// Godot: AnimalSpawner.cs TryGroundAt
pub fn try_ground_at(
    point: Vec3,
    physics: &PhysicsWorld,
    in_no_spawn_zone: impl Fn(Vec3) -> bool,
) -> Option<Vec3> {
    if in_no_spawn_zone(point) {
        return None;
    }
    let ground_y = physics.ground_height(point.x, point.z).unwrap_or(point.y);
    let slope = slope_deg(physics, point.x, point.z, ground_y);
    if slope > MAX_SLOPE_DEG {
        return None;
    }
    Some(Vec3::new(point.x, ground_y + FLOOR_OFFSET, point.z))
}

// Godot: AnimalSpawner.cs SlopeDeg
pub fn slope_deg(physics: &PhysicsWorld, x: f32, z: f32, fallback_y: f32) -> f32 {
    let h_xp = physics.ground_height(x + SLOPE_PROBE_DIST, z).unwrap_or(fallback_y);
    let h_xm = physics.ground_height(x - SLOPE_PROBE_DIST, z).unwrap_or(fallback_y);
    let h_zp = physics.ground_height(x, z + SLOPE_PROBE_DIST).unwrap_or(fallback_y);
    let h_zm = physics.ground_height(x, z - SLOPE_PROBE_DIST).unwrap_or(fallback_y);
    let dx = (h_xp - h_xm) / (2.0 * SLOPE_PROBE_DIST);
    let dz = (h_zp - h_zm) / (2.0 * SLOPE_PROBE_DIST);
    (dx * dx + dz * dz).sqrt().atan().to_degrees()
}

/// Spawns a herd of wild animals around the player out of view.
// Godot: AnimalSpawner.cs TrySpawnHerd
pub fn pick_spawn_spot(
    player_pos: Vec3,
    player_forward: Vec3,
    config: &AnimalConfigData,
    biome_name: &str,
    rng: &mut AnimalRng,
    physics: &PhysicsWorld,
    in_no_spawn_zone: impl Fn(Vec3) -> bool,
) -> Option<(String, Vec3)> {
    let spawn = &config.spawn;
    let biome = biome_name.to_lowercase();
    let species_list = spawn.biomes.get(&biome)?;
    if species_list.is_empty() {
        return None;
    }

    let origin = player_pos;
    let mut forward = Vec3::new(player_forward.x, 0.0, player_forward.z);
    if forward.length_squared() < 0.0001 {
        forward = Vec3::NEG_Z;
    } else {
        forward = forward.normalize();
    }

    for _ in 0..SPAWN_ATTEMPTS_PER_TICK {
        let angle = rng.gen_range_f32(spawn.out_of_view_angle_deg, 180.0)
            * (if rng.next_f32() < 0.5 { -1.0 } else { 1.0 });
        let dist = rng.gen_range_f32(spawn.min_dist, spawn.max_dist);
        let rot = Quat::from_rotation_y(angle.to_radians());
        let point = origin + (rot * forward) * dist;

        if let Some(feet) = try_ground_at(point, physics, &in_no_spawn_zone) {
            let pick_idx = rng.gen_range_usize(0, species_list.len() - 1);
            let species_id = species_list[pick_idx].clone();
            return Some((species_id, feet));
        }
    }
    None
}

/// Computes spawn points and model variants for a herd of the given species.
// Godot: AnimalSpawner.cs SpawnHerdOf
pub fn plan_herd(
    _species_id: &str,
    feet: Vec3,
    room: usize,
    sp: &AnimalSpeciesData,
    rng: &mut AnimalRng,
    physics: &PhysicsWorld,
    in_no_spawn_zone: impl Fn(Vec3) -> bool,
) -> Vec<(Vec3, String)> {
    let herd_min = sp.herd[0];
    let herd_max = sp.herd[1];
    let size = rng.gen_range_usize(herd_min, herd_max).min(room);
    let mut herd = Vec::new();

    for i in 0..size {
        let mut at = feet;
        if i > 0 {
            let around = rng.gen_range_f32(0.0, TAU);
            let offset = Vec3::new(around.cos() * HERD_SPACING, 0.0, around.sin() * HERD_SPACING);
            if let Some(ground) = try_ground_at(feet + offset, physics, &in_no_spawn_zone) {
                at = ground;
            }
        }
        let model = if let Some(buck) = &sp.buck_model
            && rng.next_f32() < sp.buck_chance
        {
            buck.clone()
        } else {
            sp.model.clone()
        };
        herd.push((at, model));
    }
    herd
}
