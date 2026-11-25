# R32-05: warmth in play (HUD bar, campfire/camp/indoors, clothes, effects, hints)

Owns: rotland/src/temperature_play.rs (new), rotland_ui/src/hud/warmth.rs (new), data/items.json (only the 4 new items), data/recipes.json (recipes for them), data/loot_tables.json (only adding the new items to the snow/mountains/desert tables), data/hints.json (2 hints), game/tests/all/warmth_items_tests.rs (new)
Files: rotland/src/temperature_play.rs, rotland_ui/src/hud/warmth.rs, data/items.json, data/recipes.json, data/loot_tables.json, data/hints.json, game/tests/all/warmth_items_tests.rs
Read: tasks/R32-round.md, tasks/LEAD.md, rotland_ui/src/hud/vitals.rs, rotland_ui/src/hud/mod.rs, game_player/src/survival/need.rs, game/tests/all/items_tests.rs
API: rotland/src/camp.rs, rotland/src/campfire_mesh.rs, rotland/src/app/state.rs, rotland_ui/src/lib.rs, game_player/src/lib.rs
Data: data/items.json, data/recipes.json, data/loot_tables.json, data/hints.json
Overlap: `game/tests/all/main.rs` is owned by R32-04. Do not write it: report the `mod warmth_items_tests;` line to the lead instead.
Rules: `tasks/LEAD.md` subagent rules. Use the "R32 shared names" in `tasks/R32-round.md` EXACTLY. R32-04 makes `BodyTemp` and `ambient_temperature` at the same time: code against the shared names.

1. **`rotland/src/temperature_play.rs`:** `TemperaturePlay` holds a `BodyTemp` and `TemperatureData`. `fn tick(&mut self, dt, player_pos, hour, generator, weather: &WeatherState, ...) ` builds `WarmthSources`:
   - `near_fire`: within 4 m of a lit campfire (find how campfires are stored: grep `campfire`), or a hand warmer still running;
   - `in_camp`: the existing safe-zone check (grep `in_camp` / `safe_zone`);
   - `indoors`: inside a POI or building footprint, if a cheap check already exists (grep `inside` / `indoor` / `roof`). Otherwise false, and say so;
   - `insulation` / `sun_protection`: from the equipped (or carried) items' data fields.
   It then applies the effects to the existing survival code:
   - the thirst rate multiplier (`rate_multiplier` in `game_player/src/survival/need.rs`, which nothing sets yet: use it);
   - the stamina regen multiplier (find the stamina code);
   - health drain through the normal damage path (the cause name `"cold"` / `"heat"`).
   Report the shared-file lines for the lead: the state field, the tick call, the HUD hook and the save field.
2. **HUD** (`rotland_ui/src/hud/warmth.rs`): a small bar under Food/Water (look at `vitals.rs` and copy its style), going blue → green → orange, labelled "Warmth". Show it only when the state is not Comfy or the player is outside the core (`tier > 0`), so the home lands stay clean. Add a small snowflake/sun marker drawn with egui shapes (no new textures).
3. **Items** (shared names): `warm_jacket`, `wool_hat`, `desert_scarf`, `hand_warmer`. Use the existing fields and slot system: read `data/items.json` and how gear works (grep `slot`, `equip`, `gear`). Icons come from R32-09 (`assets/icons/<id>.png`; follow how other items name their icon).
   - Recipes: warm_jacket = cloth ×4 + leather ×2; wool_hat = cloth ×2; desert_scarf = cloth ×2; hand_warmer = scrap ×1 + cloth ×1. Use real item ids that exist (check `items.json`; swap to the nearest real ones and say so).
   - Loot: snow/mountains get warm_jacket, wool_hat and hand_warmer (rare/uncommon); desert gets desert_scarf.
4. **Hints** (`data/hints.json`, same format as the others):
   - "You're getting cold. A campfire, warm clothes or shelter will help." (first time Cold)
   - "The heat makes you thirsty. Drink water and cover your head." (first time Hot)
5. **Tests:** the items load with their fields; the recipes use only existing items; the loot tables reference only existing items.
6. **Save:** report the line to save/load the `BodyTemp` value (an optional field, default 50).

## Worker notes
