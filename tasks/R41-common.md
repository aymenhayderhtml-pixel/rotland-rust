# R41 common: New Halden, zombies, loot, silent life

Setup, subagent rules and the lead loop: `tasks/LEAD.md` (worktree `../rotland-r41`, branch `r41`, shared `CARGO_TARGET_DIR`).
Plan: `docs/PLAN_1.1.md` (R41 row). Owner choices: `docs/DECISIONS.md` 2026-11-26 and 2026-11-27 rows. What exists: `world/src/metro/` (layout with `MetroDistrict` Edge/Midtown/Core, towers, tower_parts, overgrowth, damage, traces), `rotland/src/metro_render/`, `game_zombie/src/` (spawner, horde, director), `game_core/src/loot/`, `rotland/src/crates.rs`, `rotland/src/wildlife.rs`, `data/items.json`, `data/loot_tables.json`, `data/spawning.json`. Read the R40 Lead notes in `tasks/R40-round.md` once.

## What is true today (checked by Claude 2026-11-27)
- **The new city has no zombie or loot rules of its own.** `rotland/src/play.rs` `spawn_tick` only knows the OLD city Ashford (`world_gen.city()`), towns and biomes. In New Halden the spawner uses the swamp/snow biome density, and crates use the biome loot table (`crates.rs`: `loot_tables.for_biome(spot.biome)`).
- Zombie kinds: `walker`, `runner` (+ specials). Runner share comes from `data/spawning.json` `types.day/night` and `city.coreNightTypes`.
- Hordes: `game_zombie/src/horde/` (`HordeSet::create_horde(leader, members, leader_pos)`, `Horde::set_target_pos`, boids + formation).
- Items: `data/items.json` is a map id -> item. Clothing is carried in the bag and works from there (`slot: "body"|"head"`, `insulation`; read in `rotland/src/temperature_play.rs`). Item `effects` today: only `health`, `hunger`, `thirst`.
- Weapons: `ak47` (auto, `ammo_762`) and `smg` exist. Car repair: `game_player/src/vehicles/manager.rs` `repair` uses 1 `scrap_metal` = +50 HP. Downed: `game_player/src/downed.rs` (self revive by holding, `selfReviveSeconds`).
- Player damage: `rotland/src/combat_damage.rs` (`take_bites`, `take_damage_with_downed`) -> `game_core/src/health.rs` `take_damage(DamageInfo)`.
- Light: `engine/src/render/lit.wgsl` (sun + moon + ambient + spot lights, then a filmic curve). Flashlight is toggled through `HandChange::ToggleFlashlight` in `play.rs`.
- Too-long files (split rule ~300 lines): `world/src/metro/traces.rs` 434, `world/src/metro/damage.rs` 375, `rotland/src/streaming.rs` 492, `rotland/src/play.rs` 805. New code must not make these longer.

## Owner decisions this round follows (PLAN_1.1 R41 + Claude defaults, all numbers in data so the owner can change them)
- **Danger rises toward the centre.** Edge is about like a town, Midtown harder, Core the hardest place in the game.
- **Night belongs to runners.** At night most city zombies are runners, most of all in the Core.
- **Herds walk slowly down the avenues**: groups of 12-30 walkers moving at a slow shamble along long streets, turning at crossings. They don't come for you unless they see or hear you; then the whole herd turns.
- **Silent life:** crows sitting on roofs, wires, cars and bodies that burst up and fly off when you come close, plus a few circling high up; torn flags and plastic sheets flapping in the wind; fallen power lines (poles down, wires across the street); faded billboards with generic, made-up ads (no real brands or names).
- **Loot by district:** Edge = everyday city loot, Midtown = better, Core = the best. City-only items:
  - army gear: `body_armor`, `army_helmet`, `army_rifle`, `army_ration` (+ ammo `ammo_556`)
  - `night_vision` goggles that run on `battery`
  - rare medicine: `antibiotics`, `surgery_kit`, `adrenaline_shot`
  - rare parts: `gun_parts`, `car_parts`
