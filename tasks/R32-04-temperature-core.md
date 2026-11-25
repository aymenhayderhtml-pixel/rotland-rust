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
