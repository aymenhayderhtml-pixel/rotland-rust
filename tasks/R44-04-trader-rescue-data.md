# R44-04: Rooftop trader stock, floor rescue spots, rescue data (pure + data)

Runs at the same time as R44-01, 02, 03. Read first: `tasks/R44-common.md` (whole). Style: `tasks/R43-06-floor-zombies-loot.md`.

Owns:
- `data/rooftop_trader.json` (NEW).
- `data/rescue.json` (ONLY the new key `floorSurvivors`: 3).
- `game_camp/src/economy/rooftop_stock.rs` (NEW): `RooftopTraderConfig`.
- `game_camp/src/rescue_floor.rs` (NEW): `FloorRescueSeed`, `floor_rescue_spots`, `is_climb_floor_poi`.
- `game_camp/src/rescue.rs` (ONLY: the new `floor_survivors` field on `RescueConfig`, `#[serde(default)]`).
- `game_camp/src/rescue_place.rs` (ONLY: `place_survivors_in_world` skips POIs where `is_climb_floor_poi(&poi.id)` is true, so the random pass never uses climb-floor POIs).
- `game/tests/all/rooftop_trade.rs` (NEW) and `game/tests/all/rescue_floor.rs` (NEW). The lead adds the `mod` lines.

Not owned: `game_camp/src/economy/trader.rs` (read it; do not change the existing trader), `game_camp/src/economy/mod.rs` and `game_camp/src/lib.rs` (the lead adds `pub mod` lines; put them in Worker notes), anything in `world/`, `rotland/`, `rotland_ui/`.

