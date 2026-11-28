//! World survivor rescue wiring: initialization, save/load, E interaction, and POI zombie top-up.

use game::guild::GuildConfig;
use game::guild::companion::CompanionSquad;
use game::rescue::{RescueBlocker, RescueConfig, RescueSave, RescueState};
use game::rescue_floor::{FloorRescueSeed, floor_rescue_spots};
use game::rescue_place::place_survivors_in_world;
use game::{PoiSpawn, PoiZombieSite, PoiZombies};
use glam::Vec3;
use physics::PhysicsWorld;
use world::WorldGenerator;

use crate::play::PlayState;
use crate::zombies::Zombies;

/// Ring radius (m) for extra POI zombies when a building has fewer than `minZombiesNear` markers.
const EXTRA_ZOMBIE_RING_M: f32 = 3.5; // new: DECISIONS 2026-11-25

/// Result of checking/handling survivor interaction for one frame.
pub struct RescueInteract {
    pub prompt: Option<String>,
    pub distance: f32,
    pub consumed_press: bool,
}

/// Builds `RescueState` from `data/rescue.json` + `data/guild.json` and restores any saved `rescued_ids`.
pub fn load_state(world_gen: &WorldGenerator, save_file: Option<&save::SaveFile>) -> Option<RescueState> {
    let cfg = match RescueConfig::from_repo() {
        Ok(cfg) => cfg,
        Err(e) => {
            log::warn!("data/rescue.json not loaded, rescue survivors off: {e}");
            return None;
        }
    };
    let recruits = match GuildConfig::from_repo() {
        Ok(g) => g.recruits,
        Err(e) => {
            log::warn!("data/guild.json not loaded for rescue names, using fallback: {e}");
            Vec::new()
        }
    };
    let mut spots = place_survivors_in_world(world_gen, &cfg, &recruits);
    // R44-04: survivors on the stop floors of the climb towers (the random pass skips those POIs)
    if let Some(metro) = world_gen.metro_world() {
        let seeds: Vec<FloorRescueSeed> = world::metro::safehouse::rescue_floor_seeds(metro)
            .into_iter()
            .map(|s| FloorRescueSeed { poi_id: s.poi_id, building_id: s.building_id, position: s.position, yaw: s.yaw })
            .collect();
        spots.extend(floor_rescue_spots(&seeds, &cfg, &recruits));
    }
    let mut state = RescueState::new(cfg, spots);
    if let Some(guild_val) = save_file.and_then(|f| f.systems.extra.get("guild"))
        && let Ok(gs) = serde_json::from_value::<RescueSave>(guild_val.clone())
    {
        state.restore_rescued(&gs.rescued_ids);
    }
    Some(state)
}

/// Writes `rescued_ids` into the serialized `"guild"` system entry on save.
pub fn save_rescued_ids(rescue: Option<&RescueState>, file: &mut save::SaveFile) {
    let Some(rescue) = rescue else {
        return;
    };
    let ids = rescue.rescued_ids_sorted();
    if let Some(serde_json::Value::Object(map)) = file.systems.extra.get_mut("guild")
        && let Ok(val) = serde_json::to_value(ids)
    {
        map.insert("rescued_ids".into(), val);
    }
}

/// Shows `"[E] Rescue <name>"` / `"Team full"` when in range with no living zombies near, and rescues on E.
pub fn interact_tick(
    rescue: &mut RescueState,
    player_pos: Vec3,
    zombies: &Zombies,
    play_state: &mut PlayState,
    companions: Option<&mut CompanionSquad>,
    pressed: bool,
) -> Option<RescueInteract> {
    rescue.nearest_in_range(player_pos)?;
    let living: Vec<Vec3> = zombies
        .list
        .iter()
        .filter(|z| !z.health.is_dead())
        .map(|z| z.position)
        .collect();
    let max_recruits = play_state.camp.guild.manager().max_recruits();
    let hired_count = play_state.camp.guild.manager().recruits().hired().len();
    let (spot_pos, prompt_text) = {
        let (spot, text) = rescue.prompt_for(player_pos, &living, hired_count, max_recruits)?;
        (spot.position, text)
    };
    let distance = spot_pos.distance(player_pos);
    if !pressed {
        return Some(RescueInteract {
            prompt: Some(prompt_text),
            distance,
            consumed_press: false,
        });
    }
    let recruits = play_state.camp.guild.manager_mut().recruits_mut();
    match rescue.try_rescue_nearest(player_pos, &living, recruits, max_recruits) {
        Ok(recruit) => {
            if let Some(squad) = companions {
                squad.set_spawn_position(&recruit.id, spot_pos);
            }
            play_state.show_toast(RescueState::joined_message(&recruit.name));
            Some(RescueInteract {
                prompt: None,
                distance,
                consumed_press: true,
            })
        }
        Err(RescueBlocker::TeamFull) => {
            play_state.show_toast("Team full".to_string());
            Some(RescueInteract {
                prompt: Some("Team full".to_string()),
                distance,
                consumed_press: true,
            })
        }
        Err(_) => None,
    }
}

/// Ensures any waking POI holding an unrescued survivor spawns at least `min_zombies_near` zombies.
pub fn top_up_poi_zombies(
    spawns: &mut Vec<PoiSpawn>,
    pz: &mut PoiZombies,
    sites: &[PoiZombieSite],
    rescue: Option<&RescueState>,
    physics: &PhysicsWorld,
) {
    let Some(rescue) = rescue else {
        return;
    };
    let min_zombies = rescue.config.min_zombies_near;
    if min_zombies == 0 {
        return;
    }
    for spot in rescue.active_spots() {
        let existing = spawns.iter().filter(|s| s.poi_id == spot.poi_id).count();
        if existing == 0 || existing >= min_zombies {
            continue;
        }
        let site = sites.iter().find(|s| s.poi_id == spot.poi_id);
        for k in existing..min_zombies {
            let pos = site
                .and_then(|s| (k < s.spawn_points.len()).then(|| s.world_spawn_point(k)).flatten())
                .unwrap_or_else(|| {
                    let angle = std::f32::consts::TAU * (k as f32) / (min_zombies as f32);
                    let x = spot.position.x + angle.cos() * EXTRA_ZOMBIE_RING_M;
                    let z = spot.position.z + angle.sin() * EXTRA_ZOMBIE_RING_M;
                    let y = physics.ground_height(x, z).unwrap_or(spot.position.y);
                    Vec3::new(x, y, z)
                });
            let kind = pz.pick_kind();
            spawns.push(PoiSpawn::new(&spot.poi_id, pos, kind));
        }
    }
}
