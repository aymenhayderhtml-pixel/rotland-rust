#![allow(clippy::field_reassign_with_default)]
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use save::{
    godot_save_dir, import_godot_from, CampfireData, DeathRule,
    DifficultySaveData, DroppedItemData, InventorySaveData, PlayerSaveData, SaveError, SaveFile,
    SavedSlot, SlotState, Slots, StatsSaveData, TimeSaveData, Vec3Data, WalletSaveData,
    WorldSaveData, WorldStateSaveData, FORMAT_VERSION,
};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(name: &str) -> Self {
        let count = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("rotland_save_test_{name}_{stamp}_{count}"));
        std::fs::create_dir_all(&path).expect("failed to create temp test directory");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[test]
fn round_trip_save_file() {
    let temp = TempDirGuard::new("roundtrip");
    let slots = Slots::new(temp.path());

    let mut save = SaveFile::new_populated(42);
    save.saved_at_utc = "2026-11-16T10:00:00Z".to_string();

    let mut player = PlayerSaveData::default();
    player.position = Vec3Data::new(10.0, 1.5, -20.0);
    player.body_yaw = 1.25;
    player.camera_yaw = 2.5;
    player.camera_pitch = -0.15;
    player.health = 75.0;
    player.spawn = Vec3Data::new(0.0, 1.0, 0.0);
    save.systems.player = Some(player);

    let mut stats = StatsSaveData::default();
    stats.kills.insert("walker".to_string(), 5);
    stats.headshots = 3;
    stats.deaths = 1;
    stats.highest_day = 2;
    stats.play_time = 450.0;
    stats.distance = 800.0;
    save.systems.stats = Some(stats);

    let mut inventory = InventorySaveData::default();
    inventory.stacks.push(SavedSlot::new(0, "baseball_bat", 1));
    inventory.stacks.push(SavedSlot::new(1, "canned_beans", 4));
    save.systems.inventory = Some(inventory);

    save.systems.time = Some(TimeSaveData::new(14.0, 2));
    save.systems.world = Some(WorldSaveData::new(42));

    let mut worldstate = WorldStateSaveData::default();
    worldstate.looted.push("crate_0_0".to_string());
    worldstate.dropped.insert(
        "0_0".to_string(),
        vec![DroppedItemData::new("apple", 2, 1.0, 0.5, 2.0)],
    );
    worldstate.fallen_trees.push("tree_0_0_1".to_string());
    worldstate
        .campfires
        .insert("0_0".to_string(), vec![CampfireData::new(5.0, 0.5, 5.0, 4.0)]);
    worldstate.visited_pois.push("cabin_1".to_string());
    save.systems.worldstate = Some(worldstate);

    save.systems.difficulty = Some(DifficultySaveData::new(1.0, 1.0, 1.0, DeathRule::LoseAll));
    save.systems.wallet = Some(WalletSaveData::new(80));

    // Add extra system (like Godot's dogs or camp)
    save.systems.extra.insert(
        "camp".to_string(),
        serde_json::json!({ "level": 2, "tents": 3 }),
    );

    // Write to slot 1
    slots.write(1, &save).expect("failed to write slot 1");

    // Read back
    let loaded = slots.read(1).expect("failed to read slot 1 back");

    let mut compared = 0;

    assert_eq!(loaded.version, FORMAT_VERSION);
    compared += 1;
    assert_eq!(loaded.saved_at_utc, "2026-11-16T10:00:00Z");
    compared += 1;

    let p = loaded.systems.player.as_ref().expect("player missing");
    assert_eq!(p.position, Vec3Data::new(10.0, 1.5, -20.0));
    compared += 1;
    assert_eq!(p.body_yaw, 1.25);
    compared += 1;
    assert_eq!(p.camera_yaw, 2.5);
    compared += 1;
    assert_eq!(p.camera_pitch, -0.15);
    compared += 1;
    assert_eq!(p.health, 75.0);
    compared += 1;
    assert_eq!(p.spawn, Vec3Data::new(0.0, 1.0, 0.0));
    compared += 1;

    let s = loaded.systems.stats.as_ref().expect("stats missing");
    assert_eq!(s.kills.get("walker").copied(), Some(5));
    compared += 1;
    assert_eq!(s.headshots, 3);
    compared += 1;
    assert_eq!(s.deaths, 1);
    compared += 1;
    assert_eq!(s.highest_day, 2);
    compared += 1;
    assert_eq!(s.play_time, 450.0);
    compared += 1;
    assert_eq!(s.distance, 800.0);
    compared += 1;

    let inv = loaded.systems.inventory.as_ref().expect("inventory missing");
    assert_eq!(inv.stacks.len(), 2);
    compared += 1;
    assert_eq!(inv.stacks[0], SavedSlot::new(0, "baseball_bat", 1));
    compared += 1;
    assert_eq!(inv.stacks[1], SavedSlot::new(1, "canned_beans", 4));
    compared += 1;

    let t = loaded.systems.time.as_ref().expect("time missing");
    assert_eq!(t.hour, 14.0);
    compared += 1;
    assert_eq!(t.day, 2);
    compared += 1;

    let w = loaded.systems.world.as_ref().expect("world missing");
    assert_eq!(w.seed, 42);
    compared += 1;

    let ws = loaded.systems.worldstate.as_ref().expect("worldstate missing");
    assert_eq!(ws.looted, vec!["crate_0_0".to_string()]);
    compared += 1;
    assert_eq!(ws.dropped.get("0_0").unwrap().len(), 1);
    compared += 1;
    assert_eq!(ws.fallen_trees, vec!["tree_0_0_1".to_string()]);
    compared += 1;
    assert_eq!(ws.campfires.get("0_0").unwrap().len(), 1);
    compared += 1;
    assert_eq!(ws.visited_pois, vec!["cabin_1".to_string()]);
    compared += 1;

    let diff = loaded.systems.difficulty.as_ref().expect("difficulty missing");
    assert_eq!(diff.label(), "Normal");
    compared += 1;

    let wal = loaded.systems.wallet.as_ref().expect("wallet missing");
    assert_eq!(wal.dollars, 80);
    compared += 1;

    let extra_camp = loaded.systems.extra.get("camp").expect("extra camp missing");
    assert_eq!(extra_camp["level"], 2);
    compared += 1;
    assert_eq!(extra_camp["tents"], 3);
    compared += 1;

    assert!(compared > 0, "comparison test compared 0 items");
    assert_eq!(compared, 29);
}

