# R10-08: No crashes on missing or broken files; load item data once (GitHub #14, #19 RULE-003/CODE-002/CODE-004) — PHASE A

Read `tasks/R10-common.md` first. `gh issue view 14` has every finding with file:line.

## You own
- `game/src/items/**` (inventory, hotbar, start kit, registry)
- `game/src/economy/bank.rs`
- `game/src/animals/config.rs` (only the `Default` impl)
- `game/src/loot/roller.rs`
- `game/src/vehicles/manager.rs` (line ~91 unwrap only)
- `game/src/crafting/system.rs` (line ~142 unwrap only)
- `paths/src/lib.rs`
- `rotland/src/player_visual.rs`
- `save/tests/saves.rs`, `world/tests/terrain_unit_tests.rs` (RULE-003 only)
- `engine/src/render/camera.rs` (`ortho`), `save/src/slots.rs` (`SlotInfo::damaged`) — CODE-004 only: remove if truly unused (grep the whole repo first), else say where it's used.

## 1. One shared item registry (CRASH-003/004, PERF-005)
- `Inventory::new`, `with_capacity`, `Bank::new` re-read `data/items.json` from disk and panic. Add constructors that take `Arc<ItemRegistry>` (or whatever the registry type is called — open it).
- **Keep the old constructors** so call sites in main.rs / camp.rs / play.rs still compile this round, but make them load through one cached `OnceLock<Arc<ItemRegistry>>` and never panic (log + empty registry). Mark them `#[deprecated(note = "R11: pass Arc<ItemRegistry>")]` **only if** that doesn't create clippy warnings at call sites you don't own — otherwise use a doc comment.
- List every call site (grep) in your notes so Claude can switch them over.

## 2. Friendly errors instead of panics
- **CRASH-001** `player_visual.rs`: missing character/weapon model or skeleton in `models.pack` → log an error and use a fallback (a simple box mesh or skip drawing that weapon). The game must keep running.
- **CRASH-002** (main.rs:465, not yours): write a `pub fn` in the world/game crate that returns `Result<_, DataError>` with the **file name** in the message; Claude will call it from main.rs and show the message.
- **CRASH-005/006**: start kit and `AnimalConfigData::default` → no disk reads in `Default`; return `Result` from the loader.
- **CRASH-007** `paths`: started outside the project folder without `ROTLAND_REPO` → clear message: what it looked for and how to fix (`ROTLAND_REPO=/path/to/rotland-rust`). Return `Result` or exit with that message; no panic backtrace.
- **BUG-005** loot roller `.last().expect()` on an empty table → return no item. **BUG-006/007/008** → `if let` / entry API. **CODE-002** loot roller: entry API instead of 3 lookups.

## 3. RULE-003
Replace `env!("CARGO_MANIFEST_DIR")` in the two test files with `paths::repo_path`.

## Tests
- Empty loot table rolls nothing (no panic).
- Inventory built from a shared registry behaves the same as before (existing inventory tests still meaningful).
- `paths` with a bad dir returns the error message (pure function test).
- player_visual fallback: a pure helper that picks fallback when a mesh id is missing.

## Worker notes

### Files changed
- `paths/src/lib.rs`: CRASH-007: Added `try_repo_root()`, `find_repo_root_from()`, `try_repo_path()`, `RepoNotFoundError`, exits cleanly with descriptive message on missing repo root instead of panic backtrace; added pure bad dir test.
- `game/src/items/registry.rs`: CRASH-003: Added `ItemRegistry::shared() -> Arc<ItemRegistry>` using `OnceLock` cache to load once without re-reading or crashing on broken data; added unit test.
- `game/src/items/inventory.rs`: CRASH-003/PERF-005: Updated `Inventory::new()` and `Inventory::with_capacity()` to use `ItemRegistry::shared()` instead of reading disk and panicking.
- `game/src/economy/bank.rs`: CRASH-004: Updated `Bank::new()` to use `ItemRegistry::shared()` instead of reading disk and panicking.
- `game/src/items/start.rs`: CRASH-005: Added `try_start_inventory(registry) -> Result<Inventory, StartInventoryLoadError>`, made `start_inventory` log error and fall back to empty inventory rather than panicking on missing file.
- `game/src/items/data_error.rs`: CRASH-002: Created `DataError` and `load_world_generator(seed, data_dir) -> Result<WorldGenerator, DataError>` naming the exact failing file for Claude's `main.rs:465` hook.
- `game/src/items/mod.rs`: Re-exported `DataError`, `load_world_generator`, and `try_start_inventory`.
- `game/src/animals/config.rs`: CRASH-006: Changed `AnimalConfigData::default()` to construct default config structs without reading disk or panicking.
- `game/src/loot/roller.rs`: BUG-005 / CODE-002: Fixed `pick_entry` returning `Option<&LootEntryData>` avoiding panic on empty table; updated `roll()` to use Entry API with single lookup; added unit test for empty loot table.
- `game/src/items/hotbar.rs`: BUG-006: Replaced `.unwrap()` in `select_slot` weapon unequip with `if let .. && let ..`.
- `game/src/vehicles/manager.rs`: BUG-007: Replaced insert + `.unwrap()` in `spawn_car` with Entry API lookup/insert.
- `game/src/crafting/system.rs`: BUG-008: Replaced `.unwrap()` in `start_craft` with `let Some(recipe) = ... else { return false; }`.
- `rotland/src/player_visual.rs`: CRASH-001: Added `fallback_skeleton()`, `fallback_box_vertices()`, `pick_model_or_fallback()`; handled missing character model/skeleton by logging error and using fallback box mesh; handled missing weapon model by logging error and skipping weapon drawing; added fallback unit test.
- `save/tests/saves.rs`: RULE-003: Removed `env!("CARGO_MANIFEST_DIR")` and used `paths::repo_path("save/tests/fixtures/godot_save.json")`.
- `world/tests/terrain_unit_tests.rs`: RULE-003: Removed `std::env::var("CARGO_MANIFEST_DIR")` and used `paths::repo_path("data/settlements.json")`.

