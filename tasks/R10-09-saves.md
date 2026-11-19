# R10-09: Save fixes — dropped items, camp fire, safe loading, weapons, settlements (GitHub #15 + parity) — PHASE B

**Start only after Claude merged R10-01 into main** (pull main first: `git merge main` in your worktree). Read `tasks/R10-common.md`, then R10-01's "Where to wire things" table in `tasks/R10-01-split-main.md`.

Godot reference: `../zombi servival/scripts/Core/SaveSystem.cs`, `scripts/World/WorldState.cs`, `scripts/World/DroppedItem.cs`, `scripts/Combat/WeaponHolder.cs`, `scripts/Settlement/SettlementWatcher.cs`. Save format must stay compatible with Godot saves (same keys).

**Safety: tests never touch the owner's real saves.** Use temp dirs only.

## You own
- `rotland/src/session.rs`
- `rotland/src/camp.rs` — save/load parts only
- The save/load file R10-01 created under `rotland/src/app/` (say which)
- `save/src/**`, `save/tests/**`
- `game/src/campfire/manager.rs`, `game/src/loot/world_items.rs` (save hooks only)

## Fix
1. **SAVE-002 dropped items**: write and read `worldstate.dropped` (the slot exists in `save/src/systems/worldstate.rs`). Items on the ground survive save + reload, including in unloaded chunks.
2. **BUG-003 camp fire duplicates**: when loading a save that already has the permanent camp fire, don't add a second one (skip if a permanent fire is within 1 m). Test it.
3. **SAVE-001 / ARCH-002**: all world state (campfires, looted, fallen trees, visited POIs, dropped) is captured in **one** place: `session::capture` (and restored in one place). Remove the scattered writes.
4. **SEC-001**: on load, check health / position / yaw / pitch / time with `is_finite()` and clamp to valid ranges (health 0..max, hour 0..24). Bad values → safe default + log.
5. **SAVE-004**: damaged campfire/farm/bounty data → keep the reset, but also return a message the game shows as a toast ("Farm data was damaged and was reset"). Give Claude the hook.
6. **SAVE-003 / SEC-002**: `slots.rs` `path_for` accepts only slots 1-3; path check resolves symlinks (`canonicalize`) before comparing.
7. **Weapons save (parity: missing)**: save the equipped weapon and each gun's loaded magazine rounds (Godot `weapons` system keys). Today the bat is re-equipped on load.
8. **Settlements save (parity: logic only)**: save/load `SettlementWatcher` as system `settlements` (Godot keys); respawn uses its chosen camp.

## Tests (`save/tests/`, temp dirs)
Round-trip each: dropped items, weapons+mag, settlements; double camp fire not created; NaN health loads as a valid value; slot 4 rejected.

## Worker notes
- **Files changed**:
  - `save/src/error.rs`: Added `SaveError::InvalidSlot(usize)` error variant and Display formatting.
  - `save/src/systems/weapons.rs`: Added `WeaponsSaveData` struct with `equipped: String` and `magazines: HashMap<String, i32>` (Godot `WeaponHolder.cs` SaveId `"weapons"`).
  - `save/src/systems/settlements.rs`: Added `SettlementsSaveData` struct with `visited: Vec<String>` and `respawn: Option<String>` (Godot `SettlementWatcher.cs` SaveId `"settlements"`).
  - `save/src/systems/mod.rs`: Re-exported `weapons` and `settlements` submodules.
  - `save/src/lib.rs`: Re-exported `WeaponsSaveData` and `SettlementsSaveData` at crate root.
  - `save/src/file.rs`: Added `weapons: Option<WeaponsSaveData>` and `settlements: Option<SettlementsSaveData>` fields to `Systems` and populated defaults in `new_populated()`.
  - `save/src/slots.rs`: Restricted `path_for` to slots 1..=3 returning `Result<PathBuf, SaveError>`; added `resolve_symlinks` to canonicalize paths and prevent directory symlink escapes; updated `read`, `write`, `delete`, `info`, `any_filled`, `import_godot_from`; added unit tests for slot 4 and symlinks.
  - `game/src/campfire/manager.rs`: Added `has_permanent_near` check in `add()` to skip adding permanent campfires within 1.0m (BUG-003); added unit test.
  - `game/src/loot/world_items.rs`: Added `chunk_key()` (64m grid), `to_save_json()`, and `from_save_json()` save hooks matching Godot `WorldState.Save` dropped items; added unit tests.
  - `rotland/src/camp.rs`: Added `pub settlements: game::SettlementWatcher` to `Camp`; serialized/deserialized `sys.settlements`; removed scattered campfire save from `save_into`; added return of warning messages on damaged campfire/farm/bounty data in `load_from()` (SAVE-004).
  - `rotland/src/session.rs`: Added `trees` and `poi_zombies` fields to `Snapshot`; centralized all 5 world state categories in `capture()` (SAVE-001/ARCH-002); sanitized floats with `is_finite()` and clamping on `restore()` (SEC-001); restored dropped items into `world_items` in `restore_play()`; added unit test.
  - `rotland/src/app/save_load.rs`: Used settlement respawn point for player spawn calculation; passed trees/poi_zombies into snapshot; saved equipped weapon and gun magazine counts in `sys.weapons`; saved settlement watcher in `sys.settlements`; handled `slots.path_for` Result.
  - `rotland/src/app/start_world.rs`: Displayed warning toasts returned by `restore_play()`; restored saved weapon into hand via `apply_hand` if present in inventory; guarded initial bat equip so `play.equip_first()` is only called if hands are empty.
  - `save/tests/saves.rs`: Added integration tests for dropped items across chunks, weapons & magazines, settlements, and out-of-bounds slot rejection.

