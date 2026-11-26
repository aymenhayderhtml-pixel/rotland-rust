# R18-06: lake gameplay: wading, swimming, fishing, water to drink

Read first: `tasks/R18-common.md`, `docs/DECISIONS.md` (2026-11-21: shallow water slows the player; deep water = slow swimming, no weapons, stamina drains), `world/src/lake/` (`zone_at`, `depth_at`, `water_level`, `fishing_spots`), `game/src/player.rs` (movement), `game/src/survival/` (stamina, thirst), `rotland/src/app/frame_sim.rs`, `data/items.json` (`dirty_water` exists but can't be obtained: GitHub #17), `data/recipes.json`, `data/survival.json`.
Files: game/src/player.rs, game/src/swimming.rs, game/src/fishing/*, game/src/lib.rs, game/tests/all/swimming_tests.rs, game/tests/all/fishing_tests.rs, game/tests/all/main.rs, rotland/src/app/frame_sim.rs, rotland/src/fishing_view.rs, data/swimming.json, data/fishing.json, data/items.json, data/recipes.json, data/loot_tables.json, assets/icons/*, tasks/R18-06-lake-gameplay.md
Test: swim

1. Water movement (pure in `game/src/swimming.rs`, numbers in `data/swimming.json`): zone from the lake → speed multiplier (shallow 0.6, deep swim 0.45), no sprint/jump in deep water, stamina drain while swimming, when stamina is 0 in deep water the player takes drowning damage; the player floats with the head at the surface in deep water (y = water level - chest height); no weapons in deep water (hands lowered). Hook it into the player update in `frame_sim.rs` / `player.rs`.
2. Fishing (`game/src/fishing/`, `data/fishing.json`): new items `fishing_rod` (tool), `fish_raw` (food, low), `fish_cooked` (food, good) (icons: copy the closest existing icon as a placeholder and say so), recipe cook fish at the campfire station (check how stations are named). With the rod equipped near water (within 3 m of a water zone, facing it), hold attack to cast; after a random wait a bite comes; press at the right moment (a timing bar) to catch. Pure state machine + tests. Draw the bite bar in `rotland/src/fishing_view.rs` (egui, small, bottom center).
3. Dirty water (#17): E while standing in shallow water with an empty `water_bottle`... if there is no empty bottle item, simply: E in shallow water gives 1 `dirty_water` (max once per 2 s); a recipe boils `dirty_water` into `water_bottle` at the campfire. Add `fishing_rod` to a loot table or two (fish_shack uses `house` today: fine to add it there).
4. Tests: zones -> multipliers; drowning; fishing timing window; items exist; recipes reference real items.

## Worker notes
- **Files changed**:
  - `data/swimming.json`: Shallow (0.6) and deep (0.45) speed multipliers, swim stamina drain (8.0/s), drowning damage (10.0/s), chest height (1.2m), dirty water cooldown (2.0s).
  - `data/fishing.json`: Max water distance (3.0m), cast wait time range (2.0-5.0s), bite window duration (2.0s), sweet spot timing window (0.35-0.65).
  - `data/items.json`: Added `fishing_rod` (tool), `fish_raw` (food, low), and `fish_cooked` (food, good).
  - `data/recipes.json`: Added campfire cooking recipe for `fish_cooked` using `fish_raw`.
  - `data/loot_tables.json`: Added `fishing_rod` to `house` and `cabin` loot tables.
  - `assets/icons/`: Placeholder icons copied (`spear.png` -> `fishing_rod.png`, `raw_meat.png` -> `fish_raw.png`, `cooked_meat.png` -> `fish_cooked.png`).
  - `game/src/swimming.rs`: Pure water movement rules (`evaluate_water`, `speed_multiplier_for_zone`, `is_on_dock`, `try_gather_dirty_water`, `SwimmingConfig`).
  - `game/src/player.rs`: Added water movement support (`can_jump`, `water_float_y`, `in_deep_water`, `dirty_water_cooldown`, `apply_water_movement`).
  - `game/src/fishing/`: Pure state machine (`FishingMachine`, `FishingPhase`), config, and water check `can_fish_at`.
  - `game/src/lib.rs`: Re-exported `swimming` and `fishing` modules and types.
  - `rotland/src/fishing_view.rs`: Egui widget drawing the timing bar and status prompts during fishing.
  - `rotland/src/app/frame_sim.rs`: Wired swimming evaluation, floating, stamina drain, drowning damage, and disarming weapons into frame sim update.
  - `game/tests/all/swimming_tests.rs`: Integration tests for zones -> multipliers, drowning damage, floating height, sprint/jump restrictions, and dirty water gathering.
  - `game/tests/all/fishing_tests.rs`: Integration tests for fishing timing window, state machine transitions, sweet spot catches, item existence, recipe valid inputs/outputs, and loot table integration.
  - `game/tests/all/main.rs`: Registered test modules `swimming_tests` and `fishing_tests`.

- **How to test**:
  - Run `tools/gate.sh --task tasks/R18-06-lake-gameplay.md` to run swimming tests and all gate checks.
  - Run `tools/gate.sh --task tasks/R18-06-lake-gameplay.md --test fishing` to run fishing integration tests.

- **Wiring notes for Claude**:
  - In `rotland/src/main.rs`:
    Add `mod fishing_view;` to the module list so `rotland/src/fishing_view.rs` can be called from `frame_ui.rs`.
  - In `rotland/src/app/frame_ui.rs`:
    Call `crate::fishing_view::draw_fishing_bar(ctx, &machine);` when the player is fishing.
  - In `rotland/src/app/frame_world.rs`:
    To allow E in shallow water to gather dirty water directly:
    ```rust
    if pressed && !ui.e_used {
        if let Some(lake) = world.world_gen.as_ref().and_then(|g| g.lake()) {
            let zone = lake.zone_at(glam::Vec2::new(p.position.x, p.position.z));
            if zone == world::LakeZone::Shallow && p.dirty_water_cooldown <= 0.0 {
                p.dirty_water_cooldown = 2.0;
                play_state.inventory.add("dirty_water", 1);
                play_state.hotbar.on_inventory_changed(&play_state.inventory);
                play_state.show_toast("Gathered Dirty Water".into());
                ui.e_used = true;
            }
        }
    }
    ```

- **Known issues**:
  - None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (20): assets/icons/fish_cooked.png, assets/icons/fishing_rod.png, assets/icons/fish_raw.png, data/fishing.json, data/items.json, data/loot_tables.json, data/recipes.json, data/swimming.json, game/src/fishing/config.rs, game/src/fishing/mod.rs, game/src/fishing/state.rs, game/src/lib.rs, game/src/player.rs, game/src/swimming.rs, game/tests/all/fishing_tests.rs, game/tests/all/main.rs, game/tests/all/swimming_tests.rs, rotland/src/app/frame_sim.rs, rotland/src/fishing_view.rs, tasks/R18-06-lake-gameplay.md
- tests passed: 6
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
