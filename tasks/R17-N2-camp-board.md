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

## Neon notes

What was wired

  - CampScreen::Upgrades added in rotland/src/camp_ui/screen.rs (title "Camp
    Board", tab "Upgrades", as_str "upgrades").
  - CampAction::StartUpgrade(UpgradeLine) and CampAction::CancelUpgrade added in
    rotland/src/camp_ui/action.rs.
  - Plain view structs added in rotland/src/camp_ui/view.rs and re-exported from
    rotland/src/camp_ui/mod.rs: CampUpgradesView, UpgradeLineView,
    NextUpgradeView, UpgradeCostItem, UpgradeInProgressView, attached as pub
    upgrades: Option<CampUpgradesView> on CampView<'a>.
  - rotland/src/camp_ui/draw.rs adds the "Upgrades" tab to the existing camp
    screens and dispatches CampScreen::Upgrades to draw_upgrades_screen, which
    uses crate::menus::theme (with_menu_style, palette constants, tab_button,
    menu_button, status_badge, paint_top_accent, paint_divider, ui_scale) with a
    fixed-slot, zero-jump 4-column layout scaling cleanly from 1280x720
    to 2560x1440.
  - The top construction bar uses CampUiState::selected_row
    ("confirm_cancel_upgrade") to handle the two-step "Cancel (refund 50%)" ->
    "Confirm cancel" / "Keep" confirmation without modifying state.rs (which is
    outside the writable file list). Switching tabs or finishing the build
    automatically clears the confirmation state.
  - Unit tests for pure helpers (format_hours, format_building_banner,
    CostRowState, upgrade_progress_fraction, wall HP helpers, refund_percent,
    resolve_in_progress_name, effective_cant_start_reason, CampScreen::Upgrades
    metadata, and 720p-1440p panel sizing) are in
    rotland/src/camp_ui/draw_upgrades.rs under #[cfg(test)] mod tests (run via
    cargo test camp_ui).

What the caller must do

  - Match the new variants CampScreen::Upgrades, CampAction::StartUpgrade(line),
    and CampAction::CancelUpgrade in any exhaustive matches outside camp_ui/
    (and handle them via game::camp_upgrades::{can_start, start, cancel} plus
    wallet/inventory deduction and refund).
  - Populate CampView::upgrades with Some(CampUpgradesView { ... }) when
    rendering camp screens (existing callers using ..Default::default() continue
    to compile with upgrades: None; callers using explicit struct literals need
    upgrades: None or Some(...)).
  - Note on imports: game/src/lib.rs and game/src/camp_upgrades/mod.rs were not
    included in the Read set; this code imports
    game::camp_upgrades::UpgradeLine. If game only re-exports UpgradeLine at the
    crate root (game::UpgradeLine) with a private camp_upgrades module, adjust
    the three use game::camp_upgrades::UpgradeLine; lines in action.rs, view.rs,
    and draw_upgrades.rs.
  - rotland/src/camp_ui/tests.rs was not in the Read/writable set; if it has an
    exhaustive match on CampScreen or CampAction or constructs CampView as a
    full struct literal without ..Default::default(), add the Upgrades /
    upgrades: None entries there. === END ===

## Gate report
- date: 2026-11-21
- base: main
- files changed (22): assets/models/buildings/settlement.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, rotland/src/camp_ui/action.rs, rotland/src/camp_ui/draw.rs, rotland/src/camp_ui/draw_upgrades.rs, rotland/src/camp_ui/mod.rs, rotland/src/camp_ui/screen.rs, rotland/src/camp_ui/view.rs, tasks/R17-N2-camp-board.md, tools/blender/buildings/settlement.py, tools/blender/vehicles/car_parts.py
- tests passed: 0
- clippy: FAIL
- test: SKIP
- checks: SKIP
- gate: FAIL (clippy)
