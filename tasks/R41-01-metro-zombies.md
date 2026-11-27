# R41-01: Metro zombies (danger to the centre, runners at night)

Owns: `game_zombie/src/metro_spawn.rs` (NEW), `game_zombie/src/metro_spawn_tests.rs` (NEW), `game_zombie/src/spawner.rs` (ONLY the new `metro` field in `SpawnContext` + `SpawnContext::new`, and the `metro` branches in `calculate_cap` and `pick_spawn_type_id`), `game_zombie/src/spawner_config.rs` (ONLY loading the optional `metro` block), `data/spawning.json` (ONLY the new top-level key `metro`).
Lines for the lead: `mod metro_spawn;` + `pub use metro_spawn::*;` and `#[cfg(test)] mod metro_spawn_tests;` in `game_zombie/src/lib.rs`.
Not owned: `rotland/src/play.rs` (R41-09 wires it), everything else.
Read: tasks/R41-common.md, `game_zombie/src/spawner.rs` (whole), `game_zombie/src/spawner_config.rs`, `data/spawning.json`, `world/src/metro/layout.rs` (`MetroDistrict`, `district_at`).

## Data (`data/spawning.json` -> `metro`)
```json
"metro": {
  "_why": "R41: New Halden danger rises toward the centre; night belongs to runners.",
  "zones": {
    "edge":    { "density": 2.0, "maxActive": 30, "specialBonus": 0.00,
                 "day":   [ {"id":"walker","weight":0.9}, {"id":"runner","weight":0.1} ],
                 "night": [ {"id":"walker","weight":0.5}, {"id":"runner","weight":0.5} ] },
    "midtown": { "density": 2.6, "maxActive": 38, "specialBonus": 0.05,
                 "day":   [ {"id":"walker","weight":0.8}, {"id":"runner","weight":0.2} ],
                 "night": [ {"id":"walker","weight":0.35}, {"id":"runner","weight":0.65} ] },
    "core":    { "density": 3.2, "maxActive": 46, "specialBonus": 0.10,
                 "day":   [ {"id":"walker","weight":0.7}, {"id":"runner","weight":0.3} ],
                 "night": [ {"id":"walker","weight":0.2}, {"id":"runner","weight":0.8} ] }
  },
  "nightMaxActiveMultiplier": 1.4
}
```
The global `hardCap` (60) still applies on top.

## Rust
- `MetroSpawnConfig`, `ZoneSpawn`, `MetroZone`, `MetroSpawnProfile` with the names from the common file. `From<world::MetroDistrict> for MetroZone`.
- `metro_profile(cfg, zone, is_night)`:
  - `density` and `max_active` come from the zone. At night `max_active` is multiplied by `nightMaxActiveMultiplier`.
  - `types` is the zone's day or night list.
  - `special_chance_bonus` is `specialBonus`.
- `SpawnContext.metro: Option<MetroSpawnProfile>` (default `None` in `new`).
- In `calculate_cap`, when `metro` is `Some(p)`:
  - density = `p.density`
  - `day_cap = min(day_cap, p.max_active * tier_mult)`
  - the normal night factor is NOT applied a second time, because the profile already holds the night value.
  - The camp factor stays.
- In `pick_spawn_type_id`, when `metro` is `Some(p)`:
  - The biome specials are skipped (no frozen/bloater in the city).
  - The global special roll uses `special_chance + p.special_chance_bonus`.
  - Otherwise pick from `p.types` by weight, with the spawner rng.
  - The function gets the profile through a new argument. Update its one caller inside `update`.
- Every number has a `// data: spawning.json metro...` tag.

## Tests (`metro_spawn_tests.rs`)
- Loading the real `data/spawning.json` gives 3 zones.
- The Core cap is higher than the Midtown cap, and the Midtown cap is higher than the Edge cap, both day and night.
- The runner share at night is higher than in the day for every zone. Core night runner share >= 0.75 (over 2000 picks with a fixed seed).
- With `metro: None`, `calculate_cap` and the picks are exactly what they were before (same seed, same results as the old code; test a biome case and an Ashford city case).
- In the metro, no biome special is ever picked (swamp biome name + metro profile, 2000 picks: no `bloater`).

