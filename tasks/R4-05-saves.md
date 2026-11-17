# R4-05: Save slots (Rust port)

Port Rotland's save system to the Rust game in a way that reads the Godot save files (same JSON shape), so the owner's Godot saves can be imported. Write-only task: you write code and tests, Claude builds and tests once after merging.

## Read first
- `rust/README.md`, `docs/JESTER.md` (past worker mistakes: do not repeat them).
- Godot sources: `scripts/Core/SaveSystem.cs`, `SaveFile.cs`, `SaveSlots.cs`, `SaveData.cs`, `ISaveable.cs`, and the save code (ToDictionary/FromDictionary or Save/Load) of: `scripts/Player/PlayerSave.cs`, `scripts/Player/PlayerStats.cs`, `scripts/Items/Inventory.cs` (and the hotbar save), `scripts/World/TimeOfDay.cs`, `scripts/World/WorldState.cs` (seed, opened crates, dropped items), `scripts/Difficulty/WorldDifficulty.cs`, `scripts/Economy/Wallet.cs`.
- Rust state that must be saved: `rust/rotland/src/play.rs` (inventory, hotbar, loot_state, world_items, world_seed), `rust/game/src/items/*` (Inventory, Hotbar, ItemStack), `rust/game/src/loot/*`, `rust/world/src/time.rs` (Clock), `rust/game/src/player.rs`, `rust/rotland/src/combat.rs` (health).

## What to write
1. New crate `rust/save` (add to workspace members) with serde + serde_json:
   - `SaveFile` = the Godot root shape: `{ "version": int, "savedAtUtc": string, "systems": { "<key>": {...} } }`. Keep unknown systems as `serde_json::Value` so a Godot file round-trips without losing camp/farm/dog data the Rust game does not have yet.
   - Typed structs for the systems listed above, using the exact Godot key names (check each `.cs` file; don't guess). Missing keys take defaults like `SaveData.GetFloat(..., fallback)`.
   - `Slots { base_dir: PathBuf }` with `path_for(slot)`, `info(slot) -> SlotInfo { state: Empty|Damaged|Filled, seed, day, play_time_s, difficulty, saved_at }` (same rules as `SaveSlots.Info`), `read(slot)`, `write(slot, &SaveFile)` (write to a temp file then rename, like SaveFile.cs), `delete(slot)`.
   - `default_base_dir()` = `<XDG_DATA_HOME or ~/.local/share>/rotland-rust/saves` (the Rust game's own folder).
   - `godot_save_dir()` = `~/.local/share/godot/app_userdata/Rotland/saves`. `import_godot(slot, &Slots)` copies a Godot slot into a Rust slot by READING the Godot file only. Never write, rename or delete anything under the Godot folder.
   - Guard: `write` and `delete` refuse (return an error) any path under `godot_save_dir()`; in `cfg(test)` they also refuse `default_base_dir()`.
2. `rust/save/src/from_game.rs` is NOT needed: keep the crate free of the `game`/`rotland` crates. Provide plain data structs; Claude maps game state to them.
3. Tests (`rust/save/tests/`): round-trip a save through a temp folder (`std::env::temp_dir()` + unique name, removed at the end); slot info for empty/damaged/filled; a Godot-shaped JSON fixture you write by hand from the `.cs` code (put it in `rust/save/tests/fixtures/`) parses into the typed structs with the right values and unknown systems survive a write/read; the guard refuses the Godot folder. Never open the owner's real save files in tests.

## Rules (from past mistakes)
- Key names come from the `.cs` files themselves; Claude will check them against the code.
- Comparison tests must count what they compared and fail on zero.
- The default constructor must give real defaults, never quietly empty data that hides a bug.
- Do not run cargo build or tests (Claude does it once). Re-check every expected value by tracing the code by hand; never report output you did not see.
- Use `paths::repo_path` for repo files; never `env!("CARGO_MANIFEST_DIR")` (fixtures in tests may use `env!("CARGO_MANIFEST_DIR")` only inside `#[cfg(test)]`/tests, since tests run from the checkout).
- Report: files changed, public API, the exact Godot keys used per system, anything left out. Commit to your branch in one commit at the end.

## Worker notes
- Files created/changed:
  - `rust/Cargo.toml` (added `save` to workspace members)
  - `rust/save/Cargo.toml`
  - `rust/save/src/lib.rs`
  - `rust/save/src/error.rs`
  - `rust/save/src/file.rs`
  - `rust/save/src/slots.rs`
  - `rust/save/src/systems/mod.rs`
  - `rust/save/src/systems/player.rs`
  - `rust/save/src/systems/stats.rs`
  - `rust/save/src/systems/inventory.rs`
  - `rust/save/src/systems/time.rs`
  - `rust/save/src/systems/world.rs`
  - `rust/save/src/systems/worldstate.rs`
  - `rust/save/src/systems/difficulty.rs`
  - `rust/save/src/systems/wallet.rs`
  - `rust/save/tests/fixtures/godot_save.json`
  - `rust/save/tests/saves.rs`
- How to test:
  - `cargo test -p save` (all tests count assertions and fail on zero).
- Known issues:
  - None.

