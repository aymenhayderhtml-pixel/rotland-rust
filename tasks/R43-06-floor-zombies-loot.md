# R43-06: Zombies and loot on the stop-off floors (wake by height)

Owns:
- `data/poi.json` (ONLY: the new building entries below)
- `data/loot_tables.json` (ONLY: the new tables `metro_tower_office`, `metro_tower_flat`, `metro_tower_camp`, `metro_radio`)
- `world/src/poi.rs` (ONLY: `PoiBuildingDef` gets `#[serde(default)] wake_height_m: f32` from `wakeHeightM`)
- `game_zombie/src/poi_zombies/` (ONLY: the height rule in the wake check + its unit tests)
- `game/tests/all/climb_loot.rs` (NEW)

Lines for the lead: `mod climb_loot;` in `game/tests/all/main.rs`.
Read: tasks/R43-common.md, `game_zombie/src/poi_zombies/manager.rs` (wake check: flat distance <= `approach_distance`; how it reads per-building defs; despawn rule), `world/src/poi.rs`, `data/poi.json` (the R42 `metro_*` entries as the format), `data/loot_tables.json` (`metro_*`), `data/items.json` (only existing ids).

## POI entries (`data/poi.json` `buildings`)
| id | lot | lootTable | zombies | zombieModels | wakeHeightM |
|---|---|---|---|---|---|
| `tw_interior_office_24` / `_32` | [24, 24] / [32, 32] | `metro_tower_office` | [3, 5] / [4, 6] | `zombie_01`, `zombie_02` | 3.0 |
| `tw_interior_flat_24` / `_32` | as above | `metro_tower_flat` | [3, 5] / [4, 6] | `zombie_01`, `zombie_03` | 3.0 |
| `tw_interior_camp_24` / `_32` | as above | `metro_tower_camp` | [4, 6] / [5, 7] | `zombie_02`, `zombie_03`, `zombie_soldier` | 3.0 |
| `tw_stair_flight_z` | [5, 7] | `metro_edge` | [1, 2] | `zombie_01`, `zombie_02` | 3.0 |

- `chancePerChunk: 0.0`, `biome: {}`, the other fields like the R42 `metro_*` entries.
- If the zombie kind list allows it, make `tw_stair_flight_z` spawns mostly crawlers/walkers (no runners in a stairwell by day). Say how.

## The height rule
- When `wake_height_m > 0`, a building also needs `|player.y - poi.y| <= wake_height_m` to wake. `poi.y` is the floor; compare the player's FEET.
- `0.0` keeps today's behaviour exactly, so every old building is unchanged.
- Find out if awake POI zombies despawn by flat distance. If they do, zombies on floor 20 must NOT be despawned just because the player stands in the lobby under them, and killed ones stay dead (the manager already remembers visited buildings). Change only what is needed, and say what.
- `maxActiveZombies` stays 16. One floor at a time is well under it.

## Loot tables (only existing item ids; rolls like the `metro_*` ones)
- **`metro_tower_office`** (rolls 2-4): `battery`, `flashlight`, `painkillers` if it exists, `bandage`, water, canned food, `cloth`, `scrap_metal`, a small chance of `pistol` + its ammo.
- **`metro_tower_flat`** (rolls 2-4): food and water (most), `bandage`, `medkit` (rare), `cloth`, `warm_jacket`, `wool_hat`, `battery`, `revolver` + ammo (rare).
- **`metro_tower_camp`** (rolls 3-5): `shotgun` / `hunting_rifle` + their ammo, `medkit`, `antibiotics`, `battery`, `night_vision` (rare, about 4%), `army_helmet`, food, water.
- **`metro_radio`** (rolls 4-6, the reward at the top):
  - `army_rifle` + `ammo_556` (high chance)
  - `body_armor`, `night_vision`, `surgery_kit`, `adrenaline_shot`, `medkit`
  - each ammo with its gun

## Tests
- **Unit, manager:**
  - With `wake_height_m` 3, a player 30 m below doesn't wake the floor, and one on that floor does.
  - With 0, the flat-only behaviour is unchanged (same result as before for the same inputs).
- **`climb_loot.rs`:**
  - Every new table's item ids are in `items.json`; each weapon has its ammo in the same table.
  - Rolling 2000 crates: `metro_radio` gives a weapon in at least 70%, and `metro_tower_camp` beats `metro_tower_flat` on weapons.
  - The 7 POI entries parse with `wakeHeightM` and `zombieModels`; the old entries parse with 0.

## Done when
Data + code + tests written. Fill `## Worker notes`.

## Worker notes
