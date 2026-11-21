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

