# R41-06: City-only items and what they do

Owns:
- `data/items.json` (ONLY adding the 12 new ids below)
- `data/weapons.json` (ONLY the new `army_rifle` entry), `data/gun_feel.json` (ONLY an `army_rifle` entry, if guns have one there), `data/weapon_mounts.json` (ONLY `army_rifle`)
- `data/armor.json` (NEW), `data/boosts.json` (NEW)
- `game_core/src/armor.rs` (NEW) + tests
- `game_player/src/survival/boosts.rs` (NEW) + tests
- `game_player/src/night_vision.rs` (NEW) + tests
- `game_core/src/weapons/` (ONLY a new function `repair_with_parts` in the wear module, + test)
- `game_player/src/vehicles/manager.rs` (ONLY a new method `repair_with_parts` next to `repair`)
- `game/tests/all/city_items.rs` (NEW)

Lines for the lead:
- `pub mod armor;` in `game_core/src/lib.rs`
- `pub mod boosts;` in `game_player/src/survival/mod.rs`
- `pub mod night_vision;` in `game_player/src/lib.rs`
- `mod city_items;` in `game/tests/all/main.rs`

Not owned: `rotland/*` (R41-09 wires the hotbar use, damage, render and HUD), models/icons (R41-07), loot tables (R41-08).
Read: tasks/R41-common.md, `data/items.json` (`warm_jacket`, `wool_hat`, `medkit`, `flashlight`, `ak47`), `data/weapons.json` (`ak47`, `rifle`), `data/weapon_mounts.json`, `data/gun_feel.json`, `rotland/src/temperature_play.rs` (how bag clothing is read: copy that for armour), `game_core/src/health.rs`, `game_core/src/items/` (registry, how `effects` are applied on use), `game_core/src/weapons/` (wear), `game_player/src/vehicles/manager.rs` (`repair`), `game_player/src/downed.rs`.

## The 12 items (`data/items.json`)
| id | name | category | weight | stack | value | notes |
|---|---|---|---|---|---|---|
| `body_armor` | Body Armor | tool | 4.5 | 1 | 220 | `slot: "body"`, description: carry in your bag; cuts zombie bite damage |
| `army_helmet` | Army Helmet | tool | 1.3 | 1 | 120 | `slot: "head"` |
| `army_rifle` | Army Rifle | weapon | 3.4 | 1 | 520 | `weaponId: "army_rifle"` |
| `ammo_556` | 5.56 Rounds | ammo | 0.012 | 60 | 2 | |
| `army_ration` | Army Ration | food | 0.6 | 5 | 30 | `effects: { hunger: 70, thirst: 15, health: 10 }` |
| `night_vision` | Night Vision | tool | 0.9 | 1 | 300 | use = on/off; needs charge |
| `battery` | Battery | material | 0.1 | 10 | 12 | gives charge to night vision |
| `antibiotics` | Antibiotics | medical | 0.1 | 5 | 80 | heals 60 over 60 s (boost) |
| `surgery_kit` | Surgery Kit | medical | 1.2 | 2 | 150 | heals to full; 6 s to use |
| `adrenaline_shot` | Adrenaline | medical | 0.1 | 3 | 110 | while down: get up at once; standing: 20 s free sprint |
| `gun_parts` | Gun Parts | material | 0.5 | 10 | 60 | use while holding a gun: +50% condition |
| `car_parts` | Car Parts | material | 2.0 | 5 | 70 | use near a car: +150 HP, fixes wrecked |

Keep the JSON field names exactly as the existing items use them. Colours: army olive `#4a5a35`, medicine white/red, parts grey. Give each a short plain-English description.

## Weapons
`army_rifle` copies the shape of `ak47`:
- `auto: true`, `ammoItem: "ammo_556"`, magazine 30, damage 32, fireInterval 0.085, less spread than the ak47 when aiming, noiseRadius 70.
- Uses the mount points from R41-07's model; write the `weapon_mounts.json` entry to match the model's empties (agree on names in Worker notes).

