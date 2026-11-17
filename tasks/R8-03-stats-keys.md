# R8-03: Player profile stats + key rebinding

Read `tasks/R8-common.md` first.

Part A: port `scripts/Player/PlayerStats.cs` to `rust/game/src/profile_stats/`: every counter it keeps (kills, headshots, deaths, distance walked/driven, days survived, etc.), the events that raise them, and Save/Load with the exact Godot keys and system id. The profile page already exists in `rust/rotland/src/hub_ui/draw_profile.rs` and the save crate has `rust/save/src/systems/stats.rs`: read both and make your struct fit what they show/save (if they disagree with Godot, follow Godot and say so). API: `ProfileStats::on_kill(headshot: bool)`, `on_death()`, `add_walked(metres)`, `add_driven(metres)`, `on_day()`, plus getters, `to_save()`/`from_save()`.

Part B: port `scripts/Core/KeyBindings.cs` (every action and its default key, rebinding, clashes, save to settings) into `rust/game/src/input.rs` (you own this file this round; keep `InputBindings` working for its current callers: search for every use first) or a new `rust/game/src/keybinds/`. Map Godot keycodes to winit `KeyCode`. Then `rust/rotland/src/keys_ui.rs`: an egui panel listing every action with its key; clicking a row waits for the next key press and rebinds it; a "Reset to defaults" button. API: `draw_keys(ui, &mut KeyBindings, &mut KeysUiState, pressed_key: Option<KeyCode>) -> bool changed`. Look at `rust/rotland/src/menus/menus_draw/menus_draw_settings.rs` for the style.
Tests (`rust/game/tests/profile_stats_tests.rs`, `keybinds_tests.rs`): counters, save round trip with Godot keys, defaults match Godot, a rebind that clashes swaps or refuses exactly as Godot does.

## Worker notes

### Status: Complete

### Files changed and created
- Created `rust/game/src/profile_stats/mod.rs` (276 lines): Port of `scripts/Player/PlayerStats.cs`. Lifetime player profile stats (`kills`, `kills_by_type`, `headshots`, `deaths`, `highest_day`, `days_survived`, `play_time_seconds`, `distance_walked`, `distance_driven`, horizontal position tracking filtering out teleports >= 5.0m, `to_save()` and `from_save()`).
- Created `rust/game/src/keybinds/mod.rs` (252 lines): Port of `scripts/Core/KeyBindings.cs`. Rebinding system with conflict detection, swapping, reset to defaults, override tracking, token expansion, prompt generation, and settings serialization.
- Created `rust/game/src/keybinds/actions.rs` (273 lines): Data table for all 31 bindable gameplay actions with their display labels, section categories, and default Godot key codes.
- Created `rust/game/src/keybinds/keycode_map.rs` (270 lines): Two-way conversion between Godot integer key codes (with `1 << 25` ShiftMask) and winit `KeyCode` + shift flag, plus string descriptions (`Describe`).
- Modified `rust/game/src/input.rs`: Added re-export `pub use crate::keybinds::*;` and `InputBindings::from_keybindings(&KeyBindings)` / `impl From<&KeyBindings> for InputBindings`. All existing callers remain 100% backward-compatible.
- Modified `rust/game/src/lib.rs`: Added `pub mod keybinds;` and `pub mod profile_stats;` modules and `pub use keybinds::*;`, `pub use profile_stats::*;`.
- Created `rust/rotland/src/keys_ui.rs` (196 lines): egui 0.37 panel listing every action by section with its key binding, interactive rebind on row click ("Press a key..."), Esc cancel, conflict dialog with "Swap" / "Cancel", and "Reset to defaults" button.
- Created `rust/game/tests/profile_stats_tests.rs` (230 lines): 7 integration tests covering initial values, kill counters, deaths, days survived, distances, teleport rejection, save/load round-trips, and Godot JSON loading.
- Created `rust/game/tests/keybinds_tests.rs` (178 lines): 7 integration tests covering defaults against Godot project settings, rebinding, conflict detection & swap, reset all, prompt & token expansion, settings round-trip, and InputBindings conversion.

