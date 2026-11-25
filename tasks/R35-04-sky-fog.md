# R35-04: clouds and biome fog

Owns: engine/src/render/sky.rs + sky.wgsl (new, or inside lit.wgsl's sky code if that is where the sky is drawn), data/sky.json (new), rotland/src/sky_play.rs (new)
Files: engine/src/render/*, data/sky.json, rotland/src/sky_play.rs
Read only: engine/src/render/lit.wgsl and lit.rs (find how the sky/background and fog are drawn), world/src/weather (cloud cover per weather), rotland/src/weather_play.rs.

- Clouds: cheap, low-poly look: soft flat-shaded cloud shapes from 2D noise on a sky dome/plane, drifting with wind; cover follows weather (clear → few, rain/snow → many and greyer). One draw call, no textures to load.
- Fog: lerp the fog colour toward a per-biome tint (desert warm, snow blue-white, swamp green-grey, mountains cool grey) using the player's biome blend; keep the home land fog exactly as today when the player is in the core.
- Engine change must not cost more than ~0.5 ms (no per-pixel loops).
- Report the call sites for the lead.