## Mechanics (pure, tested)
- **Armour** (`data/armor.json`: `{ "body_armor": { "slot": "body", "reduce": 0.35 }, "army_helmet": { "slot": "head", "reduce": 0.15 } }`):
  - `armor_reduction(inventory, cfg) -> f32` adds the best `reduce` per slot found in the bag, capped at `maxReduce` 0.5 (in the same file).
  - A bite of `d` becomes `d * (1 - reduction)`.
- **Boosts** (`data/boosts.json`):
  - `antibiotics`: `{ "healTotal": 60, "seconds": 60 }`
  - `adrenaline_shot`: `{ "freeSprintSeconds": 20 }`
  - `surgery_kit`: `{ "useSeconds": 6 }`

  `Boosts` holds the running heal-over-time (they stack in time, not in rate: a second dose adds 60 s) and the free-sprint timer.
  - `update(dt) -> f32` returns the health to add this frame.
  - `free_sprint() -> bool`.
  - Save/load with serde.
- **Adrenaline while down**: `pub fn adrenaline_revive(downed: &mut Downed) -> bool` revives at once if downed (use the existing revive path, keep its health rule). Put it in `boosts.rs` or as a small method in `downed.rs`, whichever is cleaner (if `downed.rs`, add only that method).
- **Night vision** (`data/boosts.json` `nightVision`: `{ "secondsPerBattery": 300, "maxCharge": 300, "gain": 6.0, "tint": [0.35, 1.0, 0.45], "noise": 0.06 }`):
  - `NightVision { on, charge }`.
  - `toggle(inventory)` turns it on only if `charge > 0` or a battery can be used (it uses 1 battery -> `charge = secondsPerBattery`).
  - `update(dt, inventory)` drains charge. At 0 it auto-uses the next battery, or turns off.
  - `strength() -> f32` is 0 or 1.
  - Save/load.
- **Gun parts**: `repair_with_parts(wear_state, amount=0.5)` raises condition by 50 points of 100, capped at full, and unjams.
- **Car parts**: `repair_with_parts(car_id, inventory)` removes 1 `car_parts`, gives +150 HP (`data/vehicles.json` new key `partsRepairHp`, add it), and revives a wrecked car.
- Every number has a data tag.

## Tests (`game/tests/all/city_items.rs` + unit tests next to the code)
- All 12 ids load from the real `data/items.json`.
- Every `weaponId` and `ammoItem` exists. `army_rifle` has `auto`.
- Armour:
  - body armour alone: 0.35
  - armour + helmet: 0.5
  - two body armours: still 0.35
  - empty bag: 0
- Antibiotics give 60 HP over 60 s (sum of the updates within 0.5). Two doses run 120 s.
- Adrenaline revives a downed player.
- Night vision:
  - toggle with no battery and no charge stays off
  - with one battery it runs 300 s, then turns off
  - with 2 batteries it runs 600 s
- Gun parts cap at 100. Car parts revive a wrecked car and use exactly 1 part.

