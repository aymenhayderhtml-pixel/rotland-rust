# R4-04: Menus (Rust port)

Port Rotland's menus to the Rust game, drawn with egui 0.37. Write-only task: you write code and tests, Claude builds and tests once after merging.

## Read first
- `rust/README.md` (rules for workers), `docs/JESTER.md` (past worker mistakes: do not repeat them).
- Godot sources to port: `scripts/UI/MainMenu.cs`, `PauseMenu.cs`, `SettingsMenu.cs`, `SettingsRows.cs`, `SlotPicker.cs`, `scripts/Core/GameSettings.cs`, `scripts/Core/SettingsStore.cs`, `scripts/Core/SettingsService.cs`.
- Rust example of egui use: `rust/rotland/src/hud.rs` (how the HUD draws), `rust/engine/src/app.rs` (Game trait: `ui`, `cursor_grab`, `should_quit`).

## What to write
1. `rust/rotland/src/settings.rs`: `Settings` struct (serde) with the GameSettings fields that make sense here (mouse sensitivity, invert Y, FOV, render scale, vsync, master/music/sfx volume, show FPS, graphics preset name). `Settings::load(path)` / `save(path)`; a missing or broken file gives defaults (log a warning, never panic). `settings_path()` returns `<data_dir>/rotland-rust/settings.json` where data_dir is `$XDG_DATA_HOME` or `~/.local/share`. Tests use a temp folder only (`std::env::temp_dir()` + unique name), never the real path.
2. `rust/rotland/src/menus.rs`: a pure menu state machine plus egui drawing.
   - `enum Screen { Main, SlotPicker { mode: SlotMode }, Settings { from: Box<Screen> }, Paused, None }` (shape is up to you; keep it simple).
   - `pub struct Menus` with `pub fn ui(&mut self, ctx: &egui::Context, slots: &[SlotSummary; 3]) -> Option<MenuAction>` that draws the current screen and returns what the player picked.
   - `enum MenuAction { NewGame { slot: u8 }, Continue { slot: u8 }, DeleteSlot { slot: u8 }, Resume, SaveAndQuitToMenu, Quit, SettingsChanged(Settings) }`.
   - `pub struct SlotSummary { pub state: SlotState /* Empty, Damaged, Filled */, pub day: i32, pub play_time_s: f32, pub difficulty: String, pub saved_at: String }` (the saves task fills it; define it here in menus.rs).
   - `pub fn on_escape(&mut self)`: in game -> Paused; Paused -> resume (Screen::None); Settings -> back to where it came from.
   - `pub fn wants_cursor(&self) -> bool` (true while any menu is open), `pub fn is_open(&self) -> bool`.
   - Main menu buttons and texts as in MainMenu.cs (Continue disabled when no slot is filled). Slot picker as in SlotPicker.cs (day, play time "h:mm", difficulty, saved date; delete asks to confirm). Pause menu as PauseMenu.cs. Settings rows as SettingsRows.cs (sliders/checkboxes).
   - Keep files under ~300 lines; split drawing into `menus_draw.rs` if needed.
3. Unit tests (in `rust/rotland/src/menus_tests.rs` or `#[cfg(test)]`) for the state machine only: Escape transitions, Continue disabled with all slots empty, delete-confirm flow, settings back-navigation, settings round-trip through a temp file. Do not test egui drawing.
4. Add `serde`/`serde_json` to `rust/rotland/Cargo.toml` with `.workspace = true` if missing. Add `mod settings; mod menus;` to main.rs only (do not wire into the game loop: Claude does that).

## Rules (from past mistakes)
- egui 0.37: copy function names from the egui source in `~/.cargo/registry/src/*/egui-0.37*/src` (grep `pub fn`). `Context::run` is now `run_pass`, `screen_rect` is `content_rect`. Do not guess older APIs.
- Use `paths::repo_path` for repo files; never `env!("CARGO_MANIFEST_DIR")`.
- Never touch the owner's real Godot saves (`~/.local/share/godot/app_userdata/Rotland/saves/`) or the real Rust settings path in tests.
- Do not run cargo build or tests (Claude does it once). So re-check every expected value in tests by tracing your code by hand. Do not report test output you did not see.
- Report: files changed, public API, anything left out. Commit to your branch in one commit at the end.

## Worker notes
- Ported Settings (`rust/rotland/src/settings.rs`) with serde serialization/deserialization, clamping, default fallback, and safe path resolution.
- Ported Menu state machine (`rust/rotland/src/menus.rs`) and modular egui 0.37 drawing (`menus_draw.rs`, `menus_draw_slots.rs`, `menus_draw_settings.rs`), all kept under 300 lines each.
- Added pure unit tests (`rust/rotland/src/menus_tests.rs`) covering Escape transitions, Continue disabled when empty, delete-confirmation flow, settings back-navigation, temp-file settings round-tripping, and clamping.
- Added `serde`, `serde_json`, and `egui` workspace dependencies to `rust/rotland/Cargo.toml`.
- Added `mod settings;` and `mod menus;` to `rust/rotland/src/main.rs`.
