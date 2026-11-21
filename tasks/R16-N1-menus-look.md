# R16-N1 (Neon): new look for the main menu, pause menu and settings

Files: rotland/src/menus/menus_draw.rs, rotland/src/menus/menus_draw/menus_draw_settings.rs, rotland/src/menus/menus_draw/menus_draw_slots.rs, rotland/src/menus/theme.rs, tasks/R16-N1-menus-look.md
Read: rotland/src/menus.rs, rotland/src/menus/menus_draw.rs, rotland/src/menus/menus_draw/menus_draw_settings.rs, rotland/src/menus/menus_draw/menus_draw_slots.rs, rotland/src/settings.rs, rotland/src/keys_ui.rs
Test: menus

Rotland is a low-poly 3D zombie survival game (action, not horror; offline, solo). The menus are drawn with egui 0.37 every frame on top of the 3D world. Today they are plain default egui windows. Redesign how they LOOK, keeping what they DO exactly the same.

What to keep:
- Every public function signature and every `MenuAction` the menus return (other code calls `draw(menus, ctx, slots)` and reacts to the actions). Do not edit `menus.rs`, `settings.rs` or `keys_ui.rs` (they are only shown so you know the data).
- All settings stay, with the same ranges and the same fields.
- Keyboard: Esc goes back/closes as today.

The new look:
- A shared theme in a new `rotland/src/menus/theme.rs`: colors (dark charcoal panels with slight transparency, a rust-orange accent like old road signs, off-white text, a muted green for "safe" and a red for "danger"), corner rounding, spacing, button style (wide, tall 44 px, left-aligned label, accent bar on hover), heading font sizes. Apply it with egui `Style`/`Visuals` only inside the menus (restore the previous style after drawing, so the HUD and other screens don't change).
- **Main menu:** the game title "ROTLAND" big at the left third of the screen with a small tagline, a vertical column of buttons under it (Continue / New Game / Load / Settings / Quit, whatever exists today), the world still visible behind (a dark gradient over the left half only). Save slots as cards (slot number, day, playtime, date) with a clear "empty" state.
- **Pause menu:** centered panel, dim the game behind it, buttons Resume / Settings / Save & Quit to menu / Quit (as today), a small line with the day and time if that data is available in what you are shown (else skip).
- **Settings:** tabs across the top (Video, Audio, Controls, Gameplay: group today's settings into them), each setting as a row: label left, control right, a short gray help line under hard ones; Apply/Back at the bottom. Controls tab keeps today's key rebinding UI.
- Works at 1280x720 up to 2560x1440 (scale by screen height), no overlapping text, no text wrapping that makes rows jump.
- No new crates, no images or fonts from files (egui default fonts only), no unused code.

## Neon notes

- Module wiring without editing menus.rs: rotland/src/menus/menus_draw.rs
    loads rotland/src/menus/theme.rs via #[path = "theme.rs"] mod theme; so
    rotland/src/menus.rs stays completely untouched while placing the shared
    theme at the requested path.
  - Scoped egui style: theme::with_menu_style saves ctx.global_style(), applies
    the Rotland dark charcoal / rust-orange Style and Visuals, and restores the
    exact previous style on scope exit (via a Drop guard), so the HUD and other
    screens are unaffected.
  - Public API preserved: menus_draw::draw(menus, ctx, slots), all MenuAction
    variants, #[cfg(test)] pub use menus_draw_slots::{format_play_time,
    format_saved_at}, Esc behavior, and all settings fields/ranges (referencing
    settings.rs constants directly) are unchanged.
  - Pause menu day/time line: Current in-game day and time are not available in
    Menus, egui::Context, or [SlotSummary; 3], so per the task instructions that
    line was skipped.
  - Settings tabs & controls: The active tab and brief "Settings saved"
    confirmation timestamp are stored in egui temporary context memory
    (ctx.data_mut), avoiding any struct changes in menus.rs. Switching tabs
    cancels any in-progress key rebind in menus.keys_ui. Apply clamps, persists
    via menus.save_settings(), and emits MenuAction::SettingsChanged; Back (Esc)
    calls menus.press_back() as before.
  - Resolution scaling: All panels, fonts, margins, and controls scale from
    ctx.content_rect().height() / 720.0 clamped to [0.85, 1.85], keeping text
    single-line/non-jumping across 1280x720 to 2560x1440 while guaranteeing
    keys_ui::draw_keys's fixed 540px inner frame always fits cleanly inside the
    Controls tab.
  - Tests: Added unit tests in rotland/src/menus/theme.rs
    (ui_scale_tracks_screen_height_and_clamps and
    with_menu_style_restores_previous_style), runnable via cargo test menus. ===
    END ===

## Gate report
- date: 2026-11-21
- base: main
- files changed (23): assets/models/buildings/settlement.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, rotland/src/map_ui/mod.rs, rotland/src/map_ui/tests.rs, rotland/src/menus/menus_draw/menus_draw_settings.rs, rotland/src/menus/menus_draw/menus_draw_slots.rs, rotland/src/menus/menus_draw.rs, rotland/src/menus/theme.rs, tasks/R16-N1-menus-look.md, tools/blender/buildings/settlement.py, tools/blender/vehicles/car_parts.py, tools/claim_target.sh
- tests passed: 11
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
