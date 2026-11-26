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
- **Files changed (8):**
  - `rotland/src/app/state.rs`: Added `pub hud: crate::hud::Hud` to `UiState`.
  - `rotland/src/hud/mod.rs`: Added 1-arg `on_damage(from)` using `last_time`, `on_damage_at(from, now)`, and `car_out_of_fuel_notified` flag. Added `test_hud_wired_notifications` testing all wired notification types.
  - `rotland/src/app/frame_ui.rs`:
    - Updates `ui.hud.update` and `ui.hud.update_vitals` each frame with real vitals and `session.last_dt`.
    - Drains `play_state.pending_pickups` into `ui.hud.on_pickup`.
    - Drains `ui.overlays.pending_level_ups` into `ui.hud.notify`.
    - Detects car out of fuel when driven and calls `ui.hud.notify`.
    - Hooks trader purchases (`CampAction::Buy`) to `ui.hud.on_pickup`.
    - Hooks bounty turn-in (`CampAction::TurnIn`) to `ui.hud.notify("Bounty complete")`.
    - Hooks trunk withdraw (`HubAction::TrunkMove`) to `ui.hud.on_pickup`.
    - Calls `ui.hud.populate_state(&mut state)` before `draw_hud`.
  - `rotland/src/app/frame_sim.rs`:
    - Wires zombie attack damage to `ui.hud.on_damage(z_actor.position)`.
    - Wires animal attack damage to `ui.hud.on_damage(attacker.position)`.
  - `rotland/src/play.rs`:
    - Added `pub pending_pickups: Vec<(String, u32)>` to `PlayState`.
    - Buffers item pickups from crates (accounting for leftovers) and ground items in `interact_tick`.
  - `rotland/src/hub_ui/state.rs`:
    - Added `pub pending_level_ups: Vec<String>` to `HubUiState`.
    - Buffers level up messages in `trigger_level_up`.
  - `rotland/src/hub_ui/tests.rs`:
    - Added assertion for `pending_level_ups` in `test_level_up_banner`.
  - `tasks/R17-09-hud-wiring.md`: Filled worker notes.

- **How to test:**
  - Run `tools/gate.sh --task tasks/R17-09-hud-wiring.md` (passes all checks: clippy, unit tests, scope, numbers, unused).
  - Can run `cargo test hud` to run all HUD unit and math tests (22 passed).

- **Wiring notes for Claude:**
  - All HUD effects from R16-09 are now hooked into live gameplay:
    - Directional damage arcs appear when hit by zombies or animals pointing to their world positions.
    - Pickup popups stack above hotbar when picking up world items, opening loot crates, withdrawing from car trunk, or buying from the camp trader.
    - Event notifications appear under compass for completed bounties, skill level-ups, and driving out of fuel.
    - Vitals trails and pulses update dynamically with player vitals and needs each frame.
  - Quick save has no keybind wired in gameplay yet (key only exists in keybind table per `docs/JESTER.md`); `hud.notify` is ready if quick save is hooked.

- **Known issues:**
  - None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (8): rotland/src/app/frame_sim.rs, rotland/src/app/frame_ui.rs, rotland/src/app/state.rs, rotland/src/hub_ui/state.rs, rotland/src/hub_ui/tests.rs, rotland/src/hud/mod.rs, rotland/src/play.rs, tasks/R17-09-hud-wiring.md
- tests passed: 22
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
