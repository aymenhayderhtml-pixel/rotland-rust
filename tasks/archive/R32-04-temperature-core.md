# R32-04: body warmth logic + data + ambient temperature

Owns: game_player/src/survival/temperature.rs (new), game_player/src/survival/mod.rs (only the `pub mod temperature;` line + re-exports), data/temperature.json (new), world/src/biome/ambient.rs (new), world/src/biome/mod.rs (only `pub mod ambient;` + the re-export of `ambient_temperature`), game/tests/all/temperature_tests.rs (new), world/tests/ambient_tests.rs (new)
Files: game_player/src/survival/temperature.rs, game_player/src/survival/mod.rs, data/temperature.json, world/src/biome/ambient.rs, world/src/biome/mod.rs, game/tests/all/temperature_tests.rs, game/tests/all/main.rs, world/tests/ambient_tests.rs
Read: tasks/R32-round.md, tasks/LEAD.md, docs/WORLD.md, game_player/src/survival/need.rs, game_player/src/survival/config.rs, game_player/src/survival/mod.rs, world/src/biome/mod.rs, world/src/biome/registry.rs, world/tests/biome_layout_tests.rs, game/tests/all/main.rs
API: world/src/lib.rs, game_player/src/lib.rs
Data: data/biomes.json
Overlap: `game/tests/all/main.rs` is also needed by R32-05 and R32-06. R32-04 keeps it (lower number): write the `mod temperature_tests;` line and the other two modules' `mod` lines there (`mod warmth_items_tests;`, `mod biome_zombies_tests;`). Also: `world/src/lib.rs` is owned by R32-01. Report the `pub use` line that exports `ambient_temperature` to the lead instead of writing it.
Rules: `tasks/LEAD.md` subagent rules. Use the "R32 shared names" in `tasks/R32-round.md` EXACTLY. Read `game_player/src/survival/need.rs` + `config.rs` and copy their style (data loading, `from_config`).

1. **`ambient_temperature`** (world): exactly as in the shared names. Use `BiomeDef.temperature_day` / `temperature_night` from R31's `data/biomes.json`. Sun curve: a cosine with its peak at 14:00 and its low at 04:00. The weather offsets come from `data/temperature.json` (`weather_offset_c`: rain -3, snow -3, sandstorm 2, fog -1). World tests:
   - desert 14:00 > 30 °C;
   - snow 04:00 < -15 °C;
   - the home biome at 12:00 is between 12 and 26 °C;
   - halfway through the blend band, the result lies between the two biomes.
2. **`BodyTemp`** (game_player): the value (0..100, 50 = comfy) moves toward a target.
   - Target = 50 + (ambient_c - 18) × `degree_to_points` (~1.1).
   - Warmth sources pull the target toward 50: near the fire +35 points (capped at 55), in camp +20, indoors +15, insulation × 30 against cold only, sun_protection × 25 against heat only.
   - Speed: `follow_rate` per second (~0.01, so it takes minutes, not seconds).
   - States: Freezing < 15 ≤ Cold < 35 ≤ Comfy ≤ 65 < Hot ≤ 85 < Overheated.
   - Effects (all from data):
     - Cold: stamina regen × 0.7.
     - Freezing: stamina regen × 0.5, health drain 0.25/s, never below 10 health from cold alone.
     - Hot: thirst × 1.3.
     - Overheated: thirst × 1.8, stamina regen × 0.7.
   - Add `pub fn chill(&mut self, amount: f32)`, used by the frozen zombie bite (R32-07).
3. **`data/temperature.json`:** all the numbers above, with `_why` notes. Owner rule: light, never a quick death.
4. **Tests** (`game/tests/all/temperature_tests.rs`; the lead adds `mod temperature_tests;` to `game/tests/all/main.rs`, so put that line in your notes):
   - comfy at 18 °C;
   - drifts to Cold in the snow without help within ~3 game minutes, and NOT within 30 s;
   - near the fire it stays comfy;
   - insulation 0.7 keeps you at least Cold, not Freezing, at -10 °C;
   - Freezing drains health but never below 10;
   - desert noon gives thirst_mult > 1;
   - `chill` lowers the value.

## Worker notes

## Neon notes

* **What was implemented:**
* `data/temperature.json`: Complete body warmth curve (`comfy_value = 50.0`, `neutral_ambient_c = 18.0`, `degree_to_points = 1.1`, `follow_rate = 0.01`, `sun_peak_hour = 14.0`, `sun_low_hour = 4.0`), warmth sources (`near_fire +35` capped at `55`, `in_camp +20`, `indoors +15`, `insulation * 30` cold-only, `sun_protection * 25` heat-only), state thresholds (`15 / 35 / 65 / 85`), gameplay effects (`Cold` stamina ×0.7, `Freezing` stamina ×0.5 + `0.25/s` drain floored at `10` HP, `Hot` thirst ×1.3, `Overheated` thirst ×1.8 + stamina ×0.7), and weather offsets (`rain -3`, `snow -3`, `sandstorm +2`, `fog -1`), all with `_why` notes.
* `world/src/biome/ambient.rs` & `world/src/biome/mod.rs`: `ambient_temperature(registry: &BiomeRegistry, sample: &BiomeSample, hour: f32, weather: &WeatherState) -> f32` using a C¹ piecewise cosine sun curve (peak 14:00, low 04:00), primary/secondary blending via `sample.blend`, home biome (`BiomeId::Plains`) blending via `sample.outer`, and intensity-scaled weather offsets lazily loaded from `data/temperature.json`.
* `game_player/src/survival/temperature.rs` & `game_player/src/survival/mod.rs`: `BodyTemp`, `TempState`, `TemperatureData`, `WarmthSources` matching R32 shared names exactly, plus `from_config`, `target_value`, `chill`, `min_health_floor`, `health_drain_for` (enforces the 10 HP cold floor), and `to_save`/`from_save`.
* `world/tests/ambient_tests.rs` & `game/tests/all/temperature_tests.rs`: All required unit and integration tests.
* `game/tests/all/main.rs`: Added `mod biome_zombies_tests;`, `mod temperature_tests;`, and `mod warmth_items_tests;` per the overlap rule.


* **Shared-file line for the lead (`world/src/lib.rs`, owned by R32-01):**
Add the following re-export to `world/src/lib.rs` (alongside R32-01's `pub mod weather;`):
```rust
pub use biome::ambient_temperature;

```


* **Wiring notes & known risks:**
* `world/src/biome/ambient.rs` and `world/tests/ambient_tests.rs` depend on R32-01 providing `world::weather::{WeatherKind, WeatherState}` with public `kind: WeatherKind`, `intensity: f32`, and `WeatherState::new(seed: i32)` exactly per R32 shared names.
* `game/tests/all/main.rs` declares `mod warmth_items_tests;` (R32-05) and `mod biome_zombies_tests;` (R32-06); those files must be present before running `cargo test -p game`.
* `game/tests/all/temperature_tests.rs` imports `game_player::survival::temperature::...` directly; if `game/Cargo.toml` does not list `game_player` directly in `[dependencies]` or `[dev-dependencies]` (only re-exporting via `game`), swap the import prefix from `game_player::` to `game::`.
* For R32-05 wiring: apply cold health damage via `body_temp.health_drain_for(player.health, dt)` so freezing never drops health below 10 HP (and never heals if already below 10 HP).
