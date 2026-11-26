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