#[test]
fn slot_info_empty_damaged_filled() {
    let temp = TempDirGuard::new("slot_info");
    let slots = Slots::new(temp.path());

    let mut compared = 0;

    // Slot 1: does not exist -> Empty
    let info1 = slots.info(1);
    assert_eq!(info1.slot, 1);
    compared += 1;
    assert_eq!(info1.state, SlotState::Empty);
    compared += 1;
    assert_eq!(info1.seed, 0);
    compared += 1;
    assert_eq!(info1.day, 0);
    compared += 1;
    assert_eq!(info1.play_time_s, 0.0);
    compared += 1;
    assert_eq!(info1.difficulty, "");
    compared += 1;
    assert_eq!(info1.saved_at, "");
    compared += 1;

    // Slot 2: invalid JSON -> Damaged
    std::fs::write(slots.path_for(2), "not a json file").unwrap();
    let info2 = slots.info(2);
    assert_eq!(info2.slot, 2);
    compared += 1;
    assert_eq!(info2.state, SlotState::Damaged);
    compared += 1;
    assert_eq!(info2.seed, 0);
    compared += 1;
    assert_eq!(info2.day, 0);
    compared += 1;
    assert_eq!(info2.play_time_s, 0.0);
    compared += 1;
    assert_eq!(info2.difficulty, "");
    compared += 1;
    assert_eq!(info2.saved_at, "");
    compared += 1;

    // Slot 3: wrong version -> Damaged
    let bad_ver_json = r#"{"version": 99, "savedAtUtc": "2026-11-16T12:00:00Z", "systems": {}}"#;
    std::fs::write(slots.path_for(3), bad_ver_json).unwrap();
    let info3 = slots.info(3);
    assert_eq!(info3.slot, 3);
    compared += 1;
    assert_eq!(info3.state, SlotState::Damaged);
    compared += 1;

    // Slot 4: valid save file -> Filled
    let mut save4 = SaveFile::new();
    save4.saved_at_utc = "2026-11-16T09:30:00Z".to_string();
    save4.systems.world = Some(WorldSaveData::new(777));
    let mut stats4 = StatsSaveData::default();
    stats4.highest_day = 5;
    stats4.play_time = 1800.5;
    save4.systems.stats = Some(stats4);
    save4.systems.difficulty = Some(DifficultySaveData::new(0.5, 1.5, 0.5, DeathRule::KeepAll));
    slots.write(4, &save4).unwrap();

    let info4 = slots.info(4);
    assert_eq!(info4.slot, 4);
    compared += 1;
    assert_eq!(info4.state, SlotState::Filled);
    compared += 1;
    assert_eq!(info4.seed, 777);
    compared += 1;
    assert_eq!(info4.day, 5);
    compared += 1;
    assert_eq!(info4.play_time_s, 1800.5);
    compared += 1;
    assert_eq!(info4.difficulty, "Relaxed");
    compared += 1;
    assert_eq!(info4.saved_at, "2026-11-16T09:30:00Z");
    compared += 1;

    // Slot 5: valid save file with missing difficulty -> Filled with "Normal" default
    let mut save5 = SaveFile::new();
    save5.saved_at_utc = "2026-11-16T11:00:00Z".to_string();
    save5.systems.world = Some(WorldSaveData::new(999));
    slots.write(5, &save5).unwrap();

    let info5 = slots.info(5);
    assert_eq!(info5.slot, 5);
    compared += 1;
    assert_eq!(info5.state, SlotState::Filled);
    compared += 1;
    assert_eq!(info5.difficulty, "Normal");
    compared += 1;

    assert!(compared > 0, "comparison test compared 0 items");
    assert_eq!(compared, 26);
}