## Done when
The code and tests are written (the lead builds and tests). Fill `## Worker notes
- **Files changed:**
  - `data/spawning.json`: added top-level `"metro"` key with `zones` (`edge`, `midtown`, `core`) and `nightMaxActiveMultiplier` (1.4).
  - `game_zombie/src/metro_spawn.rs` (NEW): `MetroZone` (`From<world::MetroDistrict>`), `TypeWeight` (`= SpawnTypeEntry`), `ZoneSpawn`, `MetroZones`, `MetroSpawnConfig`, `MetroSpawnProfile`, and `metro_profile(cfg, zone, is_night)`.
  - `game_zombie/src/spawner_config.rs`: added `#[serde(default)] pub metro: Option<MetroSpawnConfig>` to `SpawnConfig`, and included metro type IDs in `all_type_ids`.
  - `game_zombie/src/spawner.rs`: added `pub metro: Option<MetroSpawnProfile>` to `SpawnContext` (default `None` in `SpawnContext::new`), added `metro` branches in `calculate_cap` (uses `p.density`, caps `day_cap` at `p.max_active * tier_mult`, skips global `night_max_active_multiplier` since `p.max_active` already includes it, keeps camp factor) and `pick_spawn_type_id` (takes `metro: Option<&'a MetroSpawnProfile>`, skips biome specials, rolls global specials with `special_chance + p.special_chance_bonus`, otherwise picks from `p.types`), and updated the call inside `try_spawn_one`.
  - `game_zombie/src/metro_spawn_tests.rs` (NEW): 5 unit tests covering `spawning.json` metro zones, cap ordering day/night, runner share day vs night & core night >= 0.75, unchanged non-metro biome & Ashford city behavior, and no biome specials in metro.
- **Exact `game_zombie/src/lib.rs` lines for the lead:**
  ```rust
  pub mod metro_spawn;
  #[cfg(test)]
  mod metro_spawn_tests;
  ```
  and in `pub use`:
  ```rust
  pub use metro_spawn::*;
  ```
- **Callers in unowned files that need updating (for lead / R41-09):**
  - `rotland/src/play.rs:535` (`SpawnContext { ... }` literal) needs `metro: ...` (or `metro: None` before R41-09 wiring).
  - `game/tests/all/biome_zombies_tests.rs:61` and `80` call `pick_spawn_type_id(..., 0.0)` / `(..., 0.25)` with 5 arguments and need `, None` as the 6th argument (`metro`).
- **Not run:** Per subagent rules, did not run `cargo`, `tools/gate.sh`, `blender`, or the game.` with the lib.rs lines.

## Worker notes
- **Files changed:**
  - `data/spawning.json`: added top-level `"metro"` key with `zones` (`edge`, `midtown`, `core`) and `nightMaxActiveMultiplier` (1.4).
  - `game_zombie/src/metro_spawn.rs` (NEW): `MetroZone` (`From<world::MetroDistrict>`), `TypeWeight` (`= SpawnTypeEntry`), `ZoneSpawn`, `MetroZones`, `MetroSpawnConfig`, `MetroSpawnProfile`, and `metro_profile(cfg, zone, is_night)`.
  - `game_zombie/src/spawner_config.rs`: added `#[serde(default)] pub metro: Option<MetroSpawnConfig>` to `SpawnConfig`, and included metro type IDs in `all_type_ids`.
  - `game_zombie/src/spawner.rs`: added `pub metro: Option<MetroSpawnProfile>` to `SpawnContext` (default `None` in `SpawnContext::new`), added `metro` branches in `calculate_cap` (uses `p.density`, caps `day_cap` at `p.max_active * tier_mult`, skips global `night_max_active_multiplier` since `p.max_active` already includes it, keeps camp factor) and `pick_spawn_type_id` (takes `metro: Option<&'a MetroSpawnProfile>`, skips biome specials, rolls global specials with `special_chance + p.special_chance_bonus`, otherwise picks from `p.types`), and updated the call inside `try_spawn_one`.
  - `game_zombie/src/metro_spawn_tests.rs` (NEW): 5 unit tests covering `spawning.json` metro zones, cap ordering day/night, runner share day vs night & core night >= 0.75, unchanged non-metro biome & Ashford city behavior, and no biome specials in metro.
- **Exact `game_zombie/src/lib.rs` lines for the lead:**
  ```rust
  pub mod metro_spawn;
  #[cfg(test)]
  mod metro_spawn_tests;
  ```
  and in `pub use`:
  ```rust
  pub use metro_spawn::*;
  ```
- **Callers in unowned files that need updating (for lead / R41-09):**
  - `rotland/src/play.rs:535` (`SpawnContext { ... }` literal) needs `metro: ...` (or `metro: None` before R41-09 wiring).
  - `game/tests/all/biome_zombies_tests.rs:61` and `80` call `pick_spawn_type_id(..., 0.0)` / `(..., 0.25)` with 5 arguments and need `, None` as the 6th argument (`metro`).
- **Not run:** Per subagent rules, did not run `cargo`, `tools/gate.sh`, `blender`, or the game.
