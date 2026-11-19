//! Save files <-> the running game. The save crate holds plain data in Godot's shape; this file maps
//! the game's state onto it. Systems the Rust game does not have yet (camp, farm, dog...) stay in the
//! file untouched, because a save is always written over the one it was loaded from.

use game::{CameraRig, Player};
use glam::Vec3;
use save::{SaveFile, Slots, Vec3Data};

use crate::menus::{SlotState, SlotSummary};
use crate::play::PlayState;

/// What a loaded save puts back into the game (the world seed is applied before the world is built).
pub struct Restore {
    pub position: Vec3,
    pub body_yaw: f32,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub health: f32,
    pub hour: f32,
    pub day: i32,
    pub play_time: f64,
}

fn d3(v: Vec3) -> Vec3Data {
    Vec3Data { x: v.x, y: v.y, z: v.z }
}

/// The world seed of a save (Godot's "world" system), if it has one.
pub fn seed_of(file: &SaveFile) -> Option<i32> {
    file.systems.world.as_ref().map(|w| w.seed)
}

/// The player and clock parts of a save. None if the save has no player (then the game starts fresh at the camp).
pub fn restore(file: &SaveFile) -> Option<Restore> {
    let p = file.systems.player.as_ref()?;
    let time = file.systems.time.clone().unwrap_or_default();

    // SEC-001: check is_finite() and clamp to valid ranges; bad values -> safe default + log
    let position = {
        let (x, y, z) = (p.position.x, p.position.y, p.position.z);
        if x.is_finite() && y.is_finite() && z.is_finite() {
            Vec3::new(x, y, z)
        } else {
            log::warn!("loaded position ({x}, {y}, {z}) is not finite, defaulting to spawn/zero");
            let (sx, sy, sz) = (p.spawn.x, p.spawn.y, p.spawn.z);
            if sx.is_finite() && sy.is_finite() && sz.is_finite() {
                Vec3::new(sx, sy, sz)
            } else {
                Vec3::ZERO
            }
        }
    };

    let body_yaw = if p.body_yaw.is_finite() {
        p.body_yaw
    } else {
        log::warn!("loaded body_yaw {} is not finite, defaulting to 0.0", p.body_yaw);
        0.0
    };

    let camera_yaw = if p.camera_yaw.is_finite() {
        p.camera_yaw
    } else {
        log::warn!("loaded camera_yaw {} is not finite, defaulting to 0.0", p.camera_yaw);
        0.0
    };

    let camera_pitch = if p.camera_pitch.is_finite() {
        p.camera_pitch.clamp(-1.5, 1.5)
    } else {
        log::warn!("loaded camera_pitch {} is not finite, defaulting to 0.0", p.camera_pitch);
        0.0
    };

    let health = if p.health.is_finite() {
        p.health.clamp(0.0, 100.0)
    } else {
        log::warn!("loaded health {} is not finite, defaulting to 100.0", p.health);
        100.0
    };

    let hour = if time.hour.is_finite() {
        time.hour.clamp(0.0, 24.0)
    } else {
        log::warn!("loaded hour {} is not finite, defaulting to 12.0", time.hour);
        12.0
    };

    let day = time.day.max(1);

    let play_time = file.systems.stats.as_ref().map_or(0.0, |s| {
        if s.play_time.is_finite() && s.play_time >= 0.0 {
            s.play_time
        } else {
            log::warn!("loaded play_time {} is not valid, defaulting to 0.0", s.play_time);
            0.0
        }
    });

    Some(Restore {
        position,
        body_yaw,
        camera_yaw,
        camera_pitch,
        health,
        hour,
        day,
        play_time,
    })
}

/// Backpack stacks, looted crates, dropped items, and camp data from a save.
/// Returns warning messages for any damaged data that was reset (SAVE-004).
pub fn restore_play(file: &SaveFile, play: &mut PlayState) -> Vec<String> {
    let stacks = game::items::InventorySaveData {
        stacks: file
            .systems
            .inventory
            .as_ref()
            .map(|i| i.stacks.iter().map(|s| game::items::SavedSlot { slot: s.slot, id: s.id.clone(), count: s.count }).collect())
            .unwrap_or_default(),
    };
    let looted = file.systems.worldstate.as_ref().map(|w| w.looted.clone()).unwrap_or_default();
    let warnings = play.camp.load_from(&file.systems);
    if let Some(st) = &file.systems.stats
        && let Ok(v) = serde_json::to_value(st)
    {
        play.stats.from_save(&v);
    }
    // Hunger and thirst live in Godot's "need_hunger" / "need_thirst" systems
    if let Some(v) = file.systems.extra.get("need_hunger") {
        play.hunger.from_save(v);
    }
    if let Some(v) = file.systems.extra.get("map") {
        play.map.from_save(v);
    }
    if let (Some(v), Some(h)) = (file.systems.extra.get("hints"), play.hints.as_mut()) {
        h.from_save(v);
    }
    if let Some(v) = file.systems.extra.get("need_thirst") {
        play.thirst.from_save(v);
    }
    // SAVE-002: Restore dropped items into play.world_items
    if let Some(ws) = &file.systems.worldstate {
        play.world_items.clear();
        for items in ws.dropped.values() {
            for d in items {
                if !d.id.is_empty() && d.count > 0 && d.x.is_finite() && d.y.is_finite() && d.z.is_finite() {
                    play.world_items.add(&d.id, d.count, Vec3::new(d.x, d.y, d.z));
                }
            }
        }
    }
    // A save without an inventory keeps the start kit
    if file.systems.inventory.is_some() {
        play.load_parts(&stacks, &looted);
    } else {
        let current = play.save_parts().0;
        play.load_parts(&current, &looted);
    }
    warnings
}

