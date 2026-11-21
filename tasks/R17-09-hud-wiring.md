# R17-09: hook the new HUD effects into the game

Read first: `tasks/R17-common.md`, `rotland/src/hud/mod.rs` (the new `Hud` API: `on_damage`, `on_pickup`, `notify`, `update`, `update_vitals`; and `## Worker notes` of `tasks/R16-09-hud-polish.md`, which lists the call sites), `rotland/src/app/frame_ui.rs` (where `draw_hud` runs), `rotland/src/app/frame_sim.rs` (search `bites`: player damage), the item pickup code (search `add_item`/`pick` under `rotland/src/`), `game/src/camp_upgrades/` and `game/src/director/` (events to notify later; only if already called somewhere).
Files: rotland/src/hud/*, rotland/src/app/frame_ui.rs, rotland/src/app/frame_sim.rs, rotland/src/app/state.rs, rotland/src/crates.rs, rotland/src/play.rs, rotland/src/trunk_play.rs, rotland/src/camp_ui/*, rotland/src/hub_ui/*, tasks/R17-09-hud-wiring.md
Test: hud

Make the R16-09 HUD features actually show in the game:
- The `Hud` state lives in the app state (`state.rs`), is updated each frame (`update`, `update_vitals` with the real health/stamina/food/water) and drawn by `draw_hud`.
- `on_damage(attacker position)` wherever the player takes a hit from a zombie or animal (the attacker's position must be known at that spot; if a hit has no source position, skip the arc).
- `on_pickup(item, count)` wherever items enter the player's bag from the world (loot crates, POI loot, ground pickups, trunk take, bought at the trader: NOT from moving items inside the bag).
- `notify(...)` for: bounty completed, skill level up, quick save done, car out of fuel (only where those events already exist in code; one line each).
- No behavior changes besides showing these. Keep each hook a one-liner where possible.
- Add a test if any pure helper is added. Claude checks it in the game with screenshots.

## Worker notes

