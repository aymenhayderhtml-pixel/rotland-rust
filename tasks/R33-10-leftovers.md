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
