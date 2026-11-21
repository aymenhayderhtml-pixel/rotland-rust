# R14-03: more zombies in the city, more runners in the core at night

Read first: `tasks/R13-common.md` ("Rust/data lanes" part), `docs/CITY_SPEC.md` section 7, `game/src/spawner.rs`, `game/src/spawner_config.rs`, `data/spawning.json`, and in `rotland/src/app/frame_sim.rs` only the part that builds the zombie `SpawnContext` and calls the spawner (search `SpawnContext`). The city: `world::CityLayout` from `world_gen.city()` with `contains(p: Vec2)` and `district_at(p) -> Option<District>`.
Files: game/src/spawner.rs, game/src/spawner_config.rs, game/tests/all/spawner_city_tests.rs, game/tests/all/main.rs, data/spawning.json, rotland/src/app/frame_sim.rs, tasks/R14-03-city-zombies.md
Test: spawner

1. `data/spawning.json`: add `"city": { "density": 2.0, "maxActive": 32, "coreNightTypes": [ {"id": "walker", "weight": 0.4}, {"id": "runner", "weight": 0.6} ] }` and load it in `SpawnConfig` (optional field: worlds without it behave exactly as today).
2. `SpawnContext` gets `in_city: bool` and `in_city_core: bool` (with builder methods like the others, default false). In `calculate_cap`: in the city, density = `city.density` instead of the biome density, and the day cap (before the night multiplier) is at most `city.maxActive`; the night multiplier still applies. At night in the core, pick types from `coreNightTypes`.
3. `frame_sim.rs`: set the two flags from the player's x/z with `world_gen.city()` (core = `district_at == Some(District::Core)`). Keep the edit small.
4. Tests in a new `game/tests/all/spawner_city_tests.rs` (add `mod spawner_city_tests;` to `main.rs`; names start with `spawner_city_`): city cap = 2x plains density capped at 32 by day; night multiplier applies on top; no `city` config gives today's numbers; core night picks runners about 60% over many rolls.

## Worker notes

