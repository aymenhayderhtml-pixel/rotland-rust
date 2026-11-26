# R13-08: city loot tables (data only)

Read first: `tasks/R13-common.md`, `docs/CITY_SPEC.md` section 6, `data/loot_tables.json`, `data/items.json` (only to find item ids).
Files: data/loot_tables.json, tasks/R13-08-city-loot.md
Test: loot

Add 8 new tables to `data/loot_tables.json`, in the exact same format as the existing ones (`rolls`, `cash`, `entries` with `id`/`weight`/`count`): `apartment`, `shop`, `office`, `pharmacy` (tier 1-2), `warehouse`, `ruin` (tier 2), `police`, `hospital` (tier 3). Don't change the existing tables.
- Use ONLY item ids that exist in `data/items.json`. Check every id with a one-line python command before you finish, and paste that command's output into Worker notes.
- Tiers (spec section 6): tier 1 = food, water, bandages, cloth, scrap, like `house`. Tier 2 = more medicine, ammo, tools, fuel cans, car parts (if such items exist). Tier 3 = guns and lots of ammo (police), first-aid kits and medicine (hospital). Rolls: tier 1 [2,3], tier 2 [2,4], tier 3 [3,5]. Cash: police and office have more.
- Themes: pharmacy = mostly medicine; shop = food, drinks, small tools; office = little loot, cash, flashlight/batteries if they exist; warehouse = tools, scrap, fuel, crafting materials; ruin = scrap and a little of everything; apartment = like `house` but a bit richer; police = guns, ammo, armor if it exists; hospital = medkits, bandages, pills.
- Then run `tools/gate.sh --task tasks/R13-08-city-loot.md` until `GATE: PASS` (it runs the loot tests, which load the JSON).

## Worker notes
- Added 8 new city loot tables to `data/loot_tables.json`:
  - `apartment` (tier 1-2): rolls [2, 4], cash { "chance": 0.45, "min": 3, "max": 15 }, food/water/bread, bandages, medkit, cloth, scrap, ammo, melee weapons, flashlight, small chance of pistol.
  - `shop` (tier 1-2): rolls [2, 4], cash { "chance": 0.6, "min": 4, "max": 18 }, canned food, soda, water, bread, produce, flashlight, knife, bat, crowbar, bandages, materials, seeds.
  - `office` (tier 1-2): rolls [2, 3], cash { "chance": 0.75, "min": 15, "max": 50 }, little loot (flashlight, drinks, snacks, cloth, scrap, knife), high cash chance and payout.
  - `pharmacy` (tier 1-2): rolls [2, 4], cash { "chance": 0.5, "min": 4, "max": 16 }, heavily weighted towards medicine (bandage, medkit), water, cloth, soda, flashlight.
  - `warehouse` (tier 2): rolls [2, 4], cash { "chance": 0.3, "min": 2, "max": 10 }, tools (crowbar, axe, machete, knife, flashlight), scrap metal, wood, cloth, fuel cans, ammo, water, canned food.
  - `ruin` (tier 2): rolls [2, 4], cash { "chance": 0.25, "min": 1, "max": 8 }, scrap metal, wood, cloth, bandages, canned beans, water, ammo, small tools.
  - `police` (tier 3): rolls [3, 5], cash { "chance": 0.7, "min": 20, "max": 60 }, firearms (pistol, shotgun, smg, rifle), abundant ammo (9mm, 12g, rifle, 7.62), riot bat, knife, flashlight, medical supplies, high cash.
  - `hospital` (tier 3): rolls [3, 5], cash { "chance": 0.4, "min": 5, "max": 25 }, high yield medical supplies (bandages, medkits, cloth gauze), water, soda, canned food, flashlight, knife.
- Validated that all referenced item IDs exist in `data/items.json`:
  Command: `python3 -c "import json; items=set(json.load(open('data/items.json')).keys()); loot=json.load(open('data/loot_tables.json')); missing={t: [e['id'] for e in d['entries'] if e['id'] not in items] for t,d in loot.items()}; print('Missing items:', {k: v for k, v in missing.items() if v}); print(f'Validated all {len(loot)} loot tables: all item IDs exist in data/items.json')"`
  Output:
  ```
  Missing items: {}
  Validated all 15 loot tables: all item IDs exist in data/items.json
  ```
- Wiring notes for Claude:
  - Building scripts / POI / chunk placers can reference table names `"apartment"`, `"shop"`, `"office"`, `"pharmacy"`, `"warehouse"`, `"ruin"`, `"police"`, `"hospital"`.
- Known issues: None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (2): data/loot_tables.json, tasks/R13-08-city-loot.md
- tests passed: 12
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
