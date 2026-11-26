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

### Files changed
- `data/camp_upgrades.json`: Complete specification of all 4 upgrade lines (walls 0..=3, guards 0..=3, workshop 0..=3, traders 0..=4), costs (dollars, wood, scrap_metal, cloth), build hours, cross-line requirements (traders 3 -> workshop 1), stat bonuses, refund ratio (0.5), and repair rates.
- `game/src/camp_upgrades/types.rs`: `UpgradeLine`, `MaterialCost`, `UpgradeCost`, `UpgradeRequirement`, `UpgradeInProgress`, `CampUpgradeState` (serde save struct with defaults), `CantStart`, `CampEffects`.
- `game/src/camp_upgrades/config.rs`: `CampUpgradesConfig`, level definitions, loader (`load_default`, `load_from_file`, `from_json`), `validate`, query helpers, and `effects()` calculation.
- `game/src/camp_upgrades/logic.rs`: Core logic functions and state methods for `can_start`, `start`, `tick`, `cancel`, `damage_walls`, `repair_walls`, `walls_breached`.
- `game/src/camp_upgrades/mod.rs`: Public re-exports for the module.
- `game/src/lib.rs`: Registered and exported `camp_upgrades`.
- `game/tests/all/main.rs`: Registered `camp_upgrades_tests`.
- `game/tests/all/camp_upgrades_tests.rs`: Integration tests covering config validation, requirements chain, costs, busy lock, tick completion, refund on cancel, effect growth, serialization roundtrip, default from empty JSON, and wall combat/repair mechanics.

### How to test
Run the gate:
`tools/gate.sh --task tasks/R16-01-camp-upgrades-core.md`
Or filter tests:
`cargo test camp_upgrades`

### Wiring notes for Claude
- State management:
  - `CampUpgradeState` derives `Serialize, Deserialize, Default`. Fields: `walls_level`, `guards_level`, `workshop_level`, `traders_level`, `in_progress: Option<UpgradeInProgress>`, `wall_hp: f32`.
  - Empty json `{}` restores as level 0 with 0 HP and no active construction.
  - Integrate into world / settlement save state as a sub-struct.
- Game loop integration:
  - Load config once at startup: `let cfg = CampUpgradesConfig::load_default()?;`
  - In settlement / camp tick, advance construction timer by in-game hours:
    ```rust
    if let Some((line, new_level)) = state.tick(&cfg, dt_hours) {
        // Log notification or announce upgrade completion
    }
    ```
  - Starting an upgrade:
    ```rust
    // Check first:
    let cost = state.can_start(&cfg, line, wallet.dollars(), |item_id, count| inventory.count(item_id) >= count)?;
    // Deduct cash and items:
    wallet.try_spend(cost.cash);
    inventory.remove_items(&cost.materials);
    state.start(&cfg, line);
    ```
  - Cancelling:
    ```rust
    if let Some(refund) = state.cancel(&cfg) {
        wallet.add(refund.cash);
        for m in refund.materials {
            inventory.add(&m.item_id, m.count);
        }
    }
    ```
  - Accessing current effects:
    `let effects: CampEffects = cfg.effects(&state);`
    Use `effects.safe_zone_half_bonus` to expand safe zone, `effects.wall_hp_max` for walls, `effects.guard_count`, `effects.guard_damage`, `effects.guard_range`, `effects.guard_fire_interval`, `effects.tower_count`, `effects.recipe_tier` for crafting station unlocks, `effects.trader_slots_bonus`, `effects.has_doctor`, `effects.has_mechanic`, and `effects.trader_stock_tier`.
  - Walls & damage:
    - During night waves: `state.damage_walls(dmg)`
    - Check breach: `if state.walls_breached() { ... }`
    - Repair walls: `let hp_restored = state.repair_walls(&cfg, &[("wood", 2), ("scrap_metal", 1)]);`

### Known issues
None. All 10 tests pass cleanly, clippy is clean with 0 warnings, scope and unused checks pass.

## Gate report
- date: 2026-11-21
- base: main
- files changed (9): data/camp_upgrades.json, game/src/camp_upgrades/config.rs, game/src/camp_upgrades/logic.rs, game/src/camp_upgrades/mod.rs, game/src/camp_upgrades/types.rs, game/src/lib.rs, game/tests/all/camp_upgrades_tests.rs, game/tests/all/main.rs, tasks/R16-01-camp-upgrades-core.md
- tests passed: 10
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
