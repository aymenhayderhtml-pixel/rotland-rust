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
