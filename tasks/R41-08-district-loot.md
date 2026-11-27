# R41-08: Loot by district

Owns:
- `world/src/loot.rs` (ONLY the two new optional fields on `LootSpot`, `#[serde(default)]`, and keeping `new`/`with_tier` setting them to `None`)
- `world/src/metro/loot.rs` (NEW, pure), `world/src/metro/loot_tests.rs` (NEW)
- `world/src/metro/config.rs` (ONLY the new optional `loot: MetroLootConfig` field, `#[serde(default)]`)
- `data/metro.json` (ONLY the new top-level key `loot`)
- `data/loot_tables.json` (ONLY the 7 new `metro_*` tables)
- `game_core/src/loot/table.rs` (ONLY a `get(name)` lookup, if one does not exist)
- `rotland/src/crates.rs` (ONLY: use `spot.table` when set; draw `spot.model` when set)

Lines for the lead: `pub mod loot as metro_loot;` (or `pub mod loot;`, your choice; state it) + `#[cfg(test)] mod loot_tests;` in `world/src/metro/mod.rs`, and the exact line that adds metro crates in `world/src/generator_contents.rs` `chunk_crates_from`.
Not owned: `data/items.json` (R41-06), models (R41-07), the other metro files.
Read: tasks/R41-common.md, `world/src/loot.rs`, `world/src/generator_contents.rs` (`chunk_crates`, `chunk_crates_from`), `rotland/src/crates.rs` (whole), `game_core/src/loot/` (table, roller), `data/loot_tables.json` (`office`, `police`, `hospital`, `pharmacy`, `town_store`), `world/src/metro/traces.rs` + `traces_camps.rs` (lobby dressing positions, rooftop camps, sandbags), `world/src/metro/jam*.rs` (cars).

## Data (`data/metro.json` -> `loot`)
```json
"loot": {
  "_why": "R41: loot by district. Chances per tower lobby / camp / jam car.",
  "lobbyChance": { "edge": 0.35, "midtown": 0.45, "core": 0.55 },
  "medicalChance": { "edge": 0.05, "midtown": 0.08, "core": 0.12 },
  "armyCaches": { "midtown": 2, "core": 5 },
  "campStash": 1.0,
  "carTrunkChance": 0.08
}
```

## Placement (`metro_loot_in_chunk`)
- **Lobbies:** each tower lobby in the chunk may get ONE container (by `lobbyChance` for its district).
  - It is a `metro_locker` against the back wall, at floor height.
  - Its table is `metro_edge` / `metro_midtown` / `metro_core`.
  - Some get a `med_cabinet` instead (`metro_medical` table, `medicalChance`).
  - Never inside a dressing prop or a wall: use the lobby box from `tower_parts`.
- **Rooftop camps:** each camp gets a `camp_stash` (`metro_camp`) beside the tent, at the deck height (re-use `roof_top_y`).
- **Army caches:** a fixed number per district (`armyCaches`), on sidewalks next to `sandbags_line` or burnt army trucks if any are in the block, else at a street corner.
  - Model `army_crate`, table `metro_army`.
  - Deterministic city-wide: pick the blocks with a seeded rng over all blocks of that district, then each chunk returns the ones inside it.
- **Jam cars:** a `carTrunkChance` of highway jam cars inside the city get a loot spot at the trunk. Table `metro_car`, no model: `model: None` draws nothing extra. Add a field note: `crate_loot` is NOT drawn when `model` is `Some("")`. Use `Some("")` for "no model".
- `crate_id`s are unique and stable: `metro_<kind>_<x>_<z>` from rounded positions.
- `tier` is 0 (the metro uses its own tables).
- No crates in reserved blocks or inside tower footprints, except lobbies.

## Tables (`data/loot_tables.json`, same format as the others)
Weights are relative; city-only items are rare everywhere except where marked.
- `metro_edge` (rolls 1-3): everyday loot like `office` + `town_store` (food, water, bandages, cloth, pistol ammo, tools). Rare: `battery`, `army_ration`.
- `metro_midtown` (rolls 2-3): better. Adds `ammo_556`, `battery`, `antibiotics` (rare), `gun_parts` (rare), `car_parts` (rare), `night_vision` (very rare).
- `metro_core` (rolls 2-4): best. More of the midtown rare ones, plus `body_armor` (rare), `army_helmet` (rare), `surgery_kit` (rare), `adrenaline_shot` (rare).
- `metro_army` (rolls 3-5): `ammo_556`, `army_ration`, `army_helmet`, `body_armor`, `army_rifle` (about 12%), `battery`, `night_vision` (about 8%), `grenade`.
- `metro_medical` (rolls 2-3): `bandage`, `medkit`, `antibiotics`, `adrenaline_shot`, `surgery_kit` (rare).
- `metro_camp` (rolls 2-4): food, water, `battery`, ammo, cloth, scrap metal, a `gun_parts` chance.
- `metro_car` (rolls 1-2): `car_parts`, fuel (use the existing fuel item id), scrap metal, a bag of cloth.

Use only item ids that exist in `data/items.json` after R41-06 (the 12 new ids are fixed in the common file).

## Tests (`loot_tests.rs` + one test for `crates.rs` logic if it is pure)
- Same seed gives the same spots.
- Every spot's `table` exists in the real `data/loot_tables.json`, and every item in the 7 tables exists in `data/items.json`.
- The army caches total exactly `armyCaches` per district over the whole city (sum over all chunks).
- Rolling 2000 `metro_core` crates gives more city-only items per crate than 2000 `metro_edge` crates (use the real roller).
- No spot is placed outside New Halden.
- An old saved `LootSpot` without the new fields still deserializes (serde default).
- `world/tests/golden.rs` is untouched and passes.

## Done when
Code + tests written (the lead builds and tests). Fill `## Worker notes`.

## Worker notes
