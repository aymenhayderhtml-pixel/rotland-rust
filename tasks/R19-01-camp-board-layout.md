# R19-01: camp board "Upgrades" screen: fix the layout and split the file

Read first: `tasks/R19-common.md`, `rotland/src/camp_ui/draw_upgrades.rs` (1459 lines), `rotland/src/camp_ui/draw_shop.rs` (a screen that looks right, copy its sizing style), `rotland/src/menus/theme.rs` (colors and fonts).
Files: rotland/src/camp_ui/draw_upgrades.rs, rotland/src/camp_ui/upgrades/*, rotland/src/camp_ui/mod.rs, tasks/R19-01-camp-board-layout.md
Test: camp_ui

Problem (screenshot `--camp-screen=upgrades` at 1920x986): the upgrade cards are far too big and get cut off; you can't see all 4 lines.

1. Split `draw_upgrades.rs` into a folder `rotland/src/camp_ui/upgrades/` (e.g. `mod.rs`, `layout.rs`, `card.rs`, `detail.rs`, `colors.rs`), each under 300 lines. Keep `draw_upgrades.rs` only as a thin re-export or delete it and fix `mod.rs`.
2. Layout must fit the screen: compute card sizes from the available rect (`ui.available_size()`), not fixed pixel sizes. 4 upgrade lines (walls & gates, guards & towers, workshop, traders & services) visible at once as 4 columns (or a 2x2 grid when the width is under 1400 px). Each card: line name, current level / max, next level's cost (dollars + items with icons if available), build time, effect text, and the Upgrade button (disabled with the reason when you can't afford it). An in-progress upgrade shows a progress bar and Cancel.
3. Put the size math in a pure function `fn card_layout(avail: egui::Vec2) -> CardLayout` (columns, card width/height, font size) with unit tests for 1920x986, 1280x720 and 1024x600: everything fits inside `avail`.
4. Don't change what the buttons do (`CampAction` values stay the same).

## Worker notes