## Done when
Code + tests written (the lead builds and tests). Fill `## Worker notes

### Files changed / added
- `data/items.json`: added the 12 city-only items (`body_armor`, `army_helmet`, `army_rifle`, `ammo_556`, `army_ration`, `night_vision`, `battery`, `antibiotics`, `surgery_kit`, `adrenaline_shot`, `gun_parts`, `car_parts`).
- `data/weapons.json`: added `army_rifle` (`auto: true`, `ammoItem: "ammo_556"`, `magazineSize: 30`, `damage: 32`, `fireInterval: 0.085`, `spreadAimDeg: 0.35`, `noiseRadius: 70`).
- `data/gun_feel.json`: added `army_rifle` (`kickDeg: 1.1`, `climb: true`, `maxClimbDeg: 7.5`).
- `data/weapon_mounts.json`: added `army_rifle` mount points (`optic`, `muzzle`, `magazine`, `grip`, `stock`) — R41-07's Blender model should use these empty names (`optic`, `muzzle`, `magazine`, `grip`, `stock`) or update the coordinates once baked.
- `data/vehicles.json`: added `"partsRepairHp": 150`.
- `data/armor.json` (NEW): `{ "maxReduce": 0.5, "items": { "body_armor": { "slot": "body", "reduce": 0.35 }, "army_helmet": { "slot": "head", "reduce": 0.15 } } }`.
- `data/boosts.json` (NEW): `antibiotics`, `adrenaline_shot`, `surgery_kit`, and `nightVision` configs.
- `game_core/src/armor.rs` (NEW): `ArmorConfig`, `ArmorItemDef`, `armor_reduction`, `apply_armor_damage` + unit test.
- `game_player/src/survival/boosts.rs` (NEW): `BoostsConfig`, `Boosts` (`start_antibiotics`, `start_adrenaline`, `update`, `free_sprint`, serde save/load), `adrenaline_revive(&mut Downed) -> bool` + unit tests.
- `game_player/src/night_vision.rs` (NEW): `NightVision` (`toggle`, `update`, `strength`, serde save/load), `BATTERY_ITEM` + unit test.
- `game_core/src/weapons/wear.rs`: added `GUN_PARTS_REPAIR_AMOUNT` (`0.5`) and `repair_with_parts(&mut ItemStack, f32) -> bool` + unit test.
- `game_player/src/vehicles/manager.rs`: added `Cars::repair_with_parts(&mut self, car_id: &str, inventory: &mut Inventory) -> bool` next to `repair`.
- `game/tests/all/city_items.rs` (NEW): integration tests covering all 12 items, `army_rifle`, armor reduction, antibiotics stacking, adrenaline revive, night vision battery drain, gun parts, and car parts.

### Shared-file lines for the lead
- `game_core/src/lib.rs`:
  - `pub mod armor;`
  - `pub use armor::*;` (or `pub use armor::{armor_reduction, apply_armor_damage, ArmorConfig, ArmorItemDef};`)
- `game_player/src/survival/mod.rs`:
  - `pub mod boosts;`
  - `pub use boosts::{adrenaline_revive, AdrenalineConfig, AntibioticsConfig, Boosts, BoostsConfig, NightVisionConfig, SurgeryKitConfig};`
- `game_player/src/lib.rs`:
  - `pub mod night_vision;`
  - `pub use night_vision::{NightVision, BATTERY_ITEM};`
- `game/src/lib.rs`:
  - `armor,` in `pub use game_core::{ ... };` and `pub use game_core::armor::*;`
  - `night_vision,` in `pub use game_player::{ ... };` and `pub use game_player::night_vision::{NightVision, BATTERY_ITEM};` (note: `pub use game_player::survival::*;` already re-exports `boosts` symbols)
- `game/tests/all/main.rs`:
  - `mod city_items;`

### Note for the lead / data integrity
- `game/tests/all/data_integrity.rs` (`weapon_wear_json_covers_all_weapons`) checks that every gun in `data/weapons.json` has a `wearPerShot > 0` entry in `data/weapon_wear.json` (which this task does not own). Please add `"army_rifle": { "wearPerShot": 0.00167 }` to `data/weapon_wear.json` (or similar).
- `data_integrity.rs` also checks that every gun's `ammoItem` (`ammo_556`) is obtainable (R41-08 adds it to `metro_army`/`metro_core` loot tables).

### Wiring items for R41-09
1. **Hotbar / item use (`rotland/src/play.rs` or item-use handler):**
   - `antibiotics`: consume 1, call `boosts.start_antibiotics(&boosts_cfg.antibiotics)`.
   - `adrenaline_shot`: if downed (`downed.is_down()`), consume 1 and call `adrenaline_revive(&mut downed)` (and restore player health via the existing revive health rule); if standing, consume 1 and call `boosts.start_adrenaline(&boosts_cfg.adrenaline_shot)`.
   - `surgery_kit`: timed use (`boosts_cfg.surgery_kit.use_seconds` = 6 s), then consume 1 and heal player to full health.
   - `night_vision`: call `night_vision.toggle(&mut inventory, &boosts_cfg.night_vision)`.
   - `gun_parts`: if holding a gun with `condition() < 1.0` (or jammed), call `repair_with_parts(held_stack, GUN_PARTS_REPAIR_AMOUNT)`, clear runtime gun jam flag, and consume 1 `gun_parts`.
   - `car_parts`: when near a damaged/wrecked car, call `cars.repair_with_parts(&car_id, &mut inventory)`.
   - `army_ration`: already uses standard `effects` (`hunger: 70, thirst: 15, health: 10`).
2. **Armour in zombie bites (`rotland/src/combat_damage.rs`):**
   - Compute `let reduction = armor_reduction(&inventory, &armor_cfg);` and scale bite damage with `apply_armor_damage(damage, reduction)`.
3. **Boosts & Night Vision per-frame update + HUD + save/load:**
   - Each frame: `let heal = boosts.update(dt);` -> add `heal` to player health; while `boosts.free_sprint()` is true, skip stamina drain on sprint.
   - Each frame: `night_vision.update(dt, &mut inventory, &boosts_cfg.night_vision);` and pass `night_vision.strength()`, `boosts_cfg.night_vision.gain`, `tint`, `noise` to the renderer/shader, plus battery/charge to the HUD.
   - Show active `boosts` (antibiotics / adrenaline timers) on the HUD and persist `Boosts` and `NightVision` in save/load.`: the lib/mod lines and the list of things R41-09 must wire (hotbar use per item, armour in bites, boosts update + HUD, night vision render input).

