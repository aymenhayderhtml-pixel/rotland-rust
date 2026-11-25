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
