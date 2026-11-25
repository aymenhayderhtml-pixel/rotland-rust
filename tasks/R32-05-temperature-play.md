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

## Neon notes

### What was implemented

* **`rotland/src/temperature_play.rs`**: `TemperaturePlay` owns `BodyTemp` and `TemperatureData`, reads warmth fields (`slot`, `insulation`, `sun_protection`, `warmth_seconds`) directly from `data/items.json`, tracks the 60 s `hand_warmer` timer, builds `WarmthSources` (4 m lit campfire radius or active hand warmer, camp safe zone, indoors, and carried gear protection taking the best item per slot so duplicates do not stack), updates `NeedStat::rate_multiplier` on thirst, and returns `TemperatureTickResult` (stamina regen multiplier, per-frame health damage + `"cold"`/`"heat"` cause, and one-shot `"cold"`/`"hot"` hint triggers).
* **`rotland_ui/src/hud/warmth.rs`**: `WarmthHudInfo`, `WarmthHudState`, and `draw_warmth_bar` matching `vitals.rs` styling. Displays a blue → green → orange bar labelled `"Warmth"` with a vector snowflake or sun drawn via egui primitives (no textures), hidden when `Comfy` inside `biome_tier == 0`.
* **Layout note**: Placed at `screen.max.y - 82.0 * scale` (continuing the 14 px stride of Water at `54.0` and Food at `68.0`) because Health occupies `max.y - 38..16` directly below Water and Dog HUD sits at `max.y - 118..90`.


* **`data/items.json`**: Added `warm_jacket`, `wool_hat`, `desert_scarf`, and `hand_warmer` using existing `"category": "tool"` (icons follow existing `<id>` convention -> `assets/icons/<id>.png`).
* **No equipment slot system**: `data/items.json` has no clothing/armor slot system yet, so warmth gear applies while carried in the bag (deduplicated per slot `"body"`/`"head"`).


* **`data/recipes.json`**: Added hand-craft recipes for all 4 items.
* **Item ID swaps to real IDs in `data/items.json**`: `leather` → `hide` (`warm_jacket`), `scrap` → `scrap_metal` (`hand_warmer`).


* **`data/loot_tables.json`**: Added `warm_jacket`, `wool_hat`, and `hand_warmer` to `snow` and `mountains`, and `desert_scarf` to `desert`.
* **`data/hints.json`**: Added `"cold"` (`trigger: "temp_cold"`) and `"hot"` (`trigger: "temp_hot"`) hints with the exact required strings.
* **`game/tests/all/warmth_items_tests.rs`**: Verifies all 4 items load through `ItemRegistry` with expected warmth fields, all recipes reference existing items, all loot tables reference existing items and include the biome gear, and both warmth hints exist.

### Shared-file lines for the lead

1. **`game/tests/all/main.rs`** (owned by R32-04):
```rust
mod warmth_items_tests;

```


2. **`rotland_ui/src/hud/mod.rs`**:
```rust
pub mod warmth;
pub use warmth::{draw_warmth_bar, WarmthHudInfo, WarmthHudState};

```


3. **`rotland_ui/src/hud/state.rs`** (`HudState`) and **`rotland_ui/src/hud/draw.rs`**:
* In `HudState`: `pub warmth: Option<super::WarmthHudInfo>,` (default `None`).
* In `draw_hud` (right beside `draw_need_bars_polished`):
```rust
super::warmth::draw_warmth_bar(painter, &screen, time, state.warmth.as_ref(), scale, config);

```




4. **`rotland/src/main.rs`** (or crate root):
```rust
mod temperature_play;

```


5. **`rotland/src/app/state.rs`** (`PlaySide`):
```rust
pub temperature: Option<crate::temperature_play::TemperaturePlay>,

```


(Initialize `temperature: None` in `Default::default()`, and populate on world init using `TemperaturePlay::new(temp_data, &paths::repo_path("data/items.json"))`.)
6. **Frame tick wiring in `rotland/src/app/**`:
* Collect lit campfire positions from `camp.campfires`, sample biome via `world_gen`, query existing safe-zone check for `in_camp`, pass `indoors: false` (no cheap building/roof interior check was present in the provided APIs; wire it if one exists in the worktree), and call:
```rust
let res = temp.tick(
    &crate::temperature_play::TemperatureTickInput {
        dt,
        player_pos,
        hour: self.play.hour,
        biome_registry,
        biome_sample: &sample,
        weather,
        lit_campfires: &lit_campfires_scratch,
        in_camp,
        indoors: false,
        inventory: &play.inventory,
    },
    Some(&mut play.thirst),
);

```


* Apply `res.stamina_regen_mult` to player stamina regen (the stamina struct was not in the prompt's API signatures).
* If `res.health_damage > 0.0`, apply damage through the normal player damage path with `res.damage_cause.unwrap_or("cold")`.
* When the player uses `"hand_warmer"` from bag/hotbar, call `temp.activate_hand_warmer(&mut play.inventory)`.
* Set `hud_state.warmth = Some(temp.hud_info(sample.tier))` (use whatever tier field/getter `BiomeSample` / `BiomeRegistry` exposes).
* Wire `"temp_cold"` / `"temp_hot"` into `HintContext` (or push the `"cold"` / `"hot"` hint when `res.triggered_hint` is `Some(id)`).


7. **Save / Load (optional field, default 50.0)**:
* On the player/world save struct:
```rust
#[serde(default = "default_body_temp")]
pub body_temp: f32,

```


```rust
fn default_body_temp() -> f32 {
    50.0 // new: DECISIONS 2026-11-25
}

```


* On save: `body_temp: self.play.temperature.as_ref().map_or(50.0, |t| t.value()),`
* On load: pass `save.body_temp` into `BodyTemp`'s setter/load method from R32-04 (not listed in the shared names section of `tasks/R32-round.md`, so left for the lead to match R32-04's exact method name).



### Missing APIs / known risks to check during gate

* **Missing APIs in this task pack**: `game::CampfireManager` methods, `WorldGenerator` biome-sampling signature, `BiomeRegistry` re-export path (`world::biome::BiomeRegistry` vs `world::BiomeRegistry`), stamina regen multiplier field, player health damage method, `HintContext` trigger matcher, `SaveFile`, and `BodyTemp` load setter. `TemperaturePlay` uses explicit input/output structs (`TemperatureTickInput` / `TemperatureTickResult`) so nothing was invented.
* **`ItemDef` deserialization**: Verify `ItemDef` in `game` does not use `#[serde(deny_unknown_fields)]` (it already tolerates extra fields in `data/items.json` such as `shortName`/`holdable`, so `slot`, `insulation`, `sun_protection`, `consumable`, and `warmth_seconds` should deserialize cleanly).
* **`rotland/Cargo.toml`**: Ensure `game_player` is listed as a direct dependency of `rotland` (if `rotland` currently accesses it through `game`, swap `use game_player::` to the workspace re-export path).
