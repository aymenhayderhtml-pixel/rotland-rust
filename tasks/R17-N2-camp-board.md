# R17-N2 (Neon): camp upgrade board screen

Files: rotland/src/camp_ui/draw_upgrades.rs, rotland/src/camp_ui/screen.rs, rotland/src/camp_ui/action.rs, rotland/src/camp_ui/view.rs, rotland/src/camp_ui/draw.rs, rotland/src/camp_ui/mod.rs, tasks/R17-N2-camp-board.md
Read: rotland/src/camp_ui/mod.rs, rotland/src/camp_ui/screen.rs, rotland/src/camp_ui/action.rs, rotland/src/camp_ui/view.rs, rotland/src/camp_ui/state.rs, rotland/src/camp_ui/draw.rs, rotland/src/camp_ui/draw_shop.rs, rotland/src/camp_ui/draw_crafting.rs, rotland/src/menus/theme.rs, game/src/camp_upgrades/types.rs, game/src/camp_upgrades/config.rs, game/src/camp_upgrades/logic.rs, data/camp_upgrades.json
Test: camp_ui

Rotland is a low-poly zombie survival game. The player's camp can be upgraded in 4 lines (walls & gates, guards & towers, workshop, traders & services), each with levels; the logic is done (`game::camp_upgrades`, shown above). Add a new camp screen **Upgrades** (a camp board, opened at the camp like the Shop/Crafting screens) that shows and starts upgrades.

- Add `CampScreen::Upgrades` (title "Camp Board") next to the other screens, a new file `draw_upgrades.rs`, and the new `CampAction` variants it returns: `StartUpgrade(UpgradeLine)` and `CancelUpgrade`. The screen does NOT change any state itself; it returns actions (like the other camp screens do). The caller (written later by Claude) runs `can_start`/`start`/`cancel` and takes the cash/items.
- The view data the screen needs goes in `view.rs` as a plain struct filled by the caller: current level per line, max level per line, per line the next level's name, description, cost (cash + list of (item name, have, need)), build hours, and whether it can start (or the reason it can't as text), the upgrade in progress (line, target level, hours left, total hours), wall HP and max HP, and the player's cash.
- Look: use the shared menu theme in `rotland/src/menus/theme.rs` (colors, button style) so it matches the new menus. Four columns (one per line) as tall cards: line icon (drawn with simple egui shapes: wall, tower, wrench, coin), line name, level pips (filled/empty), a short "what you have now" line, then the NEXT level box: name, 1-2 line description, cost rows (green when you have enough, red when not), build time, a big "Build" button (disabled with the reason as a tooltip/second line when it can't start). Max level: a "Complete" badge instead.
- When an upgrade is building: a bar across the top "Building: Metal walls — 3 h 20 min left" with a progress bar and a small "Cancel (refund 50%)" button that asks for a confirm click.
- Walls card also shows a wall HP bar (and "Repair" is NOT part of this task).
- Must fit 1280x720 to 2560x1440 (scale by screen height), no jumping layout.
- Add tests in the camp_ui tests style for any pure helper (e.g. formatting hours, cost row state). No unused code, no new crates.
