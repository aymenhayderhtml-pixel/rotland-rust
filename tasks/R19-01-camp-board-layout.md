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
- Files changed:
  - `rotland/src/camp_ui/draw_upgrades.rs`: Reduced to a thin re-export of `super::upgrades::*`.
  - `rotland/src/camp_ui/mod.rs`: Exported `pub mod upgrades;`.
  - `rotland/src/camp_ui/upgrades/card.rs`: Upgrade card renderer displaying line name, current level / max with pips, wall HP, cost rows with icons and dollars, build time, effect text, and Upgrade button.
  - `rotland/src/camp_ui/upgrades/chrome.rs`: Camp Board header (title, tabs, cash) and footer (Close, Doctor, Mechanic, status).
  - `rotland/src/camp_ui/upgrades/colors.rs`: Pure helpers for cost row states, formatting hours/costs/wall HP, progress calculation, and blocker reasons.
  - `rotland/src/camp_ui/upgrades/detail.rs`: Graphical drawing helpers for line icons (walls, guards, workshop, traders), level pips, bars, cost rows, and item color lookups.
  - `rotland/src/camp_ui/upgrades/in_progress.rs`: Card in-progress status, progress bar, and cancellation with confirmation.
  - `rotland/src/camp_ui/upgrades/layout.rs`: Pure `fn card_layout(avail: egui::Vec2) -> CardLayout` computing responsive columns (4 columns at >=1400px width, 2x2 grid under 1400px), card size, gap, and font size, with unit tests for 1920x986, 1280x720, and 1024x600.
  - `rotland/src/camp_ui/upgrades/mod.rs`: Screen wrapper and responsive grid layout dispatcher.
  - `rotland/src/camp_ui/upgrades/tests.rs`: Comprehensive unit tests for formatting, progress, refund, and view defaults.
- How to test:
  - Run `tools/gate.sh --task tasks/R19-01-camp-board-layout.md` (passes all 20 camp_ui tests and lints).
  - Run `cargo test camp_ui` to execute all unit tests.
- Wiring notes for Claude:
  - `draw_upgrades.rs` remains as a thin re-export of `upgrades::*` so `rotland/src/camp_ui/draw.rs` (which imports `draw_upgrades_screen`) compiles without any diff.
  - All files in `rotland/src/camp_ui/upgrades/` are strictly under 300 lines.
- Known issues:
  - None.

## Gate report
- date: 2026-11-22
- base: main
- files changed (11): rotland/src/camp_ui/draw_upgrades.rs, rotland/src/camp_ui/mod.rs, rotland/src/camp_ui/upgrades/card.rs, rotland/src/camp_ui/upgrades/chrome.rs, rotland/src/camp_ui/upgrades/colors.rs, rotland/src/camp_ui/upgrades/detail.rs, rotland/src/camp_ui/upgrades/in_progress.rs, rotland/src/camp_ui/upgrades/layout.rs, rotland/src/camp_ui/upgrades/mod.rs, rotland/src/camp_ui/upgrades/tests.rs, tasks/R19-01-camp-board-layout.md
- tests passed: 20
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
