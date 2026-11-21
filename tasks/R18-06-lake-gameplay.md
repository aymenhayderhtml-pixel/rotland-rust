# R18-06: lake gameplay: wading, swimming, fishing, water to drink

Read first: `tasks/R18-common.md`, `docs/DECISIONS.md` (2026-11-21: shallow water slows the player; deep water = slow swimming, no weapons, stamina drains), `world/src/lake/` (`zone_at`, `depth_at`, `water_level`, `fishing_spots`), `game/src/player.rs` (movement), `game/src/survival/` (stamina, thirst), `rotland/src/app/frame_sim.rs`, `data/items.json` (`dirty_water` exists but can't be obtained: GitHub #17), `data/recipes.json`, `data/survival.json`.
Files: game/src/player.rs, game/src/swimming.rs, game/src/fishing/*, game/src/lib.rs, game/tests/all/swimming_tests.rs, game/tests/all/fishing_tests.rs, game/tests/all/main.rs, rotland/src/app/frame_sim.rs, rotland/src/fishing_view.rs, data/swimming.json, data/fishing.json, data/items.json, data/recipes.json, data/loot_tables.json, assets/icons/*, tasks/R18-06-lake-gameplay.md
Test: swim

1. Water movement (pure in `game/src/swimming.rs`, numbers in `data/swimming.json`): zone from the lake → speed multiplier (shallow 0.6, deep swim 0.45), no sprint/jump in deep water, stamina drain while swimming, when stamina is 0 in deep water the player takes drowning damage; the player floats with the head at the surface in deep water (y = water level - chest height); no weapons in deep water (hands lowered). Hook it into the player update in `frame_sim.rs` / `player.rs`.
2. Fishing (`game/src/fishing/`, `data/fishing.json`): new items `fishing_rod` (tool), `fish_raw` (food, low), `fish_cooked` (food, good) (icons: copy the closest existing icon as a placeholder and say so), recipe cook fish at the campfire station (check how stations are named). With the rod equipped near water (within 3 m of a water zone, facing it), hold attack to cast; after a random wait a bite comes; press at the right moment (a timing bar) to catch. Pure state machine + tests. Draw the bite bar in `rotland/src/fishing_view.rs` (egui, small, bottom center).
3. Dirty water (#17): E while standing in shallow water with an empty `water_bottle`... if there is no empty bottle item, simply: E in shallow water gives 1 `dirty_water` (max once per 2 s); a recipe boils `dirty_water` into `water_bottle` at the campfire. Add `fishing_rod` to a loot table or two (fish_shack uses `house` today: fine to add it there).
4. Tests: zones -> multipliers; drowning; fishing timing window; items exist; recipes reference real items.

## Worker notes

