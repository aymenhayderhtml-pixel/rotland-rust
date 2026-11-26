# R37-07: gunsmith in camp, gun shops in towns

Owns: data/settlements.json (camp NPC/bench spot), data/camp_upgrades.json (a `gunsmith` tier track, like `traders`), data/towns.json (one `gun_shop` lot per town + footprint), world/src/towns/lots.rs (guarantee one gun_shop lot), game_camp/src/camp/npc.rs + resident.rs (the `npc_gunsmith` resident), data/camp_life.json (gunsmith lines/schedule if residents use it)
Read: tasks/R37-common.md, how the camp trader NPC and its stall are placed (grep `npc_trader` in data/ and game_camp/, world/src/camp_layout/), game_camp/src/camp_upgrades/, world/src/towns/ (mod.rs, lots.rs, interior.rs), data/camp_upgrades.json "traders".

- Camp: the gunsmith bench + NPC next to the trader area, inside the walls, not blocking paths (check the camp layout); golden test must still pass: if the camp layout is checked by `world/tests/golden.rs`, place the bench as an upgrade prop (like the R19 camp upgrade props), never move existing camp objects.
- Camp upgrade track `gunsmith` tiers 0-3 (costs like `traders`), tier unlocks stock in R37-03.
- Towns: each town gets exactly one `gun_shop` lot near its centre on the main street, with a `shopkeeper_<town_key>` NPC inside behind the counter (data spot in building-local space). Shopkeepers are friendly survivors (zombies don't target them inside; copy how rescue survivors/NPCs are kept safe if needed, else report).
- Tests: every town has one gun_shop; camp has the gunsmith spot; tiers load.

## Worker notes
### Files changed (all in `/home/aymen/Documents/rotland-r37`)
- `data/settlements.json`: added `"npc_gunsmith": "npc_mechanic"` under `npcModels`.
- `data/camp_layout.json`: added `"gunsmithOffset": [-14.0, -7.5]`, `"gunsmithYaw": 3.1415927` (bench faces south `+Z` into the yard, beside `trader_stall` and clear of `camp_doctor`, `woodpile`, and all walk/work/bed points).
- `data/camp_upgrades.json`: added `"gunsmith"` upgrade track (tiers 0..=3: `Field Bench`, `Reloading Press`, `Armory Rack` requiring `workshop` lv 1, `Master Gunsmith`), each with `gunsmithTier`.
- `data/towns.json`:
  - Added `"gun_shop": [6.0, 5.0]` under `footprints` (synced with R37-05's 8.0 × 7.0 m `gun_shop.py`).
  - Added `{ "model": "gun_shop", "weight": 1.0, "unique": true }` and `"gun_shop": "town_station"` to `buildings`/`loot_tables` for all 4 towns (`dusthollow`, `frostmere`, `blackwater`, `ridgeback_station`).
  - Added `"shopkeeper": { "building": "gun_shop", "x": -0.9, "y": -2.45, "floor_y": 0.1, "yaw": 0.0 }` matching `npc_1` behind the counter in `tools/blender/buildings/gun_shop.py` (did NOT add `gun_shop` to `interiors.buildings`, since `gun_shop.py` already bakes its counter/racks/crates).
- `world/src/camp_layout/types.rs`: added `pub gunsmith: u8` (`#[serde(default)]`) to `CampLevels`; added `gunsmith_offset` & `gunsmith_yaw` (with defaults) to `CampLayoutConfigData`.
- `world/src/camp_layout/buildings.rs`: added `camp_gunsmith_prop` and `camp_gunsmith_prop_with_config` returning `PropInstance::new("gunsmith_bench", ...)` (kept separate from `camp_upgrade_props` so `world/tests/golden.rs` and `camp_layout_level_0_empty` / `test_level_0_has_no_props` stay 100% intact while tier-0 camp still has the bench).
- `world/src/camp_layout/tests.rs`: updated `CampLevels` literals, included `gunsmith_bench` (`1.25 × 0.75` half-extents) in `camp_layout_nothing_overlaps`, and added `camp_layout_gunsmith_bench_placed_and_rotates`.
- `world/src/towns/mod.rs`: added `GUN_SHOP_MODEL`, `ShopkeeperSpot`, `TownShopkeeper`, `TownsData::shopkeeper`, `TownSite::gun_shop_lot()`, and `TownSite::shopkeeper_spawn(&data, ground_y)`.
- `world/src/towns/lots.rs`: ensured `GUN_SHOP_MODEL` is always placed at slot 0 (centermost main-street lot) in `generate_lots`, and added `every_town_has_one_gun_shop_near_center_and_shopkeeper_spot`.
- `game_camp/src/camp/npc.rs`: added `"gunsmith"` to `prompt_for_role` and `talkable`; added `npc_gunsmith` (`role: "gunsmith"`, `model_id: "npc_mechanic"`, local `(-12.3, 0.0, -7.2)` beside the bench) to `SettlementNpc::default_settlement_npcs`.
- `game_camp/src/camp/resident.rs`: added `"gunsmith"` to `CampResident::can_interact`.
- `game_camp/src/camp_upgrades/types.rs` & `config.rs`: added `UpgradeLine::Gunsmith` (`ALL` now length 5), `CampUpgradeState::gunsmith_level`, `CampEffects::gunsmith_tier`, `GunsmithLevelDef`, `CampUpgradesConfig::gunsmith`, and validation (`len >= 4`, sequential levels).
- `game/tests/all/camp_tests.rs`: added `camp_has_gunsmith_npc_and_is_interactable`.
- `game/tests/all/camp_upgrades_tests.rs`: updated `CampUpgradeState` literal, asserted `UpgradeLine::Gunsmith` max level 3 & `gunsmith_tier` 0..=3, and added `camp_upgrades_gunsmith_tiers_0_to_3`.
- Note on `data/camp_life.json`: left unchanged because `npc_gunsmith` is a fixed `SettlementNpc` (like `npc_trader` and `npc_banker`), not a wandering `CampResident`, and `game/tests/all/camp_tests.rs` asserts `data.chat_lines.len() == 30` and deterministic chat RNG.

### Shared-file lines needed by Lead
1. **`world/src/camp_layout/mod.rs`**:
   - Line 10: `pub use buildings::{camp_gunsmith_prop, camp_gunsmith_prop_with_config};`
   - (Then `#[allow(dead_code)]` on those two fns in `buildings.rs` can be removed if desired.)
2. **`world/src/lib.rs`**:
   - Under `pub use camp_layout::{ ... }`: add `camp_gunsmith_prop, camp_gunsmith_prop_with_config,`
   - Under `pub use towns::{ ... }`: add `ShopkeeperSpot, TownShopkeeper, GUN_SHOP_MODEL,`
3. **`game_camp/src/camp_upgrades/mod.rs`**:
   - Under `pub use config::{ ... }`: add `GunsmithLevelDef,`
4. **`save/src/systems/settlements.rs`** (`CampUpgradesSaveData`, already `#[serde(default)]` so old saves load as 0):
   - Add `pub gunsmith_level: u8,`
5. **`save/tests/saves.rs`**:
   - Line 640 in `roundtrip_camp_upgrades`: add `gunsmith_level: 2,` and `assert_eq!(s.camp_upgrades.gunsmith_level, 2);`.
6. **`rotland/src/camp.rs`**:
   - `Camp::new` & `Camp::load_from` (`override_levels`): `upgrades.gunsmith_level = levels.gunsmith;`
   - `Camp::levels`: add `gunsmith: self.upgrades.gunsmith_level,`
   - `Camp::upgrade_name`: `UpgradeLine::Gunsmith => self.upgrades_config.gunsmith.get(level as usize).map_or_else(|| "Gunsmith".into(), |d| d.name.clone()),`
   - `Camp::upgrades_to_save`: `gunsmith_level: self.upgrades.gunsmith_level,`
   - `Camp::load_upgrades_from_save`: `self.upgrades.gunsmith_level = data.gunsmith_level;` and `"gunsmith" => UpgradeLine::Gunsmith,`
   - `Camp::upgrades_view`:
     - `UpgradeLine::Gunsmith => "Tier 0 stock (pistol, ammo, repair)".into(),`
     - `UpgradeLine::Gunsmith => { let d = &self.upgrades_config.gunsmith[target as usize]; (d.name.clone(), format!("Unlocks gunsmith tier {} stock", d.gunsmith_tier), &d.cost) }`
     - `lines: [ make_line(UpgradeLine::Walls), make_line(UpgradeLine::Guards), make_line(UpgradeLine::Workshop), make_line(UpgradeLine::Traders), make_line(UpgradeLine::Gunsmith) ]`
7. **`rotland/src/camp_upgrade_view.rs`**:
   - `parse_camp_levels`: accept 4 or 5 parts (`if parts.len() != 4 && parts.len() != 5 { return None; }`, `let gunsmith = parts.get(4).map(|p| p.trim().parse::<u8>().ok()).transpose()?.unwrap_or(0);`) so `--camp-levels=W,G,K,T` tests stay valid and `--camp-levels=W,G,K,T,S` works too.
   - `CampLevels` literals in tests: add `gunsmith: 0` (or `..Default::default()`).
   - `CampUpgradeView::update`: right after `if !self.sync_props(site, levels) { return false; }`, append `self.props.push(world::camp_gunsmith_prop(site));` so `gunsmith_bench` renders and gets its physics collider at all tiers (0..=3) while keeping `sync_props` tier-0 empty for `test_level_0_has_no_props`.
8. **`rotland_ui/src/camp_ui/view.rs`**:
   - `CampUpgradesView.lines`: change `[UpgradeLineView; 4]` -> `[UpgradeLineView; 5]` and add 5th default entry for `UpgradeLine::Gunsmith` (`max_level: 3 // data: camp_upgrades.json gunsmith`).
9. **`rotland_ui/src/camp_ui/upgrades/layout.rs`**:
   - `card_layout`: `let count = game::camp_upgrades::UpgradeLine::ALL.len();` (`5`), `let columns = if w >= 1400.0 { 3 } else { 2 };` (or `5` on ultra-wide), `let rows = count.div_ceil(columns);` so the 5th card fits cleanly without clipping.
10. **`rotland_ui/src/camp_ui/upgrades/colors.rs`** & **`detail.rs`**:
    - `colors::line_display_name`: `UpgradeLine::Gunsmith => "Gunsmith",`
    - `detail::draw_line_icon`: add `UpgradeLine::Gunsmith` arm (simple wrench/rifle/crosshair icon).
11. **`rotland/` town shopkeepers (`shopkeeper_<town_key>`) & zombie safety (for R37-08 / Lead)**:
    - Each `TownSite` in `world_gen.towns()` exposes `town.shopkeeper_spawn(&towns_data, world_gen.height(lot.pos.x, lot.pos.y))` returning `TownShopkeeper { id: "shopkeeper_<town_key>", position, yaw, ... }` (using `TownsData::from_json` on `paths::repo_path("data/towns.json")` or `TownsData::load_embedded()`).
    - Draw each visible shopkeeper within `far_range` (60 m) using `"npc_mechanic"` skeleton + `"idle"` clip (same pattern as `RescueView` / `CampNpcs`), and check 2.5 m / 35° cone interaction in `frame_world.rs` to show `"[E] Talk to gunsmith"` and open `GunsmithScreen` for `ShopKind::Town { key }`.
    - Zombie safety: zombies in `rotland/src/zombies/mod.rs` only ever target `effective_player` (never fixed NPCs or shopkeepers), `gun_shop` is not in `TOWN_RESCUE_BUILDINGS`, and `gun_shop` should not have zombie spawns (Note: `tools/blender/buildings/gun_shop.py` line 159 currently has `k.empty("spawn_1", (0.5, 0.4, F))`; either Lead/R37-05 can omit `spawn_1` from `gun_shop.py` OR `rotland/src/app/frame_sim_zombies.rs` can skip `poi.building_id == "gun_shop"` when building `world.poi_zombie_sites` so no zombie spawns inside the shop!).

### How to verify
- `cargo test` (or `tools/gate.sh --task tasks/R37-round.md` after Lead adds the shared-file lines):
  - `world::tests::golden::terrain_matches_godot` (unchanged)
  - `world::towns::lots::tests::every_town_has_one_gun_shop_near_center_and_shopkeeper_spot`
  - `world::camp_layout::tests::camp_layout_nothing_overlaps` & `camp_layout_gunsmith_bench_placed_and_rotates`
  - `camp_has_gunsmith_npc_and_is_interactable` & `camp_upgrades_gunsmith_tiers_0_to_3`

### Unsure / notes
- `tools/blender/buildings/gun_shop.py` currently includes `k.empty("spawn_1", (0.5, 0.4, F))` at line 159; unless skipped in `frame_sim_zombies.rs` (or removed in `gun_shop.py`), `PoiZombies` will spawn 1 zombie inside `gun_shop` when approached. Filtering `gun_shop` out of `poi_zombie_sites` (or removing `spawn_1` from `gun_shop.py`) keeps the shop completely safe inside.
- Not compiled/tested directly per subagent rule (never run `cargo` or `gate.sh`); only JSON syntax (`python3 json.load`) was verified.
