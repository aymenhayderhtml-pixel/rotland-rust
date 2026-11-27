# R42-08: Top loot and heavy danger at the big places

Owns:
- `data/loot_tables.json` (ONLY the new tables `metro_hospital`, `metro_police`, `metro_fire`)
- `data/poi.json` (ONLY the 4 new building entries)
- `data/spawning.json` (ONLY the new key `metro.placeDanger`)
- `game_zombie/src/metro_spawn.rs` (+ its tests file: the place bonus)
- `game/tests/all/places_loot.rs` (NEW)

Lines for the lead: `mod places_loot;` in `game/tests/all/main.rs`.
Not owned: `rotland/*` (R42-09 passes "near a place" into the profile), models, places code.
Read: tasks/R42-common.md, `data/loot_tables.json` (`metro_*`, `police`, `hospital`), `data/items.json` (the R41 city items + every weapon/ammo id), `data/poi.json` (format), `game_zombie/src/metro_spawn.rs` + tests, `data/spawning.json` `metro`.

## Loot tables (same format as the others; only existing item ids)
- `metro_hospital` (rolls 3-5): `bandage`, `medkit`, `antibiotics` (common here), `surgery_kit`, `adrenaline_shot`, `painkillers` if it exists, water, a small chance of `battery`.
- `metro_police` (rolls 3-5): `pistol`, `revolver`, `shotgun`, `smg`, ammo for each (correct `ammoItem`s), `body_armor`, `army_helmet` (riot helmet stand-in), `flashlight`, `battery`, rare `army_rifle` (about 5%).
- `metro_fire` (rolls 3-4): `fire_axe`, `axe`, `crowbar`, `medkit`, `bandage`, `flashlight`, `battery`, `cloth`, `scrap_metal`, `car_parts`.

The army line uses `metro_army` + `metro_medical` from R41.

## POI buildings (`data/poi.json` `buildings`)
| id | lot | lootTable | zombies | zombieModels |
|---|---|---|---|---|
| `metro_hospital` | [40, 32] | `metro_hospital` | [10, 14] | `zombie_medic`, `zombie_03`, `zombie_01` |
| `metro_police` | [38, 30] | `metro_police` | [8, 12] | `zombie_police`, `zombie_police`, `zombie_02` |
| `metro_fire_station` | [36, 28] | `metro_fire` | [6, 10] | `zombie_firefighter`, `zombie_01` |
| `metro_army_line` | [60, 30] | `metro_army` | [12, 16] | `zombie_soldier` x3, `zombie_02` |

- `chancePerChunk: 0.0`, `biome: {}`, the other fields as `police`.
- If the POI manager caps awake zombies (`maxActiveZombies`), say so in Worker notes and suggest a value; don't change the global cap.

## Danger (`metro.placeDanger`: `{ "radius": 90, "densityMult": 1.6, "specialBonus": 0.08, "bruteAtArmyLine": true }`)
- `metro_profile` gets a new argument `near_place: bool` (or a wrapper `metro_profile_near_place`; keep the R41 call working).
- When near: `density *= densityMult` and `max_active *= densityMult` (still capped by `hardCap` later), and `special_chance_bonus += specialBonus`.
- `bruteAtArmyLine` is read by R42-09 (one `brute` spawns with the army line POI); just load it here.
- Tests:
  - The near profile is denser than the far one for every zone.
  - Not near = exactly the R41 numbers.
  - Loading the real file works.

## Tests (`game/tests/all/places_loot.rs`)
- The three tables exist and every item id in them is in `data/items.json`. Weapons have their ammo in the same table.
- Rolling 2000 crates: `metro_hospital` gives more medical items per crate than `metro_edge`, and `metro_police` gives a weapon or ammo in at least 60% of crates.
- The 4 POI entries parse, with `zombieModels` (if R42-07's field is not merged yet, parse the JSON with serde_json directly in the test).

## Done when
Data + code + tests written (the lead builds and tests). Fill `## Worker notes`.

## Worker notes
