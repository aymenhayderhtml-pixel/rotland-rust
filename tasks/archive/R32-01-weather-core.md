# R32-01: weather logic + data (world crate)

Owns: world/src/weather/* (new), world/src/lib.rs (only the `pub mod weather;` and `pub use` lines), data/weather.json (new), world/tests/weather_tests.rs (new), docs/WORLD.md (add a short "Weather" section)
Files: world/src/weather/mod.rs, world/src/weather/data.rs, world/src/lib.rs, data/weather.json, world/tests/weather_tests.rs, docs/WORLD.md
Read: tasks/R32-round.md, tasks/LEAD.md, docs/WORLD.md, world/src/lib.rs, world/src/random.rs, world/src/biome/mod.rs, world/src/biome/registry.rs, world/Cargo.toml, world/tests/biome_layout_tests.rs
API: paths/src/lib.rs
Data: data/biomes.json
Rules: `tasks/LEAD.md` subagent rules. Use the "R32 shared names" in `tasks/R32-round.md` EXACTLY.

1. `world/src/weather/mod.rs`: `WeatherKind` and `WeatherState`. `world/src/weather/data.rs`: `WeatherData` (serde, loads `data/weather.json` via `paths::repo_path`, plus `from_json(&str)` for tests). Files under ~300 lines.
2. **How weather changes:**
   - A "spell" lasts a random time between `min_spell_seconds` and `max_spell_seconds` (game seconds). At its end, pick the next kind from the CURRENT biome's weighted list (deterministic RNG from the seed; reuse `crate::random` if it fits).
   - Change of kind: the intensity fades out to 0 over `fade_seconds`, the kind switches, then the intensity fades in to 1.
   - Biome change (the player walks into another biome): if the current kind is not in the new biome's list (e.g. a sandstorm while entering snow), start the fade-out at once and pick from the new list.
   - Wind: direction drifts slowly (a few degrees a minute), with speed = `kinds[k].wind × intensity`.
3. **`data/weather.json`** (give each list a `_why`):
   - plains/forest: clear 0.7, rain 0.25, fog 0.05
   - desert: clear 0.65, sandstorm 0.35
   - snow: clear 0.35, snow 0.6, fog 0.05
   - mountains: clear 0.4, snow 0.5, fog 0.1
   - swamp: fog 0.45, rain 0.35, clear 0.2
   - kinds:
     - clear: fog_mult 1, visibility 1, noise_mask 0, wind 1
     - rain: fog_mult 0.75, visibility 0.8, noise_mask 0.35, wind 3, particles 0.06, tint grey-blue
     - snow: fog_mult 0.6, visibility 0.7, noise_mask 0.2, wind 2, particles 0.05, tint white
     - sandstorm: fog_mult 0.3, visibility 0.4, noise_mask 0.5, wind 9, particles 0.12, tint sand
     - fog: fog_mult 0.4, visibility 0.5, noise_mask 0.1, wind 0.5, particles 0, tint grey-green
4. `fog_mult()`, `visibility()` and `noise_mask()` blend between clear and the current kind by `intensity`.
5. **Save support:** `WeatherState` derives serde `Serialize`/`Deserialize` (check that `world` already depends on serde) so R32-03 can save it.
6. **Tests** (`world/tests/weather_tests.rs`):
   - the data loads, and every biome key has a non-empty list;
   - the same seed and the same steps give the same weather;
   - a sandstorm never starts in the snow;
   - walking from the desert into the snow ends the sandstorm within `fade_seconds`;
   - intensity always stays in 0..1;
   - `fog_mult` is 1 when clear.

## Worker notes

## Neon notes

* **Implemented:**
* `data/weather.json`: Added all 6 biome weather tables (`plains`, `forest`, `desert`, `snow`, `mountains`, `swamp`) with `_biomes_why`, plus all 5 weather kinds (`clear`, `rain`, `snow`, `sandstorm`, `fog`) with `_why` and normalized `[0.0..=1.0]` RGB `tint` values matching `WeatherParams.tint: [f32; 3]`.
* `world/src/weather/data.rs`: Added `WeatherData`, `WeatherKindDef`, and `WeatherWeight` with `load()` (via `paths::repo_path("data/weather.json")`), `load_from_file()`, and `from_json(&str)` validating all 6 biomes and 5 kinds.
* `world/src/weather/mod.rs`: Added `WeatherKind` (`Clear`, `Rain`, `Snow`, `Sandstorm`, `Fog` with `key()`, `from_key()`, `index()`, `as_u32()`) and `WeatherState` (`new(seed)`, `update(dt_game_seconds, biome, data)`, `set_weather(kind, intensity, data)`, `fog_mult()`, `visibility()`, `noise_mask()`). Reuses `crate::random::DotnetRandom` and tracks `(seed, rng_calls)` so `WeatherState` derives `Serialize`/`Deserialize` without modifying `world/src/random.rs` while preserving deterministic RNG continuity across save/load.
* `world/src/lib.rs`: Added `pub mod weather;` and `pub use weather::{WeatherData, WeatherKind, WeatherKindDef, WeatherState, WeatherWeight};`.
* `world/tests/weather_tests.rs`: Added integration tests covering data loading/validation, deterministic stepping, snow never starting sandstorms, desert-to-snow sandstorm fade-out within `fade_seconds` (both natural and 60 Hz ticks), intensity bounds, clear `fog_mult` and blending, wind drift/scaling, and serde round-trip determinism.
* `docs/WORLD.md`: Added `data/weather.json` to the Data Files table, noted weather table updates in "How to Add a New Biome", and added Section 4 ("Weather").


* **Caller / downstream wiring (R32-02, R32-03, R32-04):**
* Load `WeatherData::load()` once at startup and create `WeatherState::new(seed)` (or deserialize from save).
* Each frame call `weather.update(dt_game_seconds, current_biome, &weather_data)`.
* For `--weather=<kind>` in R32-03, call `weather.set_weather(kind, 1.0, &weather_data)`.
* Pass `kind.as_u32()`, `weather.intensity`, `[weather.wind.x, weather.wind.y]`, and `weather_data.kind(weather.kind).tint` into `WeatherParams` (R32-02/R32-03).


* **Notes / risks:**
* `world/src/math.rs` was not in the read list, so private `lerp_f32` and `smoothstep_01` helpers were written locally in `world/src/weather/mod.rs` to avoid assuming uninspected signatures.
* `WeatherKindDef.tint` uses normalized `0.0..=1.0` `[f32; 3]` floats to match `WeatherParams.tint: [f32; 3]` directly; R32-03 should keep this scale when blending with `BiomeDef.fog_tint` (which is `[u8; 3]` in `0..=255`).
* `WeatherData::load()` uses `paths::repo_path("data/weather.json")` as specified; switch to `paths::try_repo_path` if non-panicking repo-root lookup is desired later.