- **Claude defaults (2026-11-27; tell Claude if the owner disagrees):**
  - **No infection system.** Antibiotics heal slowly over time instead.
  - Armour is carried in the bag like the warm jacket. The best one per slot counts.
  - Night vision is a hand item: use it to switch it on and off. It drains battery charge.
- No sounds. Show-level gore only. Detail first. **FPS:** measure the `metro` scene only (`tools/perf_scenes.sh --set metro`), before and after. Pass = **average FPS at least 20**; the 1% low is for information only.

## R41 shared names (every task uses EXACTLY these)
- **Data.** New top-level objects: `data/spawning.json` `metro` (R41-01), `data/metro.json` `herds` (R41-02), `data/metro.json` `life` (R41-03). Data files owned by one task:
  - `data/crows.json` (R41-05)
  - item lines in `data/items.json`, the `army_rifle` line in `data/weapons.json` and `data/gun_feel.json`, `data/armor.json` (R41-06)
  - `metro_*` tables in `data/loot_tables.json` (R41-08)

  Each task adds ONLY its own keys; never rewrite a whole file.
- **Metro spawn profile** (R41-01, pure, `game_zombie/src/metro_spawn.rs`):
  - `pub struct MetroSpawnConfig` (serde from `spawning.json` `metro`).
  - `pub enum MetroZone { Edge, Midtown, Core }` (mirror of `world::MetroDistrict`; the converter `From<world::MetroDistrict>` lives in the same file).
  - `pub fn metro_profile(cfg: &MetroSpawnConfig, zone: MetroZone, is_night: bool) -> MetroSpawnProfile { density: f32, max_active: u32, types: Vec<TypeWeight>, special_chance_bonus: f32 }`.
  - `SpawnContext` gets one new field `pub metro: Option<MetroSpawnProfile>`. When it is `Some`, it wins over city/town/biome in `calculate_cap` and `pick_spawn_type_id`.
- **Herd routes** (R41-02):
  - Pure world part, `world/src/metro/herds.rs`:
    - `pub struct HerdRoute { pub id: u32, pub points: Vec<Vec2>, pub size: u32, pub speed: f32, pub zone: MetroDistrict }`
    - `pub fn herd_routes(layout: &MetroLayout, cfg: &HerdsConfig, seed: u64) -> Vec<HerdRoute>`
    - `pub fn route_pos(route: &HerdRoute, time_s: f64) -> (Vec2, f32 /*yaw*/)`: a loop, deterministic from world time.
  - Runtime part, `game_zombie/src/street_herds.rs`: `pub struct StreetHerds` with `update(dt, world_time, player_pos, ...) -> StreetHerdOrders { spawn: Vec<(u32 /*route*/, Vec<Vec3>)>, despawn_routes: Vec<u32> }`. A herd exists as zombies only while the player is within `herds.activeRadius`; otherwise it is just a position on its route.
- **Life placement** (R41-03, pure, `world/src/metro/life.rs` + `life_wires.rs`):
  - `pub fn life_props_in_chunk(metro: &MetroWorld, chunk: (i32, i32), height: &dyn Fn(f32, f32) -> f32) -> Vec<PropInstance>`: static poles, wires, billboards, flag poles.
  - `pub fn flappers_in_chunk(...) -> Vec<Flapper>` with `Flapper { kind: FlapKind /*Flag, Plastic*/, pos: Vec3, yaw: f32, scale: f32, phase: f32 }`.
  - `pub fn crow_perches_in_chunk(...) -> Vec<CrowPerch>` with `CrowPerch { pos: Vec3, yaw: f32, kind: PerchKind /*Roof, Wire, Car, Body, Street*/ , count: u8 }`.