### Public API
- `game::profile_stats::ProfileStats`:
  - `pub fn new() -> Self` / `impl Default`
  - `pub fn record_kill(&mut self, zombie_id: &str, headshot: bool)`
  - `pub fn on_kill(&mut self, headshot: bool)`
  - `pub fn record_death(&mut self)` / `pub fn on_death(&mut self)`
  - `pub fn on_day(&mut self)` / `pub fn record_day(&mut self, day: i32)`
  - `pub fn add_play_time(&mut self, seconds: f64)`
  - `pub fn add_walked(&mut self, metres: f32)`
  - `pub fn add_driven(&mut self, metres: f32)`
  - `pub fn add_distance(&mut self, metres: f32)`
  - `pub fn update_position(&mut self, pos_xz: glam::Vec2)`
  - `pub fn reset_position(&mut self)`
  - `pub fn kills(&self) -> u32`
  - `pub fn kills_by_type(&self) -> &HashMap<String, u32>`
  - `pub fn headshots(&self) -> u32`
  - `pub fn deaths(&self) -> u32`
  - `pub fn highest_day(&self) -> i32`
  - `pub fn days_survived(&self) -> i32`
  - `pub fn play_time_seconds(&self) -> f64` / `pub fn play_time(&self) -> f64`
  - `pub fn distance_metres(&self) -> f32`
  - `pub fn distance_walked(&self) -> f32`
  - `pub fn distance_driven(&self) -> f32`
  - `pub fn to_save(&self) -> serde_json::Value`
  - `pub fn from_save(&mut self, data: &serde_json::Value)`
  - `pub fn from_save_value(data: &serde_json::Value) -> Self`
  - Constants: `STATS_MAX_STEP_METRES = 5.0`, `STATS_SAVE_ID = "stats"`, `STATS_LOAD_ORDER = 0`.
- `game::keybinds::KeyBindings`:
  - `pub fn new() -> Self` / `impl Default`
  - `pub fn is_bindable(action: &str) -> bool`
  - `pub fn label_of(action: &str) -> &'static str`
  - `pub fn section_of(action: &str) -> &'static str`
  - `pub fn default_of(action: &str) -> u32`
  - `pub fn default_key_of(action: &str) -> Option<(KeyCode, bool)>`
  - `pub fn code_of(&self, action: &str) -> u32`
  - `pub fn key_of(&self, action: &str) -> Option<(KeyCode, bool)>`
  - `pub fn keycode_of(&self, action: &str) -> Option<KeyCode>`
  - `pub fn conflict_with(&self, action: &str, code: u32) -> Option<&'static str>`
  - `pub fn conflict_with_key(&self, action: &str, key: KeyCode, shift: bool) -> Option<&'static str>`
  - `pub fn set(&mut self, action: &str, code: u32)`
  - `pub fn set_key(&mut self, action: &str, key: KeyCode, shift: bool)`
  - `pub fn swap(&mut self, a: &str, b: &str)`
  - `pub fn reset_all(&mut self)`
  - `pub fn overrides(&self) -> HashMap<String, u32>`
  - `pub fn apply(&mut self, overrides: &HashMap<String, u32>)`
  - `pub fn key_name(&self, action: &str) -> String`
  - `pub fn prompt(&self, action: &str, text: &str) -> String`
  - `pub fn describe(code: u32) -> String`
  - `pub fn expand(&self, text: &str) -> String`
  - `pub fn to_settings(&self) -> serde_json::Value` / `pub fn to_save(&self) -> serde_json::Value`
  - `pub fn from_settings(&mut self, data: &serde_json::Value)` / `pub fn from_save(&mut self, data: &serde_json::Value)`
  - Mapping helpers: `godot_code_to_key`, `key_to_godot_code`, `describe_godot_code`.
  - Constants: `KEYBINDS_SHIFT_MASK = 1 << 25`, `KEYBINDS_CODE_MASK = (1 << 25) - 1`.
  - Types: `BindableAction`, `KEYBINDS_ALL_ACTIONS` (31 actions).
- `rotland::keys_ui`:
  - `KeysUiState`: state holding `waiting: Option<String>` and `conflict: Option<(String, String, u32)>`.
  - `draw_keys(ui: &mut egui::Ui, bindings: &mut KeyBindings, state: &mut KeysUiState, pressed_key: Option<KeyCode>) -> bool`
- `game::input::InputBindings`:
  - `pub fn from_keybindings(kb: &KeyBindings) -> Self`
  - `impl From<&KeyBindings> for InputBindings`

### Godot Save & Settings Keys
- `ProfileStats` save format (matches `SaveId: "stats"` and `rust/save/src/systems/stats.rs:StatsSaveData`):
  - `"kills"`: `{ [zombie_id: string]: u32 }`
  - `"headshots"`: `u32`
  - `"deaths"`: `u32`
  - `"highestDay"`: `i32` (defaults to 1, days_survived = highestDay - 1)
  - `"playTime"`: `f64` (seconds)
  - `"distance"`: `f32` (metres, walking + driving)
- `KeyBindings` settings format (matches `scripts/Core/GameSettings.cs` `"keyBindings"`):
  - `"keyBindings"`: `{ [action: string]: u32 }` (only overrides from default are stored, codes carry `1 << 25` shift mask bit).