/// Everything the game can save right now, written over `base` (the file it was loaded from).
pub struct Snapshot<'a> {
    pub seed: i32,
    pub player: &'a Player,
    pub rig: &'a CameraRig,
    pub health: f32,
    pub spawn: Vec3,
    pub clock: &'a world::time::Clock,
    pub play: &'a PlayState,
    pub play_time: f64,
    pub trees: Option<&'a game::TreeFelling>,
    pub poi_zombies: Option<&'a game::PoiZombies>,
}

pub fn capture(base: Option<&SaveFile>, s: &Snapshot) -> SaveFile {
    let mut file = base.cloned().unwrap_or_else(|| SaveFile::new_populated(s.seed));
    file.saved_at_utc = save::utc_now_iso8601();
    let sys = &mut file.systems;
    sys.world = Some(save::WorldSaveData { seed: s.seed });
    let mut player = sys.player.clone().unwrap_or_default();
    player.position = d3(s.player.position);
    player.body_yaw = s.player.yaw;
    player.camera_yaw = s.rig.yaw;
    player.camera_pitch = s.rig.pitch;
    player.health = s.health;
    player.spawn = d3(s.spawn);
    sys.player = Some(player);
    sys.time = Some(save::TimeSaveData { hour: s.clock.hour, day: s.clock.day });
    let mut stats: save::StatsSaveData = serde_json::from_value(s.play.stats.to_save()).unwrap_or_default();
    stats.play_time = s.play_time;
    stats.highest_day = stats.highest_day.max(s.clock.day);
    sys.stats = Some(stats);
    let (stacks, looted) = s.play.save_parts();
    sys.inventory = Some(save::InventorySaveData {
        stacks: stacks.stacks.into_iter().map(|e| save::SavedSlot { slot: e.slot, id: e.id, count: e.count }).collect(),
    });
    s.play.camp.save_into(sys);
    sys.extra.insert("need_hunger".into(), s.play.hunger.to_save());
    sys.extra.insert("need_thirst".into(), s.play.thirst.to_save());
    sys.extra.insert("map".into(), s.play.map.to_save());
    if let Some(h) = &s.play.hints {
        sys.extra.insert("hints".into(), h.to_save());
    }
    // SAVE-001 / ARCH-002: All world state (campfires, looted, fallen trees, visited POIs, dropped) captured in ONE place
    let mut ws = sys.worldstate.clone().unwrap_or_default();
    ws.looted = looted;
    ws.campfires = serde_json::from_value(s.play.camp.campfires.to_save()).unwrap_or_default();
    if let Some(trees) = s.trees {
        ws.fallen_trees = serde_json::from_value(trees.to_save()["fallen_trees"].clone()).unwrap_or_default();
    }
    if let Some(pz) = s.poi_zombies {
        ws.visited_pois = pz.visited_ids();
    }
    let mut dropped_map: std::collections::HashMap<String, Vec<save::DroppedItemData>> = std::collections::HashMap::new();
    for item in s.play.world_items.items() {
        let key = game::WorldItems::chunk_key(item.position);
        dropped_map.entry(key).or_default().push(save::DroppedItemData::new(
            &item.item_id,
            item.count,
            item.position.x,
            item.position.y,
            item.position.z,
        ));
    }
    ws.dropped = dropped_map;
    sys.worldstate = Some(ws);
    file
}

/// The three slot cards for the menus.
pub fn summaries(slots: &Slots) -> [SlotSummary; 3] {
    std::array::from_fn(|i| {
        let info = slots.info(i + 1);
        SlotSummary {
            state: match info.state {
                save::SlotState::Empty => SlotState::Empty,
                save::SlotState::Damaged => SlotState::Damaged,
                save::SlotState::Filled => SlotState::Filled,
            },
            day: info.day,
            play_time_s: info.play_time_s,
            difficulty: info.difficulty,
            saved_at: info.saved_at,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nan_and_infinite_values_load_safely() {
        let mut save = SaveFile::new_populated(42);
        if let Some(p) = save.systems.player.as_mut() {
            p.health = f32::NAN;
            p.position = Vec3Data::new(f32::NAN, f32::INFINITY, f32::NEG_INFINITY);
            p.spawn = Vec3Data::new(10.0, 1.0, 20.0);
            p.body_yaw = f32::NAN;
            p.camera_yaw = f32::INFINITY;
            p.camera_pitch = 100.0; // out of range
        }
        if let Some(t) = save.systems.time.as_mut() {
            t.hour = f32::NAN;
            t.day = -5;
        }
        if let Some(s) = save.systems.stats.as_mut() {
            s.play_time = -100.0;
        }

        let r = restore(&save).expect("player restored");
        assert_eq!(r.health, 100.0);
        assert!(r.health.is_finite());
        assert_eq!(r.position, Vec3::new(10.0, 1.0, 20.0));
        assert!(r.position.is_finite());
        assert_eq!(r.body_yaw, 0.0);
        assert_eq!(r.camera_yaw, 0.0);
        assert_eq!(r.camera_pitch, 1.5); // clamped to 1.5
        assert_eq!(r.hour, 12.0);
        assert_eq!(r.day, 1);
        assert_eq!(r.play_time, 0.0);
    }
}