- **Life models** (R41-04, Blender `tools/blender/life/`). Static ones go in category `props`, animated frames in `props`, crows in `animals`:
  - `power_pole`, `power_pole_leaning`, `power_pole_fallen`
  - `power_wire_span`: one sagging span 20 m long along +X, scaled in X to fit. 3 wires.
  - `power_wire_down`: wires lying in loose curves on the ground, 20 m.
  - `billboard_roof_a`, `billboard_roof_b`: steel frame + catwalk, 12 x 4 m face.
  - `billboard_street`: on one pole, 8 x 3 m face.
  - The billboard FACE is a separate model: `billboard_face_a`, `billboard_face_b`, `billboard_face_c`, `billboard_face_d`. Faded, peeling, made-up ads: soda, phone, holiday, insurance. Big shapes and plain words like "SUMMER SALE", "CALL NOW", "FLY AWAY". No real brands.
  - `flag_pole_wall`: a short pole sticking out of a wall at 45 degrees.
  - `flag_torn_f0`, `flag_torn_f1`, `flag_torn_f2`, `flag_torn_f3`: 4 frames of a torn flag waving. Same pivot at the pole end.
  - `plastic_flap_f0` .. `plastic_flap_f3`: a torn plastic sheet hanging from a fence/car wall edge, 4 frames.
  - `crow_perched`, `crow_peck`, `crow_fly_f0` .. `crow_fly_f3` (wing beat). Each crow is 0.45 m long, under 400 tris.
- **Crows + flappers runtime** (R41-05):
  - Pure logic in `game_world/src/crows/` (`mod.rs`, `flock.rs`, `tests.rs`): `CrowFlock` states `Perched -> Startled -> Flying -> Gone -> Returning`, and `pub fn flap_frame(phase: f32, time: f32, rate: f32) -> usize` (0..4).
  - Drawing in `rotland/src/metro_life.rs` (new): it keeps flocks for perches near the player and draws crows and flappers as instanced moving models (like `rotland/src/wildlife.rs` but with rigid models, no skin).
- **City items** (R41-06): ids exactly as in "Owner decisions" above. Mechanics:
  - Armour: `data/armor.json` maps item id -> `{ slot, reduce }`. `pub fn armor_reduction(inventory, armor_cfg) -> f32` lives in `game_core/src/armor.rs`.
  - Heal over time and adrenaline: `game_player/src/survival/boosts.rs` with `pub struct Boosts`.
  - Night-vision battery: `game_player/src/night_vision.rs` with `pub struct NightVision`.
  - Gun repair with parts: `game_core/src/weapons/` wear, function `repair_with_parts`.
  - Car repair with parts: `VehicleManager::repair_with_parts` next to `repair`.
- **Item models + icons** (R41-07, Blender):
  - World/hand models: `body_armor`, `army_helmet`, `army_rifle` (weapon, with mount points like the R37 guns), `army_ration`, `night_vision`, `battery`, `antibiotics`, `surgery_kit`, `adrenaline_shot`, `gun_parts`, `car_parts`, `ammo_556`.
  - Crate models: `army_crate`, `metro_locker`, `med_cabinet`, `camp_stash`.
  - One icon per item, made the same way as the existing item icons.
- **District loot** (R41-08):
  - `LootSpot` gets `#[serde(default)] pub table: Option<String>` and `#[serde(default)] pub model: Option<String>`. `crates.rs` uses `table` when it is set, else the biome table, and draws `model` when it is set, else `crate_loot`.
  - Pure placement in `world/src/metro/loot.rs`: `pub fn metro_loot_in_chunk(metro: &MetroWorld, chunk: (i32, i32), height: &dyn Fn(f32, f32) -> f32, seed: u64) -> Vec<LootSpot>`.
  - Tables: `metro_edge`, `metro_midtown`, `metro_core`, `metro_army`, `metro_medical`, `metro_camp`, `metro_car`.
- **Wiring + flags + night vision look** (R41-09):
  - Flags: `--metro=herd`, `--metro=crows`, `--metro=army`, `--metro=night` (the Core at hour 23), `--nvg` (starts with night vision on and a battery).

## Tests that must stay green
`world/tests/golden.rs` passes UNEDITED (home land, Ashford, camp, towns unchanged). Spawning outside New Halden is unchanged (a test proves it). R38-R40 metro tests still pass. Old saves load (new item ids are only added).
