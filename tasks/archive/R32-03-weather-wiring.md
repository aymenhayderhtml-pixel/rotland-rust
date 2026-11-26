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

## Neon notes

### What was implemented
1. **`rotland/src/weather_play.rs`**: Added `WeatherPlay` holding `WeatherState`, `WeatherData`, and `WeatherRenderer`, with `tick`, `params`, `light_overrides`, `visibility`, `noise_mask`, `force`, `to_save`, `apply_save`, and `draw`.
2. **`rotland/src/light.rs`**: Added `WeatherLight`, `rgb_u8_to_vec3`, and `at_hour_weather(hour, spot, weather)` while keeping `at_hour(hour, spot)` intact. Fog distances scale by `fog_mult` (clamped to `25 m` / `45 m`). Fog mixes toward `BiomeDef::fog_tint` by `outer`, then toward `weather_tint` by `intensity * 0.5`; sky horizon/top also shift subtly during active weather. Tints are luminance-capped to the base colour so night fog/sky never brighten. Added 4 unit tests.
3. **`rotland/src/app/render_frame.rs`**: Wired `light::at_hour_weather` for fog/sky and `weather.draw(...)` immediately after `crate::water_view::draw(...)`. Also restored three `* Mat4::from_...` multiplications that had been mangled to `- Mat4::from_...` in the prompt pack.
4. **`game_zombie/src/zombie/senses.rs`**: Added `SenseMultipliers { pub sight_mult: f32, pub hearing_mult: f32 }` (both default `1.0`, plus `from_weather(visibility, noise_mask)`), `Senses::set_multipliers`, and `Senses::hears` while preserving existing `Senses::update` and free `hears` signatures. Added 2 unit tests (`sight_multiplier_scales_sight_range`, `hearing_multiplier_masks_noise_radius`).
5. **`save/src/systems/world.rs`**: Added optional `weather: Option<WeatherSaveData>` on `WorldSaveData` (`skip_serializing_if = "Option::is_none"`, `#[serde(default)]`). Kept `WorldSaveData::new(seed)` unchanged so old saves load as `None` (clear weather). Note: dropped `Eq` derive from `WorldSaveData` because `WeatherSaveData` stores `f32`s (`PartialEq` remains). Added 2 unit tests.
6. **`rotland/src/app/test_flags.rs` & `tools/README_TOOLS.md`**: Added `--weather=<kind>` parsing, startup application (`weather.force(kind)`), test assertions, and README documentation.

### Shared-file lines for the lead to add
1. **`rotland/src/main.rs`**:
   ```rust
   mod weather_play;

```

2. **`rotland/src/app/state.rs`** (in `PlaySide` struct + `impl Default for PlaySide`):
```rust
pub weather: Option<crate::weather_play::WeatherPlay>,
// in Default:
weather: None,

```


3. **`rotland/src/app/start_world.rs`** (right after `world.world_gen` is created and before `test_flags::apply_startup`):
```rust
let weather_data = world::weather::WeatherData::load().unwrap_or_default(); // match R32-01's loader name
let mut weather = crate::weather_play::WeatherPlay::new(gpu, session.seed, weather_data);
if let Some(saved) = file.as_ref().and_then(|f| f.world.weather.as_ref()) {
    weather.apply_save(Some(saved));
}
play.weather = Some(weather);

```


(Call `test_flags::apply_startup` after this so `--weather=<kind>` overrides loaded weather.)
4. **Frame update (next to the `play.clock` day/night tick, e.g. `rotland/src/app/frame_update.rs` or `mod.rs`)**:
```rust
if let (Some(weather), Some(generator)) = (play.weather.as_mut(), world.world_gen.as_ref()) {
    let dt_game = if play.clock_frozen { 0.0 } else { dt * play.clock.time_scale() }; // match your Clock's game-seconds delta
    weather.tick(dt_game, actors.player, generator);
}

```


5. **Zombie senses update (in `game_zombie/src/zombie/mod.rs` and the `rotland/` zombie tick)**:
* Re-export `pub use senses::SenseMultipliers;` if desired from `game_zombie` / `game::zombie`.
* Pass `let mults = play.weather.as_ref().map_or_else(SenseMultipliers::default, |w| SenseMultipliers::from_weather(w.visibility(), w.noise_mask()));` into zombie updates, call `zombie.senses.set_multipliers(mults)`, and use `zombie.senses.hears(feet, noise, def.hearing_multiplier)` (or `hears(feet, noise, def.hearing_multiplier * mults.hearing_mult)`).


6. **World save builder (where `WorldSaveData::new(session.seed)` is built)**:
```rust
let mut world_save = save::systems::world::WorldSaveData::new(session.seed);
world_save.weather = play.weather.as_ref().map(|w| w.to_save());

```



### Known cross-task alignment points (R32-01 & R32-02 built in parallel)

* **`world::weather::WeatherData` (R32-01)**: `weather_play.rs` reads `self.data.kinds.get(kind.key())` with fields `.tint: [f32; 3]` and `.wind: f32` (matching `data/weather.json` in `tasks/R32-round.md`). Adjust `kind_tint` / `kind_wind` if R32-01 exposes a getter method instead of a `pub kinds` map.
* **`engine::render::weather::WeatherRenderer` (R32-02)**: `weather_play.rs` calls `WeatherRenderer::new(gpu)` and `self.renderer.draw(gpu, target, encoder, camera, &params)` mirroring `WaterRenderer` (`engine/src/render/water.rs`). If R32-02 took `(gpu, format, depth_format)` or `(&mut pass, &gpu.queue, view_proj, &params)`, only those two lines in `rotland/src/weather_play.rs` need adjustment.
* **`engine::render::Camera` import**: Adjust to `engine::Camera` in `weather_play.rs` if `Camera` is only exported at the crate root.