### Traced Test Numbers
1. `profile_stats_tests.rs`:
   - Initial defaults: `kills` = 0, `headshots` = 0, `deaths` = 0, `highest_day` = 1, `days_survived` = 0, `play_time` = 0.0, `distance` = 0.0.
   - Kill counts: 2 generic unknown (1 headshot) + 2 walker (1 headshot) + 1 runner (0 headshot) = 5 kills total, 2 headshots.
   - Deaths: 0 -> 1 -> 2.
   - Days: day 1 (0 survived) -> day 2 (1 survived) -> day 3 (2 survived) -> day 7 (6 survived) -> record day 4 keeps 7.
   - Distance: walked 15.5m + driven 84.5m = 100.0m total. Negative values (-5.0, -10.0) discarded.
   - Teleport filtering: (0,0) -> (1.5, 2.0) is sqrt(1.5^2 + 2.0^2) = sqrt(2.25 + 4.0) = sqrt(6.25) = 2.5m < 5.0m (accepted, dist = 2.5m). Jump to (100, 100) is ~138m >= 5.0m (rejected as teleport, dist = 2.5m). Walk to (102, 100) is 2.0m < 5.0m (accepted, dist = 4.5m).
   - Godot save JSON payload: walker=10, runner=3, fat=1 (sum = 14 kills), headshots = 7, deaths = 4, highestDay = 8 (days survived = 7), playTime = 7200.5s, distance = 3500.25m.
2. `keybinds_tests.rs`:
   - All 31 actions traced directly from `project.godot`:
     - move_forward: 87 (W)
     - move_back: 83 (S)
     - move_left: 65 (A)
     - move_right: 68 (D)
     - sprint: 4194325 (Shift)
     - jump: 32 (Space)
     - crouch: 67 (C)
     - interact: 69 (E)
     - reload: 82 (R)
     - toggle_camera: 86 (V)
     - quick_save: 4194336 (F5)
     - quick_load: 4194340 (F9)
     - slot_1..6: 49..54 (1..6)
     - inventory: 4194306 (Tab)
     - toggle_skills: 76 (L)
     - toggle_bounties: 74 (J)
     - toggle_map: 77 (M)
     - toggle_profile: 80 (P)
     - hub_prev: 81 (Q)
     - hub_next: 69 (E)
     - dog_stay: 71 (G)
     - dog_home: 71 | 33554432 = 33554503 (Shift+G)
     - look_left: 4194319 (Left)
     - look_right: 4194321 (Right)
     - look_up: 4194320 (Up)
     - look_down: 4194322 (Down)
   - Rebind: Jump to J (74), Jump to Shift+G (33554503).
   - Clash & Swap: E (69) clashes with "interact". Swapping gives Jump=69, interact=32.
   - Settings overrides: rebind crouch to Z (90) stores exactly `{"crouch": 90}` (1 override). Dropped keys in noisy JSON ignored.

### Exact Wiring Steps for Orchestrator (Claude)
1. In `rust/rotland/src/main.rs`:
   - Add `mod keys_ui;` (or `pub mod keys_ui;`).
   - In `App` or `PlayState`, hold a `ProfileStats` instance (or load from `SaveFile.systems.stats`):
     - On game load: `if let Some(s) = sys.stats { stats.from_save(&serde_json::to_value(s).unwrap()); }`
     - On game save (`session::capture`): populate `sys.stats = Some(serde_json::from_value(stats.to_save()).unwrap())`.
     - In gameplay tick / frame:
       - Update player position: `stats.update_position(glam::Vec2::new(player.position.x, player.position.z));`
       - Update play time: `stats.add_play_time(dt);` (skip when paused).
       - When zombie killed: `stats.record_kill(&zombie.kind_id, headshot);`
       - When player dies: `stats.on_death();`
       - When day advances: `stats.record_day(clock.day);`
       - When driving car: `stats.add_driven(car_distance_this_frame);`
   - In `rust/rotland/src/hub_ui/view.rs`:
     - Provide `HubView` stats directly from `ProfileStats`:
       ```rust
       days_survived: stats.days_survived(),
       play_time_seconds: stats.play_time_seconds(),
       kills: stats.kills(),
       kills_by_type: Some(stats.kills_by_type()),
       headshots: stats.headshots(),
       deaths: stats.deaths(),
       distance_metres: stats.distance_metres(),
       ```
2. For Settings / Key rebinding:
   - In `rust/rotland/src/settings.rs`:
     - Add `#[serde(default, alias = "keyBindings")] pub key_bindings: std::collections::HashMap<String, u32>` to `Settings`.
     - When loading settings: `key_bindings.apply(&settings.key_bindings);`
     - When saving settings: `settings.key_bindings = key_bindings.overrides();`
   - In `rust/rotland/src/menus/menus_draw/menus_draw_settings.rs` (or dedicated Controls sub-screen):
     - Instantiate or hold `KeysUiState`.
     - Call `keys_ui::draw_keys(ui, &mut app.key_bindings, &mut app.keys_ui_state, last_pressed_key);`.
     - When `draw_keys` returns `true`, rebuild active `InputBindings`:
       `app.input_bindings = InputBindings::from_keybindings(&app.key_bindings);`
       and mark settings changed.
