# R32 round: WEATHER + TEMPERATURE + BIOME ZOMBIES ("0.3 World" round 2). Lead file for the gate

Files: data/weather.json, data/temperature.json, data/zombies.json, data/special_zombies.json, data/spawning.json, data/items.json, data/recipes.json, data/loot_tables.json, data/hints.json, data/survival.json, world/*, game_*/*, game/*, rotland/*, rotland_ui/*, engine/src/render/*, tools/blender/characters/*, assets/models/characters/*, tools/blender/props/*, assets/models/props/*, tools/blender/icons/*, assets/icons/*, tools/README_TOOLS.md, docs/WORLD.md, tasks/R32-*
Rules: `tasks/LEAD.md`. Written by Neon (one chat per task), applied and gated by a Haiku helper, checked by Claude. R31 (biomes) is merged: read `docs/WORLD.md` for how biomes work.

## Owner decisions (2026-11-25)
- Weather per biome: snow falls in the snow and on the mountains, sandstorms in the desert, fog and rain in the swamp, and sometimes rain in the home lands.
- **Light temperature:** a body warmth bar. Cold in the snow and mountains, heat in the desert at day. It drains slowly and never kills quickly. A campfire, the camp, being indoors and warm clothes help. Heat makes you thirsty faster.
- One new zombie type per new biome: **frozen** (snow/mountains), **husk** (desert), **bloater** (swamp).
- No sounds (the owner adds them). No new music.
- FPS: the owner accepts 30-60 FPS for more detail, but THIS round keeps today's FPS (weather must cost < 2 ms). The big detail push is R35.

## R32 shared names (every task uses EXACTLY these)
- **Weather kinds:** `world::weather::WeatherKind { Clear, Rain, Snow, Sandstorm, Fog }`, with `key()` "clear"/"rain"/"snow"/"sandstorm"/"fog" and `from_key()`.
- **Weather state:** `world::weather::WeatherState`. It has `pub kind: WeatherKind`, `pub intensity: f32` (0..1, eased in/out over `fade_seconds`) and `pub wind: glam::Vec2` (m/s, xz).
  - `WeatherState::new(seed: i32)`.
  - `fn update(&mut self, dt_game_seconds: f32, biome: BiomeId, data: &WeatherData)`.
  - `fn fog_mult(&self) -> f32` (1 = normal, <1 = fog starts nearer).
  - `fn visibility(&self) -> f32` (0..1, used by zombie senses and the player's view).
  - `fn noise_mask(&self) -> f32` (0..1: how much rain/wind hides player noise).
- **`data/weather.json`:**
  ```
  { "fade_seconds": 40.0, "min_spell_seconds": 240.0, "max_spell_seconds": 720.0,
    "biomes": { "<biome key>": [ {"kind":"clear","weight":..}, {"kind":"snow","weight":..}, ... ] },
    "kinds": { "<kind key>": {"fog_mult":..,"visibility":..,"noise_mask":..,"wind":..,
               "particles_per_m3":..,"tint":[r,g,b]} } }
  ```
- **Temperature:** `game_player::survival::temperature::{BodyTemp, TemperatureData, WarmthSources}`.
  - `BodyTemp::new(data: &TemperatureData)`.
  - `fn update(&mut self, dt: f32, ambient_c: f32, sources: &WarmthSources, data: &TemperatureData)`.
  - Read-outs: `fn value(&self) -> f32` (0 = freezing, 50 = comfy, 100 = overheated), `fn state(&self) -> TempState { Freezing, Cold, Comfy, Hot, Overheated }`, `fn thirst_mult(&self) -> f32`, `fn stamina_regen_mult(&self) -> f32` and `fn health_drain_per_sec(&self) -> f32`.
  - `WarmthSources { pub near_fire: bool, pub in_camp: bool, pub indoors: bool, pub insulation: f32, pub sun_protection: f32 }`.
- **`world::biome` ambient temperature:** `world::ambient_temperature(registry: &BiomeRegistry, sample: &BiomeSample, hour: f32, weather: &WeatherState) -> f32`. It blends `temperature_day`/`temperature_night` by the sun (warmest 14:00, coldest 04:00), primary/secondary by `blend`, and the home biome by `outer`. Rain/snow -3 °C, sandstorm +2 °C (numbers in `data/temperature.json`).
- **New items** (`data/items.json`):
  - `warm_jacket`: insulation 0.5, slot body.
  - `wool_hat`: insulation 0.2, slot head.
  - `desert_scarf`: sun_protection 0.5, slot head.
  - `hand_warmer`: consumable, gives 60 s of "near fire" warmth.
  Use the slot / equipment system that exists today, and read `items.json` to see the fields. If there is no equipment slot system, use "carried in the bag gives the effect", and say so.
- **New zombie kinds** (`data/zombies.json` `types`, the same kind string everywhere): `frozen`, `husk`, `bloater`. Models: `assets/models/characters/zombie_frozen.glb`, `zombie_husk.glb`, `zombie_bloater.glb`. Specials data in `data/special_zombies.json` keys `frozen`, `husk` and `bloater`.
  - **frozen:** slow (×0.75 speed), tough (×1.6 health). Its bite "chills": the player's body warmth drops by 8.
  - **husk:** fast (×1.3 speed), weak (×0.7 health). It lunges like the crawler.
  - **bloater:** slow (×0.7), ×1.4 health. When it dies (or is hit by fire), it bursts into a toxic cloud: 4 m radius, 6 s, 4 damage/s to the player and 2 to other zombies.
- **Biome spawn mix:** `data/spawning.json` gets `biomeSpecials: { "<biome key>": [ {"kind":"frozen","share":0.35}, ... ] }`. This is the share of spawns that become that kind in that biome, on top of today's specials.
- **Engine weather render:** `engine::render::weather::{WeatherRenderer, WeatherParams}`. `WeatherParams { kind_index: u32 /*0 clear..4 fog*/, intensity: f32, wind: [f32; 2], tint: [f32; 3], camera_pos: [f32; 3], time: f32 }`, `WeatherRenderer::new(gpu, format, depth_format)` and `fn draw(&mut self, pass, queue, camera_uniform_or_matrices, &WeatherParams)`. Read `engine/src/render/water.rs` and copy how it is built and called.

## Tasks (one Neon chat each)
| Task | What |
|---|---|
| `R32-01-weather-core.md` | weather logic + data (world crate) |
| `R32-02-weather-render.md` | falling rain/snow/sand particles (engine) |
| `R32-03-weather-wiring.md` | weather in the game loop: fog/sky tint per biome + weather, zombie senses, save |
| `R32-04-temperature-core.md` | body warmth logic + data + ambient temperature |
| `R32-05-temperature-play.md` | warmth in play: bar on the HUD, campfire/camp/indoors, clothes items, effects, hints |
| `R32-06-biome-zombies-core.md` | frozen/husk/bloater behaviour + data + spawn mix |
| `R32-07-biome-zombies-play.md` | spawn them in play, toxic cloud effect, chill on bite, names |
| `R32-08-zombie-models.md` | the 3 zombie models (Blender) |
| `R32-09-clothes-icons.md` | the 4 item icons + a small toxic cloud puff prop (Blender) |

## After the gate passes (Haiku helper checks)
1. Blender: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/characters/build_characters.py > /tmp/r32-chars.log 2>&1` (0 Traceback, 17+ CHARACTER lines). Then the icons and props build that R32-09 names. Bake.
2. Screenshots (`target/play/rotland --save-dir=/tmp/r32-saves --shot=/tmp/r32/<n>.png`), using the `--weather=<kind>` flag from R32-03:
   - `--biome=snow --weather=snow`
   - `--biome=desert --weather=sandstorm`
   - `--biome=swamp --weather=fog`
   - camp `--weather=rain`
   - `--biome=snow --pose`-style shots of each new zombie (`--spawn-zombie=frozen` from R32-07)
   - the HUD with the warmth bar in the snow at night (`--hour=2`).
3. `tools/perf_scenes.sh --set all --seconds 20`: every scene within 2 ms of the R31 numbers in `docs/STATUS.md`.
4. Lead notes: done or not per task, the last 5 gate lines, FPS, screenshot paths, open problems.

## Lead notes
