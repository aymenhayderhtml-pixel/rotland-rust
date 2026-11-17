# R5-01: Economy (wallet, bank, trader) and crafting — Rust port

Read `tasks/R5-common.md` first.

Port to `rust/game/src/economy/` and `rust/game/src/crafting/`:
- `scripts/Economy/*.cs` (EconomyConfig/Data from `data/economy.json`, Wallet, Bank, TraderStock, StockEntry, TradeRules): buy and sell prices, stock and restock over game days, bank deposit/withdraw, start dollars, death penalty hook (wallet emptied).
- `scripts/Crafting/*.cs` (RecipeRegistry from `data/recipes.json`, RecipeDef, RecipeItem, CraftingSystem, RecipeXp): can-craft checks against a `game::items::Inventory`, crafting removes inputs and adds outputs (respect stacks/weight via the existing Inventory API in `rust/game/src/items/`), XP rewards returned as data.
- Godot save ids: "wallet", "bank", "trader" (check each `.cs`).
Tests: price math on real data (count items), buy/sell moves money and items, cannot buy without money/space, restock after a day, bank round trip, craft success/fail, save round trip for wallet/bank/trader.
