# R5-05: Camp screens (egui) — Rust port

Read `tasks/R5-common.md` (rules) — except this task writes UI in `rust/rotland/src/camp_ui/` (egui 0.37, like `rust/rotland/src/hud.rs` and `rust/rotland/src/menus/`), not in the game crate.

Port the screens, drawing from the already-ported game logic (read their public APIs first: `rust/game/src/economy/`, `crafting/`, `bounties/`, `items/`):
- `scripts/UI/ShopScreen.cs` + `ShopRow.cs`: trader buy/sell lists with prices, money, weight.
- `scripts/UI/BankScreen.cs`: deposit/withdraw items and dollars.
- `scripts/UI/CraftingPanel.cs` + `CraftingRecipeRow.cs`: recipes, can-craft state, progress bar.
- `scripts/UI/BountyScreen.cs` + `BountyTracker.cs`: offered/active bounties, accept, turn in; the small on-HUD tracker.
- One `pub enum CampScreen { Shop, Bank, Crafting, Bounties }` and `pub fn draw(ctx, screen, &mut CampUiState, view: &CampView) -> Option<CampAction>`; `CampAction` lists every button the player can press (Buy{item,count}, Sell{..}, Deposit{..}, Withdraw{..}, Craft{recipe}, Accept{bounty}, TurnIn{bounty}, Close). The screen never changes game state itself: Claude applies the actions. `CampView` holds read-only references to what is shown.
- Keep the menus' style (dark panels, same fonts/sizes as `menus_draw.rs`). Texts and colours from the `.cs`/`.tscn` files.
- `mod x;` inside `a.rs` means `a/x.rs`: use `camp_ui/mod.rs` with the submodules in `camp_ui/`. Add `mod camp_ui;` to `main.rs` only; do not wire it (Claude does).
Tests (`camp_ui/tests.rs`, `#[cfg(test)]`): CampUiState logic only (tab switching, selected row, count steppers clamp to stock/money/space), not drawing.
