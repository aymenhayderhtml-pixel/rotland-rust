# R37-03: gunsmith shop logic

Owns: game_camp/src/gunsmith/ (new: mod.rs, config.rs, stock.rs, actions.rs), data/gunsmith.json (new), game/tests/all/gunsmith_tests.rs (new)
Read: tasks/R37-common.md, game_camp/src/economy/trader.rs + config.rs + rules.rs (copy the stock/restock/save pattern), game_core/src/items/ (bag add/remove API), data/economy.json, data/weapons.json, data/items.json, data/camp_upgrades.json ("traders" tiers: copy that shape for gunsmith tiers).

- `data/gunsmith.json`:
  - prices in scrap for every gun, ammo pack, grenade, bolt bundle and attachment (rough guide: pistol 25, revolver 35, shotgun 45, smg 60, rifle 60, hunting_rifle 90, ak47 110, crossbow 70, ammo packs 3-8, grenade 12, attachments 15-60);
  - camp tiers 0-3 (stock grows with the camp upgrade level: tier 0 pistol + ammo + repair only; tier 3 most guns and attachments);
  - town shops (dusthollow, frostmere, blackwater, ridgeback_station): each has its own flavour and 1-2 exclusive items (e.g. ridgeback_station `scope_4x` + `hunting_rifle`; frostmere `drum_mag`);
  - `sellRate` 0.4 of price (loot sold for scrap), repair = `repairRate` × price × (1 - condition), fit cost = attachment price + small labour.
- Logic fns from the shared names; scrap moves as `scrap_metal` stacks in the bag (stack size 20, split correctly); refuse when not enough scrap or bag full (clear error enum for the UI).
- Removing a mod returns the attachment item; selling a modded gun returns its mods to the bag first.
- Save: `GunsmithStock` per shop, restock each morning (like `TraderStock`). Report the save id lines for the lead.
- Tests: buy/sell/repair/fit/remove happy paths and every refusal; tiers unlock stock; town exclusives only in their town; scrap splitting across stacks.

## Worker notes

### Files changed (owned only, in `/home/aymen/Documents/rotland-r37`)
- `data/gunsmith.json` (new): scrap prices (all weapons, ammo packs, grenades, bolt/arrow bundles, and the 9 attachments), `sellRate` 0.4, `repairRate` 0.5, `fitLabour` 5, `restockHour` 6, `interactRange` 4.0, `campStock` tiers 0–3 (tier 0 = `pistol` + `pistol_ammo`), and 4 town shops (`dusthollow` exclusive `katana`; `frostmere` exclusive `drum_mag`; `blackwater` exclusive `bolt_quiver`; `ridgeback_station` exclusives `hunting_rifle` + `scope_4x`).
- `game_camp/src/gunsmith/mod.rs` (new): module exports.
- `game_camp/src/gunsmith/config.rs` (new): `GunsmithConfig` / `GunsmithConfigData`, `GunsmithPriceEntry`, `CampStockEntry`, `TownShopConfig`, `GunsmithLoadError`, validation against `ItemRegistry`.
- `game_camp/src/gunsmith/stock.rs` (new): `GUNSMITH_SAVE_ID` (`"gunsmith"`), `ShopKind { Camp { tier: u8 }, Town { key: String } }`, `ShopStock`, `GunsmithStock` (camp + 4 town shops, morning restock, save/load).
- `game_camp/src/gunsmith/actions.rs` (new): `GunsmithError`, `BuyResult`, `SellResult`, `buy_check`, `buy`, `sell_value`, `sell`, `sell_slot`, `repair_cost`, `repair`, `repair_slot`, `repair_all_cost`, `repair_all`, `fit_cost`, `fit`, `fit_at_shop`, `fit_slot`, `fit_slot_at_shop`, `remove_mod`, `remove_mod_slot`.
- `game/tests/all/gunsmith_tests.rs` (new): tests for config validation, camp tiers, town exclusives, buy/sell/repair/fit/remove happy paths and refusals (`TierLocked`, `NotInShop`, `SoldOut`, `NotEnoughScrap`, `BagFull`, `TooHeavy`, `CannotSell`, `NotAWeapon`, `AlreadyRepaired`, `IncompatibleAttachment`, `AlreadyFitted`, `SlotOccupied`, `ModNotFitted`), scrap stack splitting (`maxStack: 20`), returning mods on sell, morning restock, and save roundtrip.

### Shared-file lines for the Lead
1. `game_camp/src/lib.rs`:
   ```rust
   pub mod gunsmith;
   pub use gunsmith::*;
   ```
2. `game/src/lib.rs`:
   - In `pub use game_camp::{ ... };` add `gunsmith,`
   - Add `pub use game_camp::gunsmith::*;`
3. `game/tests/all/main.rs`:
   ```rust
   mod gunsmith_tests;
   ```

### Save id & Camp wiring lines (for R37-08 / Lead)
- Save id: `game::GUNSMITH_SAVE_ID` (`"gunsmith"`), stored in `sys.extra`:
  - Save: `sys.extra.insert(game::GUNSMITH_SAVE_ID.into(), self.gunsmith.to_save());`
  - Load: `if let Some(v) = sys.extra.get(game::GUNSMITH_SAVE_ID) { self.gunsmith.from_save(v); }`
  - Morning tick: `self.gunsmith.tick_with_config(day, hour as u32, &self.gunsmith_config);`

### Notes for R37-04 / R37-08 & assumptions
- Built against shared names from `R37-common.md`, `R37-01-wear.md`, and `R37-02-attachments.md` (`ItemStack.wear: Option<f32>`, `ItemStack.mods: Vec<String>`, `ItemStack::condition(&self) -> f32`, `ItemStack::has_mod(&self, id: &str) -> bool`, `AttachmentRegistry::from_repo()`, `AttachmentRegistry::get(&self, id: &str) -> Option<&AttachmentDef>`, `AttachmentDef { id, name, slot, fits, model, price_scrap, effects }`). Not compiled locally per subagent rules.
- `fit` / `fit_at_shop` check if the player already carries `attachment_id` in `bag`: if so, it consumes 1 item from `bag` and charges only `cfg.fit_labour` (5 scrap); otherwise it charges `fit_cost` (`price + fit_labour`) and (`fit_at_shop`) takes 1 from shop stock.
- `sell` / `sell_slot` check capacity atomically on a cloned bag after removing the sold item, returning fitted `mods` to the bag first and then adding `scrap_metal`.