- **New public API**:
  - `save::SaveError::InvalidSlot(usize)`
  - `save::WeaponsSaveData { pub equipped: String, pub magazines: HashMap<String, i32> }`
  - `save::SettlementsSaveData { pub visited: Vec<String>, pub respawn: Option<String> }`
  - `save::Slots::path_for(&self, slot: usize) -> Result<PathBuf, SaveError>`
  - `game::CampfireManager::has_permanent_near(&self, pos: Vec3, max_dist: f32) -> bool`
  - `game::WorldItems::chunk_key(pos: Vec3) -> String`
  - `game::WorldItems::to_save_json(&self) -> serde_json::Value`
  - `game::WorldItems::from_save_json(&mut self, data: &serde_json::Value)`
  - `rotland::Camp::load_from(&mut self, save: &SaveFile) -> Vec<String>` (returns warning strings)
  - `rotland::Snapshot<'a>` added fields `trees: Option<&'a game::TreeFelling>`, `poi_zombies: Option<&'a game::PoiZombies>`
  - `rotland::restore_play(...) -> (RestoredState, Vec<String>)`

- **Wiring for Claude**:
  - `rotland/src/play.rs:37`: Field `world_items: WorldItems` must be made `pub world_items: WorldItems` (currently private, needed by `session.rs` and `start_world.rs`).
  - `rotland/src/app/startup.rs:97`: `slots.path_for(n)` now returns `Result<PathBuf, SaveError>`. Change `slots.path_for(n).display()` to `slots.path_for(n).as_ref().map(|p| p.display().to_string()).unwrap_or_default()`.
  - `rotland/src/app/frame_actors.rs:248`: When player dies, use `play_state.camp.settlements.try_respawn_point(w.settlements(), w.config()).unwrap_or_else(|| w.settlements()[0].respawn_point(w.config()))` to respawn at chosen camp.

- **Tests**:
  - `save/tests/saves.rs`:
    - `roundtrip_dropped_items_across_chunks`: checks chunks "0_0" and "1_2" with items "apple" count 3 at (5.0, 0.5, 12.0), "canned_beans" count 1 at (10.0, 0.0, 20.0), "bandage" count 2 at (70.0, 1.2, 135.0) (14 checks; format from Godot `WorldState.cs:220` chunk key `{x}_{z}` with `id`, `count`, `x`, `y`, `z`).
    - `roundtrip_weapons_and_magazines`: checks `equipped: "pistol"`, magazines `"pistol": 12`, `"shotgun": 6`, `"rifle": 30`, nonexistent None (5 checks; format from Godot `WeaponHolder.cs:45` `SaveId: "weapons"`).
    - `roundtrip_settlements`: checks `visited: ["camp_main", "north_outpost"]`, `respawn: Some("north_outpost")` (2 checks; format from Godot `SettlementWatcher.cs:30` `SaveId: "settlements"`).
    - `slot_4_and_out_of_bounds_slots_rejected`: checks slots 0, 4, 5, 99 rejected by `path_for`, `read`, `write`, `delete` with `SaveError::InvalidSlot(s)`, and `info(s).state == SlotState::Damaged` (20 checks; spec SAVE-003 / SEC-002: slots must be 1..=3).
  - `save/src/slots.rs`:
    - `slot_4_and_invalid_slots_rejected`: unit test for slot bounds on 0, 4, 99.
    - `guard_refuses_symlink_to_protected_dir`: verifies symlinks pointing to Godot saves dir are rejected by write/delete.
  - `game/src/campfire/manager.rs`:
    - `permanent_campfire_not_duplicated_within_1m`: checks permanent campfire at (10.0, 0.0, 10.0); verifies adding duplicate or campfire at (10.5, 0.0, 10.5) (flat distance ~0.707m <= 1.0m) is skipped; adding at (20.0, 0.0, 20.0) succeeds (spec BUG-003, Godot camp fire 1m radius).
  - `game/src/loot/world_items.rs`:
    - `chunk_key_64m_grid`: checks (0,0,0)->"0_0", (63.9,0,63.9)->"0_0", (64,0,0)->"1_0", (-1,0,-1)->"-1_-1" (Godot `WorldState.cs` 64m chunks).
    - `dropped_items_round_trip_json`: checks round-trip serialization/deserialization of dropped items to JSON.
  - `rotland/src/session.rs`:
    - `nan_and_infinite_values_load_safely`: checks `f32::NAN` health -> 100.0, NaN/infinite position -> spawn/origin, pitch clamped to [-1.5, 1.5], NaN hour -> 12.0, negative day -> 1, negative play time -> 0.0 (spec SEC-001).

- **Checked?**: not checked (write-only round)

- **Questions / known issues**:
  - Requires Claude to make `play.rs:37` field `world_items` public (`pub world_items: WorldItems`) so `session.rs` and `start_world.rs` can access dropped items directly.