#[test]
fn fixture_parses_and_preserves_unknown_systems() {
    let repo_fixture = paths::repo_path("rust/save/tests/fixtures/godot_save.json");
    let content = if repo_fixture.is_file() {
        std::fs::read_to_string(&repo_fixture).expect("read fixture via repo_path")
    } else {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/godot_save.json");
        std::fs::read_to_string(&manifest).expect("read fixture via CARGO_MANIFEST_DIR")
    };

    let save = SaveFile::from_json_str(&content).expect("parse fixture");

    let mut compared = 0;

    assert_eq!(save.version, 1);
    compared += 1;
    assert_eq!(save.saved_at_utc, "2026-11-16T12:34:56Z");
    compared += 1;

    let p = save.systems.player.as_ref().expect("player missing");
    assert_eq!(p.position, Vec3Data::new(12.5, 1.8, -45.0));
    compared += 1;
    assert_eq!(p.body_yaw, 1.57);
    compared += 1;
    assert_eq!(p.camera_yaw, 2.5);
    compared += 1;
    assert_eq!(p.camera_pitch, -0.25);
    compared += 1;
    assert_eq!(p.health, 85.0);
    compared += 1;
    assert_eq!(p.spawn, Vec3Data::new(0.0, 1.0, 0.0));
    compared += 1;

    let s = save.systems.stats.as_ref().expect("stats missing");
    assert_eq!(s.kills.get("walker").copied(), Some(12));
    compared += 1;
    assert_eq!(s.kills.get("runner").copied(), Some(3));
    compared += 1;
    assert_eq!(s.headshots, 7);
    compared += 1;
    assert_eq!(s.deaths, 1);
    compared += 1;
    assert_eq!(s.highest_day, 4);
    compared += 1;
    assert_eq!(s.play_time, 3600.5);
    compared += 1;
    assert_eq!(s.distance, 2450.0);
    compared += 1;

    let inv = save.systems.inventory.as_ref().expect("inventory missing");
    assert_eq!(inv.stacks.len(), 5);
    compared += 1;
    assert_eq!(inv.stacks[0], SavedSlot::new(0, "baseball_bat", 1));
    compared += 1;
    assert_eq!(inv.stacks[1], SavedSlot::new(1, "pistol", 1));
    compared += 1;
    assert_eq!(inv.stacks[2], SavedSlot::new(2, "canned_beans", 3));
    compared += 1;
    assert_eq!(inv.stacks[3], SavedSlot::new(5, "flashlight", 1));
    compared += 1;
    assert_eq!(inv.stacks[4], SavedSlot::new(8, "bandage", 2));
    compared += 1;

    let t = save.systems.time.as_ref().expect("time missing");
    assert_eq!(t.hour, 14.5);
    compared += 1;
    assert_eq!(t.day, 4);
    compared += 1;

    let w = save.systems.world.as_ref().expect("world missing");
    assert_eq!(w.seed, 123456789);
    compared += 1;

    let ws = save.systems.worldstate.as_ref().expect("worldstate missing");
    assert_eq!(ws.looted, vec!["crate_0_0_1", "crate_1_0_0"]);
    compared += 1;
    assert_eq!(ws.dropped.get("0_0").unwrap().len(), 1);
    compared += 1;
    assert_eq!(ws.dropped["0_0"][0].id, "canned_beans");
    compared += 1;
    assert_eq!(ws.dropped["0_0"][0].count, 1);
    compared += 1;
    assert_eq!(ws.dropped["0_0"][0].x, 10.0);
    compared += 1;
    assert_eq!(ws.fallen_trees, vec!["0_0_3"]);
    compared += 1;
    assert_eq!(ws.campfires.get("0_0").unwrap().len(), 1);
    compared += 1;
    assert_eq!(ws.campfires["0_0"][0].hours, 3.5);
    compared += 1;
    assert_eq!(ws.visited_pois, vec!["cabin_0_0"]);
    compared += 1;

    let diff = save.systems.difficulty.as_ref().expect("difficulty missing");
    assert_eq!(diff.zombies, 1.0);
    compared += 1;
    assert_eq!(diff.loot, 1.0);
    compared += 1;
    assert_eq!(diff.damage, 1.0);
    compared += 1;
    assert_eq!(diff.death, DeathRule::LoseAll);
    compared += 1;
    assert_eq!(diff.label(), "Normal");
    compared += 1;

    let wal = save.systems.wallet.as_ref().expect("wallet missing");
    assert_eq!(wal.dollars, 150);
    compared += 1;

    // Unknown systems must be captured in extra
    let dogs = save.systems.extra.get("dogs").expect("dogs in extra");
    assert_eq!(dogs["level"], 2);
    compared += 1;
    let farm = save.systems.extra.get("farm").expect("farm in extra");
    assert_eq!(farm["plots"][0]["crop"], "potato");
    compared += 1;

    // Round-trip through a temp folder slot
    let temp = TempDirGuard::new("fixture_roundtrip");
    let slots = Slots::new(temp.path());
    slots.write(1, &save).expect("write fixture save to slot 1");
    let loaded = slots.read(1).expect("read fixture save from slot 1");

    assert_eq!(loaded.systems.extra.get("dogs"), Some(dogs));
    compared += 1;
    assert_eq!(loaded.systems.extra.get("farm"), Some(farm));
    compared += 1;

    assert!(compared > 0, "comparison test compared 0 items");
    assert_eq!(compared, 43);
}