## Worker notes

### Files changed / added
- `data/items.json`: added the 12 city-only items (`body_armor`, `army_helmet`, `army_rifle`, `ammo_556`, `army_ration`, `night_vision`, `battery`, `antibiotics`, `surgery_kit`, `adrenaline_shot`, `gun_parts`, `car_parts`).
- `data/weapons.json`: added `army_rifle` (`auto: true`, `ammoItem: "ammo_556"`, `magazineSize: 30`, `damage: 32`, `fireInterval: 0.085`, `spreadAimDeg: 0.35`, `noiseRadius: 70`).
- `data/gun_feel.json`: added `army_rifle` (`kickDeg: 1.1`, `climb: true`, `maxClimbDeg: 7.5`).
- `data/weapon_mounts.json`: added `army_rifle` mount points (`optic`, `muzzle`, `magazine`, `grip`, `stock`) — R41-07's Blender model should use these empty names (`optic`, `muzzle`, `magazine`, `grip`, `stock`) or update the coordinates once baked.
- `data/vehicles.json`: added `"partsRepairHp": 150`.
- `data/armor.json` (NEW): `{ "maxReduce": 0.5, "items": { "body_armor": { "slot": "body", "reduce": 0.35 }, "army_helmet": { "slot": "head", "reduce": 0.15 } } }`.
- `data/boosts.json` (NEW): `antibiotics`, `adrenaline_shot`, `surgery_kit`, and `nightVision` configs.
- `game_core/src/armor.rs` (NEW): `ArmorConfig`, `ArmorItemDef`, `armor_reduction`, `apply_armor_damage` + unit test.
- `game_player/src/survival/boosts.rs` (NEW): `BoostsConfig`, `Boosts` (`start_antibiotics`, `start_adrenaline`, `update`, `free_sprint`, serde save/load), `adrenaline_revive(&mut Downed) -> bool` + unit tests.
- `game_player/src/night_vision.rs` (NEW): `NightVision` (`toggle`, `update`, `strength`, serde save/load), `BATTERY_ITEM` + unit test.
- `game_core/src/weapons/wear.rs`: added `GUN_PARTS_REPAIR_AMOUNT` (`0.5`) and `repair_with_parts(&mut ItemStack, f32) -> bool` + unit test.
- `game_player/src/vehicles/manager.rs`: added `Cars::repair_with_parts(&mut self, car_id: &str, inventory: &mut Inventory) -> bool` next to `repair`.
- `game/tests/all/city_items.rs` (NEW): integration tests covering all 12 items, `army_rifle`, armor reduction, antibiotics stacking, adrenaline revive, night vision battery drain, gun parts, and car parts.

