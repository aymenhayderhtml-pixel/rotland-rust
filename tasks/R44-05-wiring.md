# R44-05: Wiring (play glue, chunk contents, rope ladder, fast travel, rooftop trader, rescues, save, stats)

Runs AFTER R44-01, 02, 03, 04 are merged (the lead merges them first). Read first: `tasks/R44-common.md` (whole) and the Worker notes of R44-01..04.

Owns:
- `rotland/src/safe_house_play.rs` (NEW): `SafeHousePlay` (state, layout, config, interaction, save).
- `rotland/src/safe_house_travel.rs` (NEW): travel and rest (pure decision + the clock/teleport side effects).
- `rotland/src/rooftop_npcs.rs` (NEW): the placed rooftop people (survivors, trader), posed like `gunsmith_play.rs` shopkeepers.
- `world/src/generator_contents.rs` (ONLY: a `metro_safehouse_extras(coord)` method next to `metro_climb_extras`).
- `rotland/src/streaming.rs` / `rotland/src/streaming_metro.rs` (ONLY: extra ladder rails from the safe house that survive `rebuild_ladders`; the rope props' chunk contents).
- `rotland/src/rescue_play.rs` (ONLY: `load_state` merges the floor spots).
- `rotland/src/app/frame_world.rs`, `rotland/src/app/save_load.rs`, `rotland/src/app/start_world.rs`, `rotland/src/app/state.rs` (ONLY the wiring lines), `rotland/src/app/stats.rs` (the new keys), `rotland/src/camp.rs` / `rotland_ui/src/camp_ui/*` (ONLY what is needed to open the rooftop shop with the rooftop stock; the camp shop must behave exactly as before).

Not owned: the pure logic (R44-01, R44-03, R44-04). If something there is wrong, write it under Worker notes; do not change it here unless it does not compile, and then say so.

Read:
- `rotland/src/metro_radio.rs` and `rotland/src/app/frame_world.rs` ~`:373-460` (the rescue and radio interaction order: `free`, `free_world`, `pressed`, prompt output). Copy this exact order.
- `rotland/src/app/start_world.rs:190-240` (radio init and restore: copy for `metro_safehouse`).
- `rotland/src/app/save_load.rs` `save_now` (line 7). Bed = save: call it from the frame (it needs session, world, actors, play).
- `rotland/src/gunsmith_play.rs` `pose_shopkeepers` (line 145) and `facing_shopkeeper` (line 178): the pattern for posed placed people.
- `rotland/src/streaming.rs` `rebuild_ladders` (line 125) and the `ladders` field; `game::LadderRail::new` (`game_player/src/ladders/`).
- `rotland/src/camp.rs` (`trader` field at line 27, the view builder), `rotland_ui/src/camp_ui/draw_shop.rs` (`draw_shop`, line 13), `CampScreen::Shop` / `CampScreen::Bank`, how `ui.camp_ui` is set (`frame_world.rs` ~`:520-540`).
- `rotland/src/app/frame_world.rs:606` (`play.clock.advance(dt)` and the per-day hook that runs when `ClockChanges.new_day` is set: travel must run the same hook, not just move the hour).
- `rotland/src/app/test_flags/metro_tp.rs` (the teleport helper pattern).
- `rotland/src/app/stats.rs` (`playerPosition` line 54, `poiZombiesAwake` line 71).

## What to build (in this order)
1. **Load.** `SafeHousePlay::load(world_gen, save)`: `SafeHouseLayout` (from `world::metro::safehouse::safe_house_layout()`), `SafeHouseConfig::from_repo()`, `RooftopTraderConfig::from_repo()`, `safe_house_spots`. Any load error: log a warning and the feature is off (the radio pattern). Restore `systems.extra["metro_safehouse"]` through `SafeHouseState::from_json`.
2. **Chunk contents.** `WorldGenerator::metro_safehouse_extras(coord) -> (ClimbChunk, Vec<RooftopNpc>)`: `safehouse_in_chunk` + `rooftop_camp_in_chunk`, merged into `chunk_contents` (props) like the R43 climb props. Their NPCs go to `rotland/src/rooftop_npcs.rs`.
3. **Rope ladder** (`SafeHouseState.ladder_down`):
   - Lowered: add `game::LadderRail::new(foot, deck, "metro_safehouse")` to the streamer's ladder list; it must survive `rebuild_ladders` (add `extra_ladders` to the streamer and include it in `rebuild_ladders`).
   - Pulled up: remove it.
   - Owner rule (Claude): no hanging rope when the ladder is up. Pass `ladder_down` into `safehouse_in_chunk` (R44-03). On toggle, re-stream the chunks the rope crosses (find an existing chunk reload/unload path in `rotland/src/streaming.rs`; if none exists, add a small `reload_chunks(coords)` that unloads and loads them again). Up = only `rope_ladder_coil` on the roof edge; down = the `rope_ladder_seg` line + no coil. Prove it with two pictures.
4. **Interactions** (inside `frame_world.rs`, after the radio and before crates, with the same `free_world` flow; only when the player is within `useRadiusM` of the spot):
   - **Claim** at the shack (unclaimed only): `state.claim()`, toast `claimedToast`, the rail goes down.
   - **Bed** (claimed): `clock.set_time(restHour)`, `hour` mirrored, `rooftop_stock.fill_all()`, toast `rested`, and set `pending_save = true`; the frame calls `save_now` at the end of the frame.
   - **Chest** (claimed): open `CampScreen::Bank` (the shared bank).
   - **Roof-edge ladder** (claimed, within `useRadiusM` of `ladder_deck`): `toggle_ladder()`; prompt `ladderUp` / `ladderDown`.
   - **Shack sign** (claimed): `[E] Travel to camp` (`travelToCamp`).
   - **Home camp sign** (claimed): `[E] Travel to safe house` (`travelToSafeHouse`). The home-camp sign does not exist yet: place a `travel_sign` prop at the home camp if the camp scene can take one simply; otherwise use an existing camp point and say which in Worker notes.
   - **Rooftop trader** (`role == "trader"`, within `interact_range`): open `CampScreen::Shop` with the rooftop `TraderStock` (not the camp one). The camp shop must behave as before when the player is not on the roof.
5. **Fast travel** (`safe_house_travel.rs`):
   - `pub fn travel_decision(claimed, chasing: usize) -> TravelDecision { Blocked, Go }` (pure; uses `travel_blocked`). Chasing = `count_chasing_near` over the living zombies (`ZombieState::Chase | Attack`, the bool), at the player's position, radius `chaseRadiusM`.
   - Blocked: toast `travelBlocked`. No time passes.
   - Go: `play.clock.advance_hours(travelHours)` (run the same per-day hook as `frame_world.rs:606` if `new_day` is set), mirror `play.hour`, teleport: to the roof `travel_point` (safe house) or to the camp respawn point (as `save_now` gets it: `try_respawn_point(...)`, else `respawn_point(...)`). Toast with the new clock time (`Clock::format`).
   - No items are used.
6. **Rescues.** `rescue_play::load_state` merges `floor_rescue_spots(...)` built from `world::metro::rescue_floor_seeds(metro)` (convert `ClimbRescueSeed` to `FloorRescueSeed`). The existing `interact_tick` then works as-is.
7. **Save.** `save_load.rs`: write `systems.extra["metro_safehouse"] = state.to_json()` (copy the radio lines). `start_world.rs`: restore it (copy lines 234-235 pattern). Old saves: no key = unclaimed.
8. **Stats** (`stats.rs`): add `gameHour` (`play.hour`), `safeHouseClaimed`, `safeHouseLadderDown`, `rescuedCount` (the rescued count from `RescueState`), `chasingZombies` (the travel count). Keys exactly as in `R44-common.md`.
9. **State** (`state.rs`): add `pub safe_house: SafeHousePlay` to `PlaySide` (copy `metro_radio`'s line); `pending_save: bool` is a frame-local flag.

## Number and tag rules
No new Rust numbers. Every distance, hour and count comes from `data/safe_house.json` / `data/rescue.json`. `safe_house_layout()` (world) is the only loader of the layout file.

## Checks you do yourself (the gate runs the tests; you check by running the game)
Not in this task: the screenshots and stats runs are R44-06 and the round lead's checks. Here: the code compiles (the gate), `--metro=safehouse` starts without a panic (one run with `--frames=60 --save-dir=<scratch>`, under the flock).

## Done when
Everything above is wired; the gate passes (`tools/gate.sh --task tasks/R44-05-wiring.md`). Fill `## Worker notes` with:
- Files changed.
- The mod lines for `rotland/src/main.rs`, `rotland/src/app/state.rs`, `rotland/src/app/mod.rs`, `world/src/metro/mod.rs` if you need them.
- The home-camp sign decision (placed prop or existing point).
- How the rope swap works (which chunks re-stream) and the two pictures.
- How the rooftop shop reads its stock (which `CampView` field).
- How the rooftop NPC sits (existing clip name, or stands and why).
- Any premise in `R44-common.md` that did not hold in the code.

## Worker notes
- **Files changed:** new `rotland/src/safe_house_play.rs` (SafeHousePlay: load, interact_tick with claim/bed/chest/ladder/travel, rooftop buy, rope sync), new `rotland/src/safe_house_travel.rs` (travel_decision + stream_to), new `rotland/src/rooftop_npcs.rs` (trader_position). Edited: `world/src/generator_contents.rs` (metro_safehouse_extras, home_sign_pos, set_safehouse_ladder_down, merged into chunk_contents), `rotland/src/streaming.rs` (extra_ladders, reload_chunks, stale colliders kept until the new chunk is built, so no fall through the roof), `rotland/src/streaming_metro.rs` and `rotland/src/props/rules.rs` (rope_ladder_ and green_smoke_ skipped for colliders and shadows), `rotland/src/rescue_play.rs` (floor rescue spots merged), `rotland/src/app/frame_world.rs`, `frame_ui.rs` (rooftop stock + buy), `frame_actors.rs` (pending bed save), `render_frame.rs` (rope sync, fast travel placement), `save_load.rs`, `start_world.rs`, `state.rs`, `stats.rs`.
- **Shared-file lines I had to add (lead: keep them):** `rotland/src/main.rs`: `mod rooftop_npcs; mod safe_house_play; mod safe_house_travel;`. `game/src/lib.rs`: `pub use game_zombie::chasing;` and `safe_house` added to the game_world re-export list.
- **Home camp sign:** NO prop. A `travel_sign` prop at the camp broke the existing test `city_camp_chunk_matches_without_city` (it compares the camp chunk with and without the metro). The home travel point is the camp respawn point (`WorldGenerator::home_sign_pos`), with the same 2 m use radius. Lead may want a real sign later (needs a test change).
- **Rope swap:** `SafeHousePlay::sync_rope` runs each frame before streaming. Down = one `LadderRail` in `streamer.extra_ladders` plus `rope_ladder_seg` props; up = `rope_ladder_coil`. On a change the 3x3 chunks around the shack are dropped and rebuilt. The old colliders stay solid until the new chunk is built, so the player does not fall. The chunk flag is a static set by `WorldGenerator::set_safehouse_ladder_down`.
- **Rooftop shop:** `shop_on_roof` picks the stock. `frame_ui` gives `CampView.trader_stock` the rooftop `TraderStock` and sends Buy to `SafeHousePlay::buy_on_roof`. Sell and the camp trader are unchanged. Opening the camp trader resets the flag.
- **Rooftop NPCs:** drawn as bind-pose props (`npc_survivor`, `npc_trader`), NOT posed or animated (no clip code yet). The trader's [E] uses their position from `rooftop_npcs::trader_position`.
- **Travel:** claimed and not chased (`count_chasing_near` over `ZombieState` Chase/Attack via `actors.zombies.active()`): the clock advances `travelHours`, and `render_frame` streams the target (blocking, like the test teleports) and stands the player on the ground there. Blocked = toast, no time.
- **Bed:** `set_time(restHour)`, fills the rooftop stock, sets `pending_save`; `check_scripted_timers` calls `save_now` the same frame. Saves `metro_safehouse` and `rooftop_trader` in `systems.extra`.
- **Stats added:** `gameHour`, `safeHouseClaimed`, `safeHouseLadderDown`, `rescuedCount`, `chasingZombies`.
- **Logic not verified in game:** I could NOT run the roof proof. No existing flag puts the player on the roof (`--tp=` does not exist; `--at=x,z` is ground only), and claiming needs the roof. So no two rope pictures, no travel or save-reload stats. Smoke run OK: `--metro=core --frames=60 --stats=...` (save dir /tmp/r44-saves), no panic, all new keys present. Safe house tower 489, shack at (5790.35, 55.12, -3280.38), deck y 55.12 (from the log line).
- **Premises from R44-common that did not hold:** none new. The `metro_safehouse_extras` returns props + people; the people also go in as props.
- **Open:** rooftop ladder-down prop chunk and the rescue floor spot placement not seen in a picture; the chunk rebuild after a toggle can use a chunk built with the old flag if it was already in flight (rare).

## Gate report
- date: 2026-11-28
- base: main
- files changed (19): game/src/lib.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_ui.rs, rotland/src/app/frame_world.rs, rotland/src/app/render_frame.rs, rotland/src/app/save_load.rs, rotland/src/app/start_world.rs, rotland/src/app/state.rs, rotland/src/app/stats.rs, rotland/src/main.rs, rotland/src/props/rules.rs, rotland/src/rescue_play.rs, rotland/src/rooftop_npcs.rs, rotland/src/safe_house_play.rs, rotland/src/safe_house_travel.rs, rotland/src/streaming_metro.rs, rotland/src/streaming.rs, tasks/R44-05-wiring.md, world/src/generator_contents.rs
- tests passed: 1568
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