#[test]
fn guard_refuses_godot_and_default_folders() {
    let mut compared = 0;

    // Refuses godot_save_dir
    let godot_slots = Slots::new(godot_save_dir());
    let write_err = godot_slots.write(1, &SaveFile::default());
    assert!(matches!(write_err, Err(SaveError::ProtectedPath(_))));
    compared += 1;

    let delete_err = godot_slots.delete(1);
    assert!(matches!(delete_err, Err(SaveError::ProtectedPath(_))));
    compared += 1;

    // The real Rust folder guard is a unit test in src/slots.rs (cfg(test) does not reach integration tests)

    assert!(compared > 0, "comparison test compared 0 items");
    assert_eq!(compared, 2);
}

#[test]
fn import_godot_copies_by_reading_only() {
    let src_guard = TempDirGuard::new("sim_godot");
    let dest_guard = TempDirGuard::new("sim_rust");

    let src_slots = Slots::new(src_guard.path());
    let dest_slots = Slots::new(dest_guard.path());

    let mut original_save = SaveFile::new_populated(1001);
    original_save.saved_at_utc = "2026-11-16T15:00:00Z".to_string();
    original_save.systems.wallet = Some(WalletSaveData::new(350));

    // Write to simulated godot source directory
    src_slots.write(1, &original_save).unwrap();
    let src_bytes_before = std::fs::read(src_slots.path_for(1)).unwrap();

    // Import into destination
    let imported = import_godot_from(1, src_guard.path(), &dest_slots).expect("import slot 1");

    let mut compared = 0;

    assert_eq!(imported.systems.world.unwrap().seed, 1001);
    compared += 1;
    assert_eq!(imported.systems.wallet.unwrap().dollars, 350);
    compared += 1;

    // Verify file exists in dest
    assert!(dest_slots.path_for(1).is_file());
    compared += 1;

    let dest_loaded = dest_slots.read(1).expect("read dest slot 1");
    assert_eq!(dest_loaded.systems.world.unwrap().seed, 1001);
    compared += 1;
    assert_eq!(dest_loaded.systems.wallet.unwrap().dollars, 350);
    compared += 1;

    // Source was ONLY read: exactly identical bytes, still exists
    let src_bytes_after = std::fs::read(src_slots.path_for(1)).unwrap();
    assert_eq!(src_bytes_before, src_bytes_after);
    compared += 1;

    assert!(compared > 0, "comparison test compared 0 items");
    assert_eq!(compared, 6);
}

