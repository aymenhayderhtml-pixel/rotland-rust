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

fn v3(v: &Vec3Data) -> Vec3 {
    Vec3::new(v.x, v.y, v.z)
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
    Some(Restore {
        position: v3(&p.position),
        body_yaw: p.body_yaw,
        camera_yaw: p.camera_yaw,
        camera_pitch: p.camera_pitch,
        health: p.health,
        hour: time.hour,
        day: time.day,
        play_time: file.systems.stats.as_ref().map_or(0.0, |s| s.play_time),
    })
}

/// Backpack stacks and looted crates from a save, in the game's own types.
pub fn restore_play(file: &SaveFile, play: &mut PlayState) {
    let stacks = game::items::InventorySaveData {
        stacks: file
            .systems
            .inventory
            .as_ref()
            .map(|i| i.stacks.iter().map(|s| game::items::SavedSlot { slot: s.slot, id: s.id.clone(), count: s.count }).collect())
            .unwrap_or_default(),
    };
    let looted = file.systems.worldstate.as_ref().map(|w| w.looted.clone()).unwrap_or_default();
    // Hunger and thirst live in Godot's "need_hunger" / "need_thirst" systems
    if let Some(v) = file.systems.extra.get("need_hunger") {
        play.hunger.from_save(v);
    }
    if let Some(v) = file.systems.extra.get("need_thirst") {
        play.thirst.from_save(v);
    }
    // A save without an inventory keeps the start kit
    if file.systems.inventory.is_some() {
        play.load_parts(&stacks, &looted);
    } else {
        let current = play.save_parts().0;
        play.load_parts(&current, &looted);
    }
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
    let mut stats = sys.stats.clone().unwrap_or_default();
    stats.play_time = s.play_time;
    stats.highest_day = stats.highest_day.max(s.clock.day);
    sys.stats = Some(stats);
    let (stacks, looted) = s.play.save_parts();
    sys.inventory = Some(save::InventorySaveData {
        stacks: stacks.stacks.into_iter().map(|e| save::SavedSlot { slot: e.slot, id: e.id, count: e.count }).collect(),
    });
    sys.extra.insert("need_hunger".into(), s.play.hunger.to_save());
    sys.extra.insert("need_thirst".into(), s.play.thirst.to_save());
    let mut ws = sys.worldstate.clone().unwrap_or_default();
    ws.looted = looted;
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