Read:
- `game_camp/src/economy/config.rs` (`StockEntry`, `EconomyConfigData::from_repo`, `validate(items)`), `game_camp/src/economy/trader.rs` (`TraderStock::from_entries`, `take`, `qty`, `fill_all`).
- `game_camp/src/rescue.rs` (`RescueConfig`, the struct and serde attributes, `from_repo` pattern; `RescueState::new`, `try_rescue_nearest`).
- `game_camp/src/rescue_place.rs` (`RescueSpot` at line 35, the `place_survivors_in_world` candidate loop around lines 327-345, the `RecruitDef` import, `parse_recruit_role`).
- `game_camp/src/economy/config.rs` and `data/economy.json` (`stock` ids: all of them exist; `sellRate`, `buyRate`).
- `data/items.json` (the item ids you use must exist: grep `"id": "<name>"` for each one). `game/tests/all/data_integrity.rs` (how the repo's item registry is loaded in tests; copy it).

## What to write
1. **`data/rooftop_trader.json`** (NEW):
   ```json
   {
     "_why": "R44: the rooftop trader sells city-only gear and basic supplies. Prices come from economy.json (buyRate, sellRate). Claude defaults 2026-11-28.",
     "mapLabel": "Rooftop Camp",
     "stock": [
       { "id": "bandage", "qty": 6 },
       { "id": "medkit", "qty": 2 },
       { "id": "water_bottle", "qty": 6 },
       { "id": "canned_beans", "qty": 6 },
       { "id": "pistol_ammo", "qty": 30 },
       { "id": "shells_12g", "qty": 8 },
       { "id": "rifle_rounds", "qty": 6 }
     ]
   }
   ```
   - These ids are already in `data/economy.json`. You MAY add up to 4 R41 city-only ids (army gear, night vision, antibiotics, surgery kit, adrenaline; check the exact ids in `data/items.json` first), each with qty 1-2. Do not invent ids.
2. **`data/rescue.json`**: add `"floorSurvivors": 3` (with a `"_floorWhy"` note: "R44: survivors on climb-tower stop floors").
3. **`RescueConfig.floor_survivors: usize`** in `game_camp/src/rescue.rs`: `#[serde(default)]` so older configs still load. Keep every other field as it is.
4. **`game_camp/src/economy/rooftop_stock.rs`**:
   - `pub struct RooftopTraderConfig { pub map_label: String, pub stock: Vec<StockEntry> }` (serde; `stock` is `Vec<StockEntry>`, `StockEntry` derives serde already).
   - `from_repo() -> Result<Self, String>` (`paths::repo_path("data/rooftop_trader.json")`), `from_json(&str) -> Result<Self, String>`.
   - `pub fn rooftop_stock(&self) -> TraderStock` = `TraderStock::from_entries(&self.stock)`.
   - `pub fn validate(&self, items: &ItemRegistry) -> Result<(), String>`: every stock id exists in the registry (use the same registry type as `EconomyConfigData::validate`).
5. **`game_camp/src/rescue_floor.rs`**:
   - `pub struct FloorRescueSeed { pub poi_id: String, pub building_id: String, pub position: Vec3, pub yaw: f32 }` (`Clone`, `Debug`).
   - `pub fn is_climb_floor_poi(poi_id: &str) -> bool`: `poi_id.starts_with("metro_floor_")`.
   - `pub fn floor_rescue_spots(seeds: &[FloorRescueSeed], cfg: &RescueConfig, recruits: &[RecruitDef]) -> Vec<RescueSpot>`:
     - Takes the first `cfg.floor_survivors` seeds (all of them if fewer).
     - Ids `rescue_floor_1`, `rescue_floor_2`, ...
     - `name`, `role` (`parse_recruit_role`), `level`, `daily_wage`, `max_health` (`health`) from the recruit template. The template is chosen by walking `recruits` in order and skipping names already used. If every name is used, reuse from the start (as `place_survivors_in_world` does).
     - `position` = seed position, `yaw` = seed yaw, `poi_id` and `building_id` = seed values.
     - If `recruits` is empty, use the same fallback as `place_survivors_in_world` (the `rescue_default` template there).
     - Pure: no RNG. Same input, same output.
6. **`game_camp/src/rescue_place.rs`**: in `place_survivors_in_world`, skip any candidate POI with `is_climb_floor_poi(&poi.id)`. Make no other change to that function.

## Number tags
`floorSurvivors` is data. No new Rust numbers. The `"metro_floor_"` prefix is a string, tagged `// new: DECISIONS 2026-11-28 (climb floor POI id prefix, R43-01)`.

## Tests
`game/tests/all/rooftop_trade.rs`:
- `RooftopTraderConfig::from_repo()` loads; `map_label == "Rooftop Camp"`; `stock.len() >= 7`.
- `validate` against the repo's `ItemRegistry` is `Ok` (every id exists).
- `rooftop_stock()` has each entry's qty; `take(id, 1)` lowers it by 1; `take` of more than in stock returns false.
- A broken JSON string gives `Err`.

`game/tests/all/rescue_floor.rs`:
- `is_climb_floor_poi("metro_floor_3_2") == true`; `is_climb_floor_poi("poi_12") == false`; `is_climb_floor_poi("metro_hospital") == false`.
- `RescueConfig::from_repo()` has `floor_survivors == 3`; a JSON without the key gives 0 (serde default).
- `floor_rescue_spots` with 3 seeds and the repo recruits: 3 spots, ids `rescue_floor_1..3`, each `poi_id` equal to its seed's, names unique.
- With `floor_survivors = 0` (use a config with the field set to 0): no spots.
- With 2 seeds and `floor_survivors = 3`: 2 spots.
- Same input twice: equal output.
- Empty `recruits` uses the fallback: 1 spot with a non-empty name.

## Done when
The files above exist; `data/rescue.json` still parses; the tests pass in the gate (`tools/gate.sh --task tasks/R44-04-trader-rescue-data.md`). Fill `## Worker notes` with:
- Files changed.
- The `pub mod` lines for `game_camp/src/economy/mod.rs` (or lib) and `game_camp/src/lib.rs`, and the `mod` lines for `game/tests/all/main.rs`.
- The item ids you used beyond the economy list, and where you checked them.
- Whether `RecruitDef` / `RescueSpot` fields differ from the common file.

## Worker notes
- **Files changed (mine):** `data/rooftop_trader.json` (new, 11 stock lines: the 7 listed + `antibiotics` 2, `surgery_kit` 1, `adrenaline_shot` 2, `night_vision` 1), `data/rescue.json` (`floorSurvivors: 3`, `_floorWhy`), `game_camp/src/rescue.rs` (`floor_survivors` field with field-level `#[serde(default)]`, so a missing key gives 0; `Default` impl has 3), `game_camp/src/rescue_place.rs` (`place_survivors_in_world` filters POIs with `is_climb_floor_poi`; plus one `use` line), `game_camp/src/rescue_floor.rs` (new), `game_camp/src/economy/rooftop_stock.rs` (new), `game/tests/all/rooftop_trade.rs` and `game/tests/all/rescue_floor.rs` (new, 13 tests).
- **Shared-file lines the lead must add (not committed by me):**
  - `game_camp/src/lib.rs`: `pub mod rescue_floor;` (after `pub mod rescue_place;`).
  - `game_camp/src/economy/mod.rs`: `mod rooftop_stock;` and `pub use rooftop_stock::RooftopTraderConfig;`.
  - `game/src/lib.rs`: add `rescue_floor` to the `pub use game_camp::{ ... rescue, rescue_place, ... }` list (the facade does not re-export it otherwise).
  - `game/tests/all/main.rs`: `mod rescue_floor;` and `mod rooftop_trade;`.
- **Gate:** without the four lines above, the gate FAILS in clippy (`unresolved import crate::rescue_floor`). I added the four lines temporarily, ran the gate (GATE: PASS, 1540 tests), then reverted them, so the commit holds only my files. The lead must add them or the round will not build.
- **Item ids:** economy list ids are all validated by the new test. Extra ids checked in `data/items.json`: `antibiotics`, `surgery_kit`, `adrenaline_shot`, `night_vision` (exist).
- **Deviation:** the fallback recruit template in `floor_rescue_spots` is copied from `place_survivors` (that function is not to be changed). Names are unique while templates last, then they are reused from the start.
- `RecruitDef` and `RescueSpot` fields match the common file. No differences.
- No open questions.

## Gate report
- date: 2026-11-28
- base: main
- files changed (13): data/rescue.json, data/rooftop_trader.json, game_camp/src/economy/mod.rs, game_camp/src/economy/rooftop_stock.rs, game_camp/src/lib.rs, game_camp/src/rescue_floor.rs, game_camp/src/rescue_place.rs, game_camp/src/rescue.rs, game/src/lib.rs, game/tests/all/main.rs, game/tests/all/rescue_floor.rs, game/tests/all/rooftop_trade.rs, tasks/R44-04-trader-rescue-data.md
- tests passed: 1540
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
