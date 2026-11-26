# R37-04: gunsmith screen

Owns: rotland_ui/src/gunsmith_ui/ (new: mod.rs, screen.rs, draw_buy.rs, draw_sell.rs, draw_repair.rs, draw_upgrade.rs, action.rs, tests.rs)
Read: tasks/R37-common.md, rotland_ui/src/camp_ui/draw_shop.rs, screen.rs, action.rs, view.rs (copy the look, fonts, colours, layout rules exactly), rotland_ui/src/hub_ui/item_text.rs (item names/tooltips), how icons are drawn there.

- Title bar: shop name ("Camp Gunsmith" / "Dusthollow Gun Shop") + your scrap count with the scrap icon. Tabs Buy | Sell | Repair | Upgrade (keys 1-4 and click).
- Buy: grid of icons with price; locked items greyed with "Camp tier N". Sell: your bag's sellable items with the scrap you'd get. Repair: your worn weapons with a condition bar and cost; "Repair all". Upgrade: pick a gun on the left, its 5 slots (Optic, Muzzle, Magazine, Grip, Stock) on the right with fitted mod / options + cost; before/after stat bars (damage, spread, kick, mag, noise, zoom).
- Must fit 1920×986 and 1280×720 without overlap (the camp board once overflowed: test both sizes like `camp_ui/tests.rs`).
- The screen takes a plain view-model struct (no game crate types beyond ids/numbers) and returns `GunsmithAction`. Report the mod lines for `rotland_ui/src/lib.rs`.

## Worker notes

### Files changed
Created `rotland_ui/src/gunsmith_ui/`:
- `mod.rs`: plain view-model structs (`GunsmithView`, `GunsmithBuyItem`, `GunsmithSellItem`, `GunsmithRepairItem`, `GunsmithUpgradeWeapon`, `GunsmithSlotState`, `GunsmithFittedMod`, `GunsmithModOption`, `GunsmithWeaponStats`), shared icon/scrap badge helpers (`IconCache` with swatch fallback), and central `draw(ctx, &mut state, &view, icons)` dispatcher.
- `screen.rs`: `GunsmithTab` (`Buy`, `Sell`, `Repair`, `Upgrade`, `parse_name`, `from_key`), `GunsmithUiState`, `GunsmithScreen` (with `handle_key` for `Num1..Num4` and `Escape`), and `GunsmithLayout` responsive bounds calculation.
- `action.rs`: `GunsmithSlot` (`Optic`, `Muzzle`, `Magazine`, `Grip`, `Stock`) and `GunsmithAction` (`Buy`, `Sell`, `Repair`, `RepairAll`, `FitMod`, `RemoveMod`, `Close`).
- `draw_buy.rs`: Buy tab icon grid, scrap prices, `"Camp tier N"` lock badges, hover blockers, and bottom quantity stepper bar.
- `draw_sell.rs`: Sell tab listing bag items with scrap payouts, optional condition/mod tags, quick Sell 1 / Sell all, and quantity stepper.
- `draw_repair.rs`: Repair tab with `"Repair all"` header bar, total scrap cost, `"BROKEN"` badges, condition bars, and per-weapon repair buttons.
- `draw_upgrade.rs`: Upgrade tab with left-column weapon picker, right-column 5 attachment slots (`Optic`, `Muzzle`, `Magazine`, `Grip`, `Stock`) with fitted mod (`Remove`) and options (`Fit` + cost), plus 2x3 before/after stat comparison bars (`Damage`, `Spread`, `Kick`, `Mag`, `Noise`, `Zoom`).
- `tests.rs`: unit tests for tab parsing, key shortcuts (`1`-`4`, `Esc`), state transitions, `GunsmithLayout::compute` bounds at `1920x986`/`1280x720`/`1024x600`, and headless egui `ctx.run_pass` checks across all 4 tabs at `1920x986` and `1280x720`.

### Shared-file lines for Lead
Add to `rotland_ui/src/lib.rs`:
```rust
pub mod gunsmith_ui;
```

### Wiring notes (for R37-08)
- Construct `GunsmithScreen::new(initial_tab)` (e.g. `GunsmithTab::parse_name(flag).unwrap_or_default()`) or hold `GunsmithUiState`.
- Build a `GunsmithView` from inventory, gunsmith stock, attachments, and `effective_stats(...)`. Note: `GunsmithWeaponStats::zoom_x` expects magnification (e.g. `1.0` for iron sights, `base_fov / scope_fov` when scoped).
- Call `screen.draw(ctx, &view, Some(&mut ui.icons))` and handle the returned `Option<GunsmithAction>`.

### Unsure / Not run
- Per subagent rules, `cargo` and `tools/gate.sh` were not executed here; compile/clippy/test verification will run in the lead's gate pass after `pub mod gunsmith_ui;` is added to `rotland_ui/src/lib.rs`.

