# R32-03: weather in the game loop (fog, sky tint, zombie senses, save, dev flag)

Owns: rotland/src/light.rs, rotland/src/weather_play.rs (new), rotland/src/app/render_frame.rs (only the weather draw + fog lines), rotland/src/app/test_flags.rs (only the `--weather=` flag), game_zombie/src/zombie/senses.rs, save/src/systems/world.rs (only the weather field), tools/README_TOOLS.md (one line for `--weather=`)
Files: rotland/src/light.rs, rotland/src/weather_play.rs, rotland/src/app/render_frame.rs, rotland/src/app/test_flags.rs, game_zombie/src/zombie/senses.rs, save/src/systems/world.rs, tools/README_TOOLS.md
Read: tasks/R32-round.md, tasks/LEAD.md, rotland/src/light.rs, rotland/src/app/render_frame.rs, rotland/src/app/test_flags.rs, game_zombie/src/zombie/senses.rs, save/src/systems/world.rs, tools/README_TOOLS.md
API: rotland/src/app/state.rs, world/src/generator.rs, world/src/biome/registry.rs, world/src/lib.rs, engine/src/render/water.rs, save/src/systems/mod.rs
Data: data/biomes.json
Overlap: `rotland/src/app/test_flags.rs` and `tools/README_TOOLS.md` are also named in R32-07. R32-03 keeps them (lower number): write them here.
Rules: `tasks/LEAD.md` subagent rules. Use the "R32 shared names" in `tasks/R32-round.md` EXACTLY. R32-01 makes `world::weather` and R32-02 makes `engine::render::weather` at the same time: code against the shared names.

1. **`rotland/src/weather_play.rs`:** `WeatherPlay`, holding the `WeatherState`, `WeatherData` and `WeatherRenderer`.
   - `fn tick(&mut self, dt_game: f32, player_pos: Vec3, generator: &WorldGenerator)` uses the player's primary biome (`generator.biome(x, z)`).
   - `fn params(&self, camera_pos, time) -> WeatherParams`.
   - Report the lines the lead must add (a field in the app state, a tick call, a draw call after the water pass). Find the right places with grep (`water` draw, the `day/night` tick).
2. **Fog and sky** (`light.rs` `at_hour`):
   - Add an input for the weather fog mult and a tint. `fog_begin` and `fog_end` are scaled by `fog_mult` (never below 25 m / 45 m).
   - The fog colour is mixed toward the biome `fog_tint` (R31 data: `BiomeRegistry::get(primary).fog_tint`, mixed by `outer`), then toward the weather tint by intensity × 0.5.
   - Keep `at_hour(hour, spot)` working for old callers (e.g. a new `at_hour_weather(...)`), so other code doesn't break.
   - **At night, fog must stay dark:** mix the tints with the night fog colour, don't brighten it.
3. **Zombie senses** (`game_zombie/src/zombie/senses.rs`): sight range × `visibility`, and hearing radius × (1 - `noise_mask`).
   - Pass plain `f32`s through whatever context the senses already take (read it first). Add `sight_mult: f32` and `hearing_mult: f32`, both defaulting to 1.0, so existing tests stay correct.
   - Add 2 unit tests for the multipliers.
4. **Save:** store the `WeatherState` in the world save part (`save/src/systems/world.rs`) as an optional field. Old saves without it load as clear weather.
5. **`--weather=<kind>` dev flag:** force that kind at intensity 1 from the start, with no changes for the whole run (for screenshots).

## Worker notes
