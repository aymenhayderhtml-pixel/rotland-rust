# R33-10: R32 leftovers (warmth saved, cold slows stamina, indoors warms, short rain streaks near the camera)

Owns: rotland/src/temperature_play.rs, save/src/systems/*.rs (only the warmth field, next to where R32 saved weather), engine/src/render/weather.wgsl, game_player/src/survival/stamina*.rs or wherever stamina regen lives (find it), game/tests/all/warmth_save_tests.rs (new)
Files: rotland/src/temperature_play.rs, save/src/systems/world.rs, engine/src/render/weather.wgsl, game_player/src/player.rs, game/tests/all/warmth_save_tests.rs
Read: tasks/R33-round.md, tasks/LEAD.md, docs/WORLD.md, tasks/R32-round.md, tasks/R32-05-temperature-play.md, rotland/src/temperature_play.rs, save/src/systems/world.rs, engine/src/render/weather.wgsl, engine/src/render/weather.rs, game_player/src/player.rs, game_player/src/survival/temperature.rs, rotland/src/app/frame_warmth.rs
API: game_player/src/lib.rs, game_player/src/survival/mod.rs, save/src/systems/mod.rs, world/src/lib.rs
Data: data/temperature.json
Overlap: `game/tests/all/main.rs` is R33-09's (it writes `mod warmth_save_tests;`). `rotland/src/app/frame_warmth.rs` still passes `indoors: false` (line ~51): that call site is not in your Files; report `generator.indoors(x, z)` there to the lead. Stamina callers in rotland/ that should pass the new multiplier are also not in your Files: report them.
Rules: `tasks/LEAD.md` subagent rules. Use the "R33 shared names" in `tasks/R33-round.md` EXACTLY. Read `tasks/R32-round.md` "Lead notes" and `tasks/R32-05-temperature-play.md` "Neon notes" first: they list what was left undone.

1. **Warmth saved:** store the `BodyTemp` value in the save (an optional field; old saves load 50 = comfy). Copy how R32 stored the weather (`save/src/systems/world.rs`) or the player survival needs. Add a round-trip test.
2. **Cold slows stamina:** `BodyTemp::stamina_regen_mult()` exists, but nothing applies it. Find the stamina regen code (grep `stamina`) and multiply the regen by it. Pass it in as a plain `f32` with a default of 1.0, so other callers and tests don't change.
3. **Indoors warms you:** `WarmthSources.indoors` is always false. Use `generator.indoors(x, z)` (R33-04 adds it).
4. **Rain streaks near the camera** (`weather.wgsl`): raindrops close to the camera draw as long white sticks across the screen. Fix:
   - fade rain out fully within 2.5 m of the camera (today it's 0.5 m);
   - clamp the streak's screen length: shorten the streak when it's near, so it's never longer than ~6% of the screen height;
   - lower the near alpha.
   Snow and sand keep their look.
5. **Tests:** the save round-trip (warmth 22 saved and loaded as 22, old save → 50); the stamina regen multiplier of 0.5 halves regen.

## Worker notes

### What changed
- `save/src/systems/world.rs`: Added `pub body_temp: Option<f32>` (`#[serde(default, skip_serializing_if = "Option::is_none")]`) to `WorldSaveData`, initialized to `None` in `WorldSaveData::new`. Added unit tests `old_save_without_body_temp_loads_as_none` and `world_save_round_trips_body_temp` (22.0).
- `rotland/src/temperature_play.rs`: Added `TemperaturePlay::apply_save(&mut self, saved: Option<f32>)` (restores finite saved value, or resets to `self.data.comfy_value` = 50.0 on `None`).
- `game_player/src/player.rs`: Added `pub warmth_regen_multiplier: f32` (default `1.0`) to `Stamina`, and multiplied `Stamina::update` regen by `self.warmth_regen_multiplier` so `update_stamina_regen_multiplier` (hunger/thirst) does not overwrite it.
- `engine/src/render/weather.wgsl`: For rain only (`kind != 2.0 && kind != 3.0`), faded out fully within `2.5 m` (`RAIN_NEAR_MIN = 2.5`), lowered near alpha with `smoothstep(2.5, 7.0, dist)` (`RAIN_NEAR_MAX = 7.0`), and clamped projected streak half-length in NDC to `0.06` (`~6%` of screen height). Snow and sandstorm keep `NEAR_FADE = 0.5` and their existing geometry/alpha.
- `game/tests/all/warmth_save_tests.rs`: Added tests for `BodyTemp` save round-trip (22.0 -> 22.0, empty JSON -> 50.0), `Stamina` regen with `warmth_regen_multiplier = 0.5` halving regen, and `BodyTemp` freezing `stamina_regen_mult()` applied to `Stamina`.

### Shared-file lines for the lead
1. `game/tests/all/main.rs`:
   ```rust
   mod warmth_save_tests;
   ```
2. `rotland/src/session.rs`:
   - In `pub struct Snapshot<'a>` (after `weather`):
     ```rust
     /// Player body warmth snapshot (R33-10); `None` loads as 50.0 (comfy).
     pub body_temp: Option<f32>,
     ```
   - In `pub fn capture(...)` (`sys.world = Some(...)`):
     ```rust
     sys.world = Some(save::WorldSaveData { seed: s.seed, weather: s.weather.clone(), body_temp: s.body_temp });
     ```
3. `rotland/src/app/save_load.rs` (in `save_now`, inside `session::Snapshot { ... }`):
   ```rust
   body_temp: play.temperature.as_ref().map(|t| t.value()),
   ```
4. `rotland/src/app/start_world.rs` (inside `TemperaturePlay::new(...)` `Ok(mut temp)` arm):
   ```rust
   Ok(mut temp) => {
       if let Some(file) = session.loaded_save.as_ref() {
           temp.apply_save(file.systems.world.as_ref().and_then(|w| w.body_temp));
       }
       play.temperature = Some(temp);
   }
   ```
5. `rotland/src/app/frame_warmth.rs`:
   - Line 51 (`TemperatureTickInput`): replace `indoors: false,` with:
     ```rust
     indoors: generator.indoors(player.x, player.z),
     ```
   - After `let res = temp.tick(...);`, apply stamina regen multiplier:
     ```rust
     if let Some(p) = actors.player_ctrl.as_mut() {
         p.stamina.warmth_regen_multiplier = res.stamina_regen_mult;
     }
     ```

### How to see it
- Run `--weather=rain` in camp and verify no long white streaks appear near the camera.
- Enter snow/cold biome, verify stamina regenerates slower when Cold/Freezing, save & reload, and verify warmth value persists.
- Step inside a town/POI building footprint (`generator.indoors(x, z)`) and verify warmth recovers toward comfy.

### Unsure / notes
- Did not run `cargo` or `tools/gate.sh` per subagent rules. Lead needs to apply the shared-file lines above and run the gate.
