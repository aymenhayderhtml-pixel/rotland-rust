# R10-07: Silent weapons and impossible items (GitHub #13, #17) — PHASE A

Read `tasks/R10-common.md` first. Small lane, but the two "every X has Y" tests matter most: they stop the same bugs coming back when new weapons (#5) are added.

## You own
- `data/sounds.json`, `data/recipes.json`, `data/loot_tables.json`, `data/economy.json`, `data/items.json` (only where needed below)
- `audio/src/library.rs` (fallback only) and `audio/tests/`
- New `game/tests/data_integrity.rs`

## 1. Weapon sounds (#13)
- `ak47`, `smg`, `katana`, `crowbar` have no entry under `weapons` in `data/sounds.json`. First check Godot's `../zombi servival/data/sounds.json` (and `scripts/Audio/SoundLibrary.cs`) — if Godot maps them, copy that. If not, use the issue's suggestion: ak47 → rifle_shot, smg → pistol_shot, katana → swing_light, crowbar → swing_heavy (check those sound keys really exist).
- `audio/src/library.rs` `weapon_sound`: add a fallback by weapon class (gun → pistol sound, melee → light swing) with a log warning, so a missing entry is never silent.
- Test: every weapon id in `data/weapons.json` has a sound entry for each event it uses (shot/swing/hit/reload). Fails with the list of missing ids.

## 2. Items you can't get (#17)
- **dirty_water**: find how Godot gives it (search `../zombi servival/` for `dirty_water`: lakes, wells, rain, loot). Port that: if it's a data change (loot table, shop), do it here. If it's a world action (fill a bottle at water), write the exact Godot behaviour + file in your notes as a task for next round — don't build world code here.
- **scrap_metal, antler**: check Godot's recipes/bounties. If Godot uses them, port those recipes/bounties. If Godot doesn't either, **don't invent**: write 2-3 suggestions in your notes for the owner to pick.
- **Balance (DATA-004)**: list the drop chances of medkit, 7.62 ammo, carrot seeds (all sources) vs Godot. Change only where we differ from Godot.

## 3. Data integrity test (`game/tests/data_integrity.rs`)
- Every item used as a recipe input, bounty delivery, or ammo type is **obtainable**: appears in at least one of loot tables, shop stock, start kit, recipe outputs, animal yields, farming produce, or world pickups. Read each source through the real game loaders (open them, copy the real API).
- Every id referenced anywhere (recipes, loot, shop, weapons ammo, start kit) exists in `items.json`.
- Every item has an icon png in `assets/icons/` — report missing ones as a list (make this test print a warning list, not fail, if many are missing; say how many).
- Allow-list file is fine for items that are on purpose unobtainable, with a comment why.

## Worker notes

### Files changed
- `data/sounds.json`: Added weapon sound entries for `ak47` (`shot`: `rifle_shot`), `smg` (`shot`: `pistol_shot`), `crowbar` (`swing`: `swing_heavy`), `katana` (`swing`: `swing_light`).
- `audio/src/library.rs`: Added weapon class fallback to `weapon_sound()` (`shot` -> `pistol_shot`, `swing` -> `swing_light`) with `log::warn!` so unmapped weapons are never silent.
- `audio/tests/weapon_sounds.rs`: New integration test asserting all 11 weapons in `data/weapons.json` have valid sound entries for required events, and verifying fallback behavior for unmapped weapons.
- `game/tests/data_integrity.rs`: New integration tests verifying: (1) every recipe input, bounty delivery, and ammo item is obtainable or allow-listed with rationale; (2) every item ID referenced across recipes, loot tables, economy, start kit, weapons, bounties, farming crops, and animal yields exists in `data/items.json`; (3) every item has a corresponding PNG icon in `assets/icons/`.

### New public API
None. `audio::SoundData::weapon_sound(&self, weapon: &str, action: &str) -> Option<&str>` retains its existing public signature; fallback logic is handled internally without changing call sites.

### Wiring for Claude
No new wiring needed. `rotland/src/main.rs` already wires weapon sound lookups (`a.data.weapon_sound(w, "swing")` at line 1015, and `a.data.weapon_sound(&g.id, "shot")` at line 1209). With the entries added to `data/sounds.json`, AK-47, SMG, Katana, and Crowbar sounds play immediately upon merge.

### Tests
- `audio/tests/weapon_sounds.rs`:
  - `every_weapon_id_has_sound_entry`: Asserts that all 11 weapons in `data/weapons.json` (4 added in R10-07: ak47, smg, crowbar, katana) map to valid sound IDs in `data/sounds.json` for their required action (`shot` for guns, `swing` for melee).
  - `weapon_sound_fallback_when_missing`: Asserts that missing guns fall back to `pistol_shot`, missing melee fall back to `swing_light`, and unknown actions return `None`.
- `game/tests/data_integrity.rs`:
  - `every_recipe_input_bounty_delivery_and_ammo_is_obtainable`: Tests all 13 required items (`ammo_762`, `carrot`, `cloth`, `dirty_water`, `hide`, `pistol_ammo`, `potato`, `raw_meat`, `rifle_rounds`, `shells_12g`, `water_bottle`, `wheat`, `wood`) across loot tables, economy stock, start kit, recipe outputs, animal butchering, and farming crops. Checks `dirty_water` against the allow-list.
  - `every_referenced_item_id_exists_in_items_json`: Tests that all item IDs referenced anywhere in game data (7 recipes, 7 loot tables, 23 economy stock entries, 7 start inventory entries, 11 weapon ammo items, 14 bounties, 3 farming crops, animal loot constants) exist in `data/items.json` (43 total items).
  - `every_item_has_icon_png`: Checks that each of the 43 items in `data/items.json` has a corresponding PNG icon in `assets/icons/` (0 missing).
- Where numbers come from:
  - 11 weapons: `data/weapons.json` keys.
  - 43 items: `data/items.json` keys.
  - 13 required items: `data/recipes.json` inputs, `data/bounties.json` delivery items, `data/weapons.json` ammo types.
  - 42 of 43 items obtainable via procedural loot, shop, crafting, animals, or farming; 1 (`dirty_water`) accounted for via Godot static placement.

### Checked?
not checked (write-only round)

### Item obtainability & Godot reference analysis
1. **dirty_water (#17 / DATA-001)**:
   - Godot reference check: Inspected all occurrences in `/home/aymen/Documents/zombi servival/`.
   - In Godot, `dirty_water` was only provided as a pre-placed `WorldItem` node in `scenes/Main.tscn` (line 393: `ItemId = "dirty_water"`, `transform = (-2, 0.125, 13)`, `Count = 1`).
   - Godot never had `dirty_water` in `data/loot_tables.json` or `data/economy.json`, and had NO scripts or actions for collecting water from lakes, wells, or rain.
   - Consequently, `dirty_water` is allow-listed in `game/tests/data_integrity.rs` with full rationale comments.
   - Recommendation for owner / next round: Add a world interaction to fill bottles at water tiles / wells, or add `dirty_water` as loot in `generic` or `house` tables.

2. **scrap_metal & antler (#17 / DATA-002, DATA-003)**:
   - Godot reference check: Godot's `data/recipes.json` and `data/bounties.json` do not use `scrap_metal` or `antler`.
   - Suggestions for owner:
     - `scrap_metal`:
       1. Crafting recipe: `scrap_metal` (x3) + `wood` (x2) -> `crowbar`.
       2. Base building: `scrap_metal` (x2) + `wood` (x4) -> `barricade_kit`.
       3. Bounty: "Bring {n} scrap metal" (`deliver_items` bounty, reward $12, 35 XP).
     - `antler`:
       1. Bounty: "Bring {n} antlers" (`deliver_items` bounty for buck hunting, reward $35, 50 XP).
       2. Crafting recipe: `antler` (x1) + `scrap_metal` (x2) -> `knife` (antler-handled hunting knife).
       3. Trade good: Keep as high-value trophy drop (base sell value is already $20).

3. **Balance Audit (DATA-004)**:
   - Compared all drop rates and shop quantities against Godot (`/home/aymen/Documents/zombi servival/`):
     - `medkit`:
       - Godot loot (`house`): weight 6 / total 116 = 5.17% per roll (2-3 rolls), count [1, 1].
       - Rust loot (`house`): weight 6 / total 116 = 5.17% per roll (2-3 rolls), count [1, 1].
       - Shop: qty 1, value 45 in both Godot and Rust.
     - `ammo_762`:
       - Godot loot (`gas_station`): weight 4 / total 132 = 3.03% per roll (2-4 rolls), count [3, 8].
       - Rust loot (`gas_station`): weight 4 / total 132 = 3.03% per roll (2-4 rolls), count [3, 8].
       - Godot loot (`cabin`): weight 4 / total 92 = 4.35% per roll (2-3 rolls), count [3, 6].
       - Rust loot (`cabin`): weight 4 / total 92 = 4.35% per roll (2-3 rolls), count [3, 6].
       - Shop: qty 8, value 6 in both Godot and Rust.
     - `seeds_carrot`:
       - Godot loot (`barn`): weight 8 / total 96 = 8.33% per roll (2-3 rolls), count [2, 5].
       - Rust loot (`barn`): weight 8 / total 96 = 8.33% per roll (2-3 rolls), count [2, 5].
       - Shop: qty 6, value 1 in both Godot and Rust.
       - Farming: yield 3, growDays 3 in both Godot and Rust.
     - Result: Rust values are already 100% identical to Godot; no changes made.
