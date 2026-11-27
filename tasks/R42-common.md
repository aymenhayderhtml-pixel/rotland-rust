# R42 common: New Halden big places + the failed army line

Setup, subagent rules and the lead loop: `tasks/LEAD.md` (worktree `../rotland-r42`, branch `r42`, shared `CARGO_TARGET_DIR`).
Plan: `docs/PLAN_1.1.md` (R42 row). Owner choices: `docs/DECISIONS.md` 2026-11-26/27 rows. Read the R41 Lead notes + "Claude review fixes" in `tasks/R41-round.md` once.

## What is true today (checked by Claude 2026-11-27)
- **The plots already exist.** `data/metro.json` `reserved` keeps empty block groups:
  - `hospital` (ix 18, iz 13, 2x2)
  - `police` (12, 17, 2x2)
  - `fire_station` (20, 12, 2x2)
  - `army_line` (14, 8, 4x1)

  Towers, overgrowth, loot, herds and poles all skip reserved blocks (`block.reserved.is_some()`), so these plots are bare ground today.
- **Blocks:**
  - Each block is 64 m. Streets run on every block edge, also INSIDE a 2x2 group, so a 2x2 group is 4 lots with a street cross between them.
  - Street 12 m wide, avenue 20 m (every 4th grid line is an avenue), sidewalk 2.5 m.
  - A building lot inside one block is safely **40 x 40 m**.
- **Buildings with zombies and loot = POIs.** A chunk `PoiInstance` (`world/src/poi.rs`) has a building model id, position, yaw and loot table. `data/poi.json` `buildings` lists each id with `lot`, `lootTable` and `zombies: [min, max]`. Zombies wake when you come close and spawn at the model's `spawn_*` empties (`rotland/src/app/frame_sim_zombies.rs`, `game_zombie/src/poi_zombies/`). Town lots do exactly this (`world/src/generator_contents.rs` `append_town_contents`).
- **Loot boxes = `LootSpot`s**, with the R41 `table` and `model` overrides (`world/src/loot.rs`, `rotland/src/crates.rs`). The containers `army_crate`, `metro_locker`, `med_cabinet` and `camp_stash` exist. `data/loot_tables.json` has `metro_*` tables, plus `police` and `hospital` from the towns.
- **Blender buildings** use the kit in `tools/blender/buildings/kit.py` + `tools/blender/common.py`:
  - `add_collider(label, bm)` exports `<model>_<label>-convcolonly`, so the baker makes a convex collider. Walk-in rooms need one collider per wall piece, with door gaps left open.
  - Empties named `spawn_*` are zombie spawn points.
  - Look at `tools/blender/buildings/police.py` and `hospital.py` (the small town versions) and the R39 lobbies (`tools/blender/towers/shell.py`, `lobby_props*.py`) for style.
- Existing pieces to reuse:
  - `sandbags_line`, `army_truck_jam`
  - `body_sheet`, `body_sheet_b`, `body_bag` (if missing, R42-03 makes it)
  - `car_burnt_pile`, `rubble_field_m`, the R41 crates
  - zombie looks `zombie_01..03`
- Zombie kinds and looks: `data/zombies.json` (each kind has `models: [...]`). Spawning: `rotland/src/zombies/` `spawn_kind(kind, pos)`.

## Owner decisions this round follows (PLAN_1.1 R42 + Claude defaults; numbers live in data)
- Three big places, each on its 2x2 plot. Each has ONE main building you can walk into on the ground floor (upper floors are closed facades until R43 climbing), and the other three lots are yards. All of it is overgrown and wrecked like the rest of the city.
  - **Hospital:**
    - The main building has an ER entrance with an ambulance canopy.
    - Inside: reception, waiting room, a ward with beds and curtains, a pharmacy room behind a cage.
    - Yards: an ambulance bay with crashed ambulances; a triage area with white tents, a **big pile of body bags** and cots; a parking lot of wrecked cars.
  - **Police station:**
    - Inside: front desk, open office with desks, holding cells, a locker room, and an armoury cage.
    - Yards: police cars (some burnt), a barricade of concrete blocks and riot shields; an impound lot.
  - **Fire station:**
    - Big garage bays with a fire truck and a ladder truck (one bay empty, door torn).
    - Inside: a hose/gear room, lockers and a kitchen.
    - Yards: a training yard and a burnt-out truck.
- **The failed army line** runs along the avenue beside its 4x1 strip:
  - a wall of sandbags and concrete barriers with razor-wire coils across the avenue
  - a **tank** (stopped, hatch open)
  - **burnt army trucks**
  - a **crashed helicopter** (rotor broken, tail snapped)
  - **evacuation tents** with cots and a field kitchen
  - **loudspeaker poles**
  - a checkpoint booth, scattered gear and body bags

  It reads as "the army tried to hold the street here and lost".
- **Top loot, heavy danger.**
  - Each place has many containers with rich tables: hospital = medicine, police = guns and armour, fire station = tools/axes/medkits, army line = army gear.
  - More zombies around them, including new **zombie looks**: soldiers at the army line, police at the station, medics (doctors/nurses) at the hospital, firefighters at the fire station. A brute guards the army line.
- Detail first; flat palette style; no real brands, names or logos (generic words only: "EMERGENCY", "POLICE", "FIRE DEPT", "EVACUATION", "ARMY"). No sounds. Show-level gore only.
- **FPS:** `metro` scene only, average at least 20; the 1% low is for information.

