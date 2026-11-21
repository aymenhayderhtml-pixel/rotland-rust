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
