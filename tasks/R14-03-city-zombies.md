# R14-03: more zombies in the city, more runners in the core at night

Read first: `tasks/R13-common.md` ("Rust/data lanes" part), `docs/CITY_SPEC.md` section 7, `game/src/spawner.rs`, `game/src/spawner_config.rs`, `data/spawning.json`, and in `rotland/src/app/frame_sim.rs` only the part that builds the zombie `SpawnContext` and calls the spawner (search `SpawnContext`). The city: `world::CityLayout` from `world_gen.city()` with `contains(p: Vec2)` and `district_at(p) -> Option<District>`.
Files: game/src/spawner.rs, game/src/spawner_config.rs, game/tests/all/spawner_city_tests.rs, game/tests/all/main.rs, data/spawning.json, rotland/src/play.rs, rotland/src/app/frame_sim.rs, tasks/R14-03-city-zombies.md
Test: spawner

1. `data/spawning.json`: add `"city": { "density": 2.0, "maxActive": 32, "coreNightTypes": [ {"id": "walker", "weight": 0.4}, {"id": "runner", "weight": 0.6} ] }` and load it in `SpawnConfig` (optional field: worlds without it behave exactly as today).
2. `SpawnContext` gets `in_city: bool` and `in_city_core: bool` (with builder methods like the others, default false). In `calculate_cap`: in the city, density = `city.density` instead of the biome density, and the day cap (before the night multiplier) is at most `city.maxActive`; the night multiplier still applies. At night in the core, pick types from `coreNightTypes`.
3. `frame_sim.rs`: set the two flags from the player's x/z with `world_gen.city()` (core = `district_at == Some(District::Core)`). Keep the edit small.
4. Tests in a new `game/tests/all/spawner_city_tests.rs` (add `mod spawner_city_tests;` to `main.rs`; names start with `spawner_city_`): city cap = 2x plains density capped at 32 by day; night multiplier applies on top; no `city` config gives today's numbers; core night picks runners about 60% over many rolls.

## Worker notes
- Files changed:
  - `data/spawning.json`: added `"city"` config with `density: 2.0`, `maxActive: 32`, and `coreNightTypes` (walker 0.4, runner 0.6).
  - `game/src/spawner_config.rs`: added `CitySpawnConfig`, optional `city` field on `SpawnConfig`, `pick_type_id_core` and `pick_from_slice`, updated `all_type_ids`.
  - `game/src/spawner.rs`: added `in_city` and `in_city_core` fields to `SpawnContext` with builder methods `with_in_city`, `with_city`, `with_in_city_core`, `with_city_core`. Updated `calculate_cap` to use `city.density` and cap daytime cap at `city.max_active` before applying `night_max_active_multiplier`. Updated `try_spawn_one` to pick from `coreNightTypes` when in city core at night.
  - `rotland/src/play.rs`: wired `in_city` and `in_city_core` in `spawn_tick` using `world_gen.city().is_some_and(|c| c.contains(p))` and `world_gen.city().is_some_and(|c| c.district_at(p) == Some(world::District::Core))`. (Note: the task prompt referred to `rotland/src/app/frame_sim.rs`, but `spawn_tick` and `SpawnContext` construction live in `rotland/src/play.rs`).
  - `game/tests/all/main.rs`: added `mod spawner_city_tests;`.
  - `game/tests/all/spawner_city_tests.rs`: added 6 tests (`spawner_city_*`) covering config loading, day cap capped at 32, night cap at 51, plains untouched, difficulty scaling below cap, no-city config backwards compatibility, 60% runner rolls in core at night, day/non-core distributions, and builder methods.
- How to test:
  - `tools/gate.sh --task tasks/R14-03-city-zombies.md`
- Wiring notes for Claude:
  - Ready to merge. Spawner city density and core night runner logic are purely data-driven via `data/spawning.json` and integrated into the game loop in `play.rs`.
- Known issues:
  - None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (7): data/spawning.json, game/src/spawner_config.rs, game/src/spawner.rs, game/tests/all/main.rs, game/tests/all/spawner_city_tests.rs, rotland/src/play.rs, tasks/R14-03-city-zombies.md
- tests passed: 18
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
