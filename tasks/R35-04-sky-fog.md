# R35-04: clouds and biome fog

Owns: engine/src/render/sky.rs + sky.wgsl (new, or inside lit.wgsl's sky code if that is where the sky is drawn), data/sky.json (new), rotland/src/sky_play.rs (new)
Files: engine/src/render/*, data/sky.json, rotland/src/sky_play.rs
Read only: engine/src/render/lit.wgsl and lit.rs (find how the sky/background and fog are drawn), world/src/weather (cloud cover per weather), rotland/src/weather_play.rs.

- Clouds: cheap, low-poly look: soft flat-shaded cloud shapes from 2D noise on a sky dome/plane, drifting with wind; cover follows weather (clear → few, rain/snow → many and greyer). One draw call, no textures to load.
- Fog: lerp the fog colour toward a per-biome tint (desert warm, snow blue-white, swamp green-grey, mountains cool grey) using the player's biome blend; keep the home land fog exactly as today when the player is in the core.
- Engine change must not cost more than ~0.5 ms (no per-pixel loops).
- Report the call sites for the lead.

## Worker notes
- **Files added (owned):**
  - `engine/src/render/sky.rs` + `engine/src/render/sky.wgsl`: `SkyRenderer` & `SkyParams`. Single full-screen triangle at reverse-Z depth `0.0` (`GreaterEqual`, no depth write, `ALPHA_BLENDING`), 2-octave 2D value noise with 3-band flat-shaded low-poly bevel, zero loops, no textures.
  - `data/sky.json`: cloud scale/softness/alpha/wind speed, sunlit/shadow/overcast colours, per-weather (`clear`, `rain`, `snow`, `sandstorm`, `fog`) cover/greyness/speed multipliers, `fog_tint_blend`, and per-biome `biome_fog_tint` (`plains`, `forest`, `desert`, `snow`, `swamp`, `mountains`).
  - `rotland/src/sky_play.rs`: `SkyData` (loaded via `paths::repo_path("data/sky.json")`) and `SkyPlay` (`tick`, `apply_biome_fog`, `params`, `draw`). `apply_biome_fog` blends `lighting.fog_color` toward `sky.json` per-biome tint using `generator.biome_sample` (`primary`/`secondary`/`blend`/`outer`), leaving home land core (`outer <= 0.0`) untouched and capping luminance so night fog never brightens.
- **Shared-file lines for the lead:**
  1. `engine/src/render/mod.rs`:
     - `pub mod sky;`
  2. `rotland/src/main.rs`:
     - `mod sky_play;`
  3. `rotland/src/app/state.rs` (in `PlaySide` + `Default` impl):
     - `pub sky: Option<crate::sky_play::SkyPlay>,` (init `sky: None,`)
  4. `rotland/src/app/start_world.rs` (right after `play.weather = Some(weather);`):
     - `match crate::sky_play::SkyData::load() { Ok(sky_data) => play.sky = Some(crate::sky_play::SkyPlay::new(gpu, sky_data)), Err(e) => log::error!("sky data: {e}"), }`
  5. `rotland/src/app/frame_warmth.rs` (in `update_weather_and_warmth`, after `weather.tick(...)`):
     - `if let Some(sky) = play.sky.as_mut() { sky.tick(dt, play.weather.as_ref().map(|w| &w.state)); }`
  6. `rotland/src/app/render_frame.rs`:
     - After `let mut lighting = ...` (around line 210):
       `if let (Some(sky), Some(generator)) = (play.sky.as_ref(), world.world_gen.as_ref()) { sky.apply_biome_fog(&mut lighting, actors.player, generator); }`
     - Right after `renderer.draw(...)` and before `crate::water_view::draw(...)` (around line 499):
       `if let Some(sky) = play.sky.as_ref() { sky.draw(gpu, target, encoder, &render.camera, &lighting, play.weather.as_ref().map(|w| &w.state)); }`
- **How to see it:**
  - Look up at the sky in camp/forest (scattered bright low-poly clouds drifting with wind) vs `--weather=rain` or `--weather=snow` (dense greyer clouds).
  - Visit outer biomes (`--biome=desert`, `--biome=snow`, `--biome=swamp`, `--biome=mountains`) to see the fog tint shift while camp/core fog stays unchanged.
- **Notes / not run:** Per subagent rules, `cargo` / `tools/gate.sh` / game were not run.