## R42 shared names (every task uses EXACTLY these)
- **Coordinates in data:**
  - All positions in place data files are building-local GAME metres: `x` = Blender x, `y` = Blender z (up), `z` = -Blender y.
  - The building's front (Blender +Y) is game -Z local.
  - Placement turns local -Z toward the street the place faces, with the place's yaw (Godot yaw: forward = `(-sin yaw, -cos yaw)`).
- **Place data files** (one per place, each owned by its model task), `data/places/<kind>.json`, `<kind>` in `hospital`, `police`, `fire_station`:
```json
{
  "_why": "R42: <kind> layout. Local game metres; front = local -Z.",
  "model": "metro_hospital",
  "footprint": [40, 32],
  "floors": 5,
  "crates": [ { "x": 0, "y": 0, "z": 0, "yaw": 0, "model": "med_cabinet", "table": "metro_hospital" } ],
  "props":  [ { "x": 0, "y": 0, "z": 0, "yaw": 0, "model": "gurney", "scale": 1.0 } ],
  "yards": {
    "a": [ { "x": 0, "y": 0, "z": 0, "yaw": 0, "model": "ambulance_wreck", "scale": 1.0 } ],
    "b": [], "c": []
  },
  "yardCrates": { "a": [], "b": [], "c": [] }
}
```
  - `crates`/`props` are inside or right around the main building (main-lot local).
  - `yards.a/b/c` are the other three lots, each in ITS OWN lot-local frame (lot centre = 0,0, the same front rule). Placement decides which lot is a/b/c (see R42-01).
  - Every model named there must exist.
- **Army line data** `data/places/army_line.json` (owned by R42-02, the placement task) uses the same `props`/`crates` entries but along the strip: `x` from -128 to +128 along the avenue (the strip centre = 0), `z` across (0 = the avenue centre line, + = toward the strip).
- **Main building models** (one each, Blender, category `buildings`):
  - `metro_hospital` (+ `_lod1`)
  - `metro_police` (+ `_lod1`)
  - `metro_fire_station` (+ `_lod1`)

  Each has walk-in ground floor colliders, `spawn_1..spawn_N` empties (N 8-14, inside and at doors), and closed upper floors.
- **Building ids in `data/poi.json` `buildings`** (R42-08): the same three ids (`metro_hospital`, `metro_police`, `metro_fire_station`) plus `metro_army_line` (spawn points come from data, see R42-07). `chancePerChunk: 0.0` (placed only by the city).
- **Prop models** (Blender, category `props` unless said otherwise):
  - Hospital (R42-03): `gurney`, `hospital_bed`, `ward_curtain`, `wheelchair`, `iv_stand`, `reception_desk_h`, `pharmacy_cage`, `waiting_chairs`, `body_bag`, `body_bag_pile`, `triage_tent`, `cot`, `ambulance_wreck` (category `vehicles`).
  - Police (R42-04): `police_car_wreck` (vehicles), `police_car_burnt` (vehicles), `police_desk`, `holding_cell`, `locker_row`, `armory_cage`, `riot_shield_pile`, `jersey_barrier`, `police_barricade`.
  - Fire station (R42-05): `fire_truck` (vehicles), `ladder_truck` (vehicles), `fire_truck_burnt` (vehicles), `hose_rack`, `gear_rack`, `bay_door_torn`, `kitchen_counter`.
  - Army line (R42-06): `army_tank` (vehicles), `army_truck_burnt` (vehicles), `heli_crashed` (vehicles), `heli_rotor_blade`, `sandbag_wall` (straight 6 m), `sandbag_corner`, `razor_wire` (8 m coil run), `concrete_barrier`, `evac_tent`, `field_kitchen`, `loudspeaker_pole`, `checkpoint_booth`, `army_gear_scatter`, `flood_light`.
- **Zombie looks** (R42-07, category `characters`, same rig and size as `zombie_01`): `zombie_soldier`, `zombie_police`, `zombie_medic`, `zombie_firefighter`.
  - `data/poi.json` building entries get an optional `zombieModels: [..]`.
  - Spawning uses `Zombies::spawn_kind_look(kind, pos, model)`.
- **Placement** (R42-01, pure, `world/src/metro/places.rs`):
  - `pub struct PlaceSite { kind: Reserved, main_lot: (i32,i32), yard_lots: [(i32,i32); 3], pos: Vec2 /*main lot centre, world*/, yaw: f32 }`
  - `pub fn place_sites(layout: &MetroLayout) -> Vec<PlaceSite>`
  - `pub fn places_in_chunk(metro: &MetroWorld, chunk: (i32,i32), height: &dyn Fn(f32,f32)->f32) -> PlaceChunk { pois: Vec<PoiInstance>, props: Vec<PropInstance>, crates: Vec<LootSpot> }`
- **Army line placement** (R42-02, pure, `world/src/metro/army_line.rs`): `pub fn army_line_in_chunk(...) -> PlaceChunk` (same struct).
- **Danger** (R42-08): `data/spawning.json` `metro.placeDanger`: `{ "radius": 90, "densityMult": 1.6, "specialBonus": 0.08 }`. `MetroSpawnProfile` gains the place bonus when the player is within `radius` of a place.
- **Flags** (R42-09):
  - `--metro=hospital|hospital_in`
  - `--metro=police|police_in`
  - `--metro=fire|fire_in`
  - `--metro=armyline|armyline_top`

  `_in` = standing inside the main hall looking across it. `armyline_top` = a high view over the whole line.

## Tests that must stay green
`world/tests/golden.rs` passes UNEDITED. R38-R41 metro tests still pass. Spawning outside New Halden is unchanged. Old saves load.
