# R16-01: camp upgrades core (pure logic + data)

Read first: `tasks/R16-common.md`, `game/src/camp/mod.rs` and `game/src/camp/types.rs` (what a camp is), `game/src/economy/wallet.rs` (money), `data/settlements.json`, `data/economy.json`, `data/items.json` (only to pick existing material ids).
Files: game/src/camp_upgrades/*, game/src/lib.rs, game/tests/all/camp_upgrades_tests.rs, game/tests/all/main.rs, data/camp_upgrades.json, tasks/R16-01-camp-upgrades-core.md
Test: camp_upgrades

Four upgrade lines, each with levels 0 (start) to 3 or 4. Owner: a **steady goal**: level 1 of a line after about 1 hour of play, top levels after many hours; building takes a few in-game hours.
- `walls`: 0 none (today's camp), 1 wood palisade, 2 metal sheet walls, 3 concrete walls; each level: wall HP, gate count, safe-zone half-size growth in metres (today `safeZoneHalf` 17.5 in settlements.json).
- `guards`: 0 today's guards, 1 +1 guard + wooden tower, 2 rifles + range, 3 two towers + spotlight at night; each level: guard count, damage, range, fire interval, tower count.
- `workshop`: 0 hand crafting, 1 workbench (recipe tier 1), 2 tool bench (tier 2: weapons, arrows, repair kits), 3 machine shop (tier 3: car parts, best weapons).
- `traders`: 0 today, 1 more stock slots, 2 doctor (heal for cash), 3 mechanic (repair car, sell fuel), 4 second trader with better stock tier.
Each level costs cash + materials (existing item ids only: e.g. wood/log, scrap_metal, cloth, nails if they exist; check `items.json`) and `buildHours` (in-game hours). Level N needs level N-1 of the same line; some levels need another line (e.g. traders 3 needs workshop 1). All numbers in `data/camp_upgrades.json`.

API (in `game/src/camp_upgrades/`, split into small files):
- `CampUpgradesConfig` (serde, `load_from_file`, `from_json`, `validate()`).
- `CampUpgradeState` (save struct, see common rules): current level per line, the one upgrade under construction (line, target level, hours left), wall HP.
- `can_start(&state, &cfg, line, wallet_cash, has_items: impl Fn(&str, u32) -> bool) -> Result<UpgradeCost, CantStart>` (reasons: max level, missing requirement, busy, not enough cash, missing item).
- `start(&mut state, &cfg, line) -> UpgradeCost` (caller removes cash/items), `tick(&mut state, &cfg, game_hours: f32) -> Option<(UpgradeLine, u8)>` (finished), `cancel` (refund 50%).
- `CampEffects` computed from state + cfg: `safe_zone_half_bonus`, `wall_hp_max`, `guard_count`, `guard_damage`, `guard_range`, `guard_fire_interval`, `tower_count`, `recipe_tier`, `trader_slots_bonus`, `has_doctor`, `has_mechanic`, `trader_stock_tier`.
- Walls: `damage_walls(&mut state, amount)`, `repair_walls(&mut state, &cfg, materials) -> hp restored`; `walls_breached(&state) -> bool`. (Only big night waves damage walls: that rule lives in the zombie director later; here just the HP.)

Tests (`game/tests/all/camp_upgrades_tests.rs`, names start `camp_upgrades_`): config loads and validates; requirements chain; costs reported; can't start two at once; tick finishes after buildHours; cancel refunds half; effects grow with level; save round-trip; `{}` loads as level 0.

## Worker notes