### New public API
- `paths::RepoNotFoundError`: `pub struct RepoNotFoundError { pub message: String }` (implements Display, Error)
- `paths::find_repo_root_from(env: Option<&Path>, cwd: Option<&Path>, exe: Option<&Path>) -> Result<PathBuf, RepoNotFoundError>`
- `paths::try_repo_root() -> Result<PathBuf, RepoNotFoundError>`
- `paths::try_repo_path(relative: &str) -> Result<PathBuf, RepoNotFoundError>`
- `game::items::ItemRegistry::shared() -> Arc<ItemRegistry>`
- `game::items::try_start_inventory(registry: &ItemRegistry) -> Result<Inventory, StartInventoryLoadError>`
- `game::items::DataError`: `pub struct DataError { pub file_name: String, pub message: String }` (implements Display, Error)
- `game::items::load_world_generator(seed: i32, data_dir: impl AsRef<Path>) -> Result<world::WorldGenerator, DataError>`
- `rotland::player_visual::pick_model_or_fallback<'a>(pack: &'a ModelPack, name: &str, fallback_name: Option<&str>) -> Option<&'a pack::Model>`
- `rotland::player_visual::fallback_skeleton() -> pack::Skeleton`
- `rotland::player_visual::fallback_box_vertices() -> Vec<pack::PackVertex>`

### CODE-004 findings
- `engine/src/render/camera.rs:70` (`ortho`): NOT UNUSED. Actively used in `engine/src/render/shadow.rs:27`: `ortho(radius, radius, 0.0, depth) * shift * view`. Kept as is.
- `save/src/slots.rs:43` (`SlotInfo::damaged`): NOT UNUSED. Actively used in `save/src/slots.rs:59,65,69` on read errors, and tested in `save/tests/saves.rs:271`. Kept as is.

### Call sites to switch over for Claude / Phase B
- `Inventory::new()`:
  - `game/tests/animals_tests.rs:25, 59, 98, 135`
  - `game/tests/farming_tests.rs:25, 60`
  - `game/tests/trees_tests.rs:26`
  - `game/tests/vehicles_tests.rs:24`
  - `rotland/src/shooting.rs:284, 341, 392, 469, 507, 615, 684, 735`
  -> Switch to `Inventory::with_registry(shared_items.clone())`
- `Inventory::with_capacity(slots, weight)`:
  - `game/src/dogs/dog.rs:51`
  - `game/src/vehicles/car.rs:48`
  - `game/tests/difficulty_tests.rs:32`
  - `game/tests/dogs_tests.rs:24, 57`
  - `game/tests/loot_tests.rs:33`
  -> Switch to `Inventory::with_capacity_and_registry(slots, weight, shared_items.clone())`
- `Bank::new()`:
  - `game/src/economy/bank.rs:21` (`Default::default`)
  -> Switch callers to `Bank::with_registry(shared_items.clone())` or `Bank::with_slots_and_registry(...)`.

### Wiring for Claude
- File: `rotland/src/main.rs:465` (CRASH-002):
  Replace:
  ```rust
  let world_gen = WorldGenerator::from_data_dir(self.seed, paths::repo_path("data")).expect("data files");
  ```
  With:
  ```rust
  let world_gen = match game::load_world_generator(self.seed, paths::repo_path("data")) {
      Ok(gen) => gen,
      Err(err) => {
          eprintln!("{err}");
          // Or display popup/toast naming err.file_name and err.message
          panic!("{err}");
      }
  };
  ```
  Input: `self.seed: i32`, `paths::repo_path("data")`.
  When: On scene start/reset in `main.rs`.

### Tests
- `paths::tests::bad_dir_returns_error_message`: Verifies pure `find_repo_root_from(None, Some(bad_dir), None)` returns `RepoNotFoundError` containing `ROTLAND_REPO=/path/to/rotland-rust`.
- `game::items::registry::tests::inventory_shared_registry_matches`: Verifies `Inventory::new()` successfully uses process-wide `ItemRegistry::shared()` and has matching slot count (24 from `DEFAULT_SLOT_COUNT`) and registry length.
- `game::loot::roller::tests::empty_loot_table_rolls_nothing_without_panic`: Rolls on a table with 0 entries and rolls [1, 3]. Verifies returns empty `Vec` and 0 cash without panicking.
- `rotland::player_visual::tests::player_visual_picks_fallback_when_mesh_missing`: Tests `pick_model_or_fallback` returning fallback when primary is missing, returning primary when present, and returning None when both missing.

### Checked?
not checked (write-only round)

### Questions / known issues
None.