### Shared-file lines for the lead
- `game_core/src/lib.rs`:
  - `pub mod armor;`
  - `pub use armor::*;` (or `pub use armor::{armor_reduction, apply_armor_damage, ArmorConfig, ArmorItemDef};`)
- `game_player/src/survival/mod.rs`:
  - `pub mod boosts;`
  - `pub use boosts::{adrenaline_revive, AdrenalineConfig, AntibioticsConfig, Boosts, BoostsConfig, NightVisionConfig, SurgeryKitConfig};`
- `game_player/src/lib.rs`:
  - `pub mod night_vision;`
  - `pub use night_vision::{NightVision, BATTERY_ITEM};`
- `game/src/lib.rs`:
  - `armor,` in `pub use game_core::{ ... };` and `pub use game_core::armor::*;`
  - `night_vision,` in `pub use game_player::{ ... };` and `pub use game_player::night_vision::{NightVision, BATTERY_ITEM};` (note: `pub use game_player::survival::*;` already re-exports `boosts` symbols)
- `game/tests/all/main.rs`:
  - `mod city_items;`

### Note for the lead / data integrity
- `game/tests/all/data_integrity.rs` (`weapon_wear_json_covers_all_weapons`) checks that every gun in `data/weapons.json` has a `wearPerShot > 0` entry in `data/weapon_wear.json` (which this task does not own). Please add `"army_rifle": { "wearPerShot": 0.00167 }` to `data/weapon_wear.json` (or similar).
- `data_integrity.rs` also checks that every gun's `ammoItem` (`ammo_556`) is obtainable (R41-08 adds it to `metro_army`/`metro_core` loot tables).

### Wiring items for R41-09
1. **Hotbar / item use (`rotland/src/play.rs` or item-use handler):**
   - `antibiotics`: consume 1, call `boosts.start_antibiotics(&boosts_cfg.antibiotics)`.
   - `adrenaline_shot`: if downed (`downed.is_down()`), consume 1 and call `adrenaline_revive(&mut downed)` (and restore player health via the existing revive health rule); if standing, consume 1 and call `boosts.start_adrenaline(&boosts_cfg.adrenaline_shot)`.
   - `surgery_kit`: timed use (`boosts_cfg.surgery_kit.use_seconds` = 6 s), then consume 1 and heal player to full health.
   - `night_vision`: call `night_vision.toggle(&mut inventory, &boosts_cfg.night_vision)`.
   - `gun_parts`: if holding a gun with `condition() < 1.0` (or jammed), call `repair_with_parts(held_stack, GUN_PARTS_REPAIR_AMOUNT)`, clear runtime gun jam flag, and consume 1 `gun_parts`.
   - `car_parts`: when near a damaged/wrecked car, call `cars.repair_with_parts(&car_id, &mut inventory)`.
   - `army_ration`: already uses standard `effects` (`hunger: 70, thirst: 15, health: 10`).
2. **Armour in zombie bites (`rotland/src/combat_damage.rs`):**
   - Compute `let reduction = armor_reduction(&inventory, &armor_cfg);` and scale bite damage with `apply_armor_damage(damage, reduction)`.
3. **Boosts & Night Vision per-frame update + HUD + save/load:**
   - Each frame: `let heal = boosts.update(dt);` -> add `heal` to player health; while `boosts.free_sprint()` is true, skip stamina drain on sprint.
   - Each frame: `night_vision.update(dt, &mut inventory, &boosts_cfg.night_vision);` and pass `night_vision.strength()`, `boosts_cfg.night_vision.gain`, `tint`, `noise` to the renderer/shader, plus battery/charge to the HUD.
   - Show active `boosts` (antibiotics / adrenaline timers) on the HUD and persist `Boosts` and `NightVision` in save/load.