#[test]
fn default_constructors_have_real_defaults() {
    let mut compared = 0;

    let player = PlayerSaveData::default();
    assert_eq!(player.health, 100.0);
    compared += 1;

    let time = TimeSaveData::default();
    assert_eq!(time.hour, 8.0);
    compared += 1;
    assert_eq!(time.day, 1);
    compared += 1;

    let stats = StatsSaveData::default();
    assert_eq!(stats.highest_day, 1);
    compared += 1;

    let diff = DifficultySaveData::default();
    assert_eq!(diff.zombies, 1.0);
    compared += 1;
    assert_eq!(diff.loot, 1.0);
    compared += 1;
    assert_eq!(diff.damage, 1.0);
    compared += 1;
    assert_eq!(diff.death, DeathRule::LoseAll);
    compared += 1;
    assert_eq!(diff.label(), "Normal");
    compared += 1;

    let wallet = WalletSaveData::default();
    assert_eq!(wallet.dollars, 20);
    compared += 1;

    let campfire = CampfireData::new(0.0, 0.0, 0.0, 5.0);
    assert_eq!(campfire.hours, 5.0);
    compared += 1;

    let save = SaveFile::default();
    assert_eq!(save.version, 1);
    compared += 1;

    assert!(compared > 0, "comparison test compared 0 items");
    assert_eq!(compared, 12);
}
