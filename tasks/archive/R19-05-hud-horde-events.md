# R19-05: HUD warnings for hordes, the director's waves and camp attacks

Read first: `tasks/R19-common.md`, `rotland/src/hud/mod.rs` (`notify`, `NotificationKind`), `rotland/src/hud/notification.rs`, `rotland/src/zombies/mod.rs` (`ZombieFrame`, `hordes`), `game/src/horde/*`, `game/src/director/*`, `rotland/src/play.rs` (where the director runs), `rotland/src/app/frame_sim.rs`, `data/hud.json`.
Files: rotland/src/zombies/mod.rs, rotland/src/zombies/types.rs, rotland/src/app/frame_sim.rs, rotland/src/hud/*, rotland/src/play.rs, data/hud.json, game/src/director/*, game/tests/all/director_tests.rs, tasks/R19-05-hud-horde-events.md
Test: hud

1. `ZombieFrame` gets events: `HordeFormed { size, position }`, `HordeNear { size, distance }` (first time a horde of 5+ comes within 60 m of the player; distance from `data/hud.json`), `SpecialSeen { kind }` (first sighting of a brute / screamer / crawler within 30 m), `CampAttacked` (a zombie inside the camp wall radius while the player is in the camp).
2. The director's phase changes (calm -> tension -> wave) produce an event too (from `play.rs` or `game/src/director`).
3. `frame_sim.rs` turns events into HUD notifications with short texts ("A horde is coming from the north-east", "Screamer nearby!", "The camp is under attack!", "Wave incoming"). Direction words from the angle (8 directions, pure fn + test). Each event type has a cooldown (seconds in `data/hud.json`) so the screen isn't spammed.
4. Horde warning kind uses a red/orange color; add a small screen-edge arrow toward a near horde if it fits cleanly (optional).
5. Tests: direction words, cooldown, `HordeNear` fires once per horde.

## Worker notes
- **Files changed**:
  - `data/hud.json`: Added `horde_color` (`#f97316`) and `horde_events` configuration with thresholds (near distance 60m, special seen distance 30m, horde min size 5, camp wall radius 50m) and cooldowns in seconds.
  - `rotland/src/zombies/types.rs`: Added `ZombieEvent` enum (`HordeFormed`, `HordeNear`, `SpecialSeen`, `CampAttacked`), added `events: Vec<ZombieEvent>` to `ZombieFrame`, and implemented `ZombieEventTracker` struct to track near-horde alerts (one alert per horde), first sightings of special zombies within 30m, and camp attacks within wall radius. Added unit test `test_hud_horde_near_fires_once_per_horde`.
  - `rotland/src/zombies/mod.rs`: Wired `ZombieEventTracker` into `Zombies`, storing camp center and wall radius, and populated `frame.events` in `update_full`.
  - `game/src/director/state.rs`: Added `HordeEvent::PhaseChange { from, to }`.
  - `rotland/src/play.rs`: Emitted `HordeEvent::PhaseChange` on director state change in `spawn_tick`.
  - `rotland/src/hud/config.rs`: Added `horde_color` to `NotificationConfig`, `HordeEventsConfig` struct to `HudConfig`.
  - `rotland/src/hud/notification.rs`: Added `NotificationKind::Horde` with `#f97316` orange-red notification styling.
  - `rotland/src/hud/events.rs`: Implemented pure function `direction_word_from_angle` (8 directions: north, north-east, east, south-east, south, south-west, west, north-west) using Godot coordinate conventions, `direction_word_to_target`, `EventCooldowns` rate limiting, `HordeIndicator` screen-edge arrow renderer (`draw_horde_arrows`). Added unit tests for 8-direction angles, target vectors, and cooldown timers.
  - `rotland/src/hud/mod.rs`: Re-exported events module, added `horde_indicators` and `on_horde_near` method to `HudState` and `Hud`, and rendered horde screen-edge indicator arrows in `draw_hud`.
  - `rotland/src/app/frame_sim.rs`: Updated `z.camp_center` and `z.camp_wall_radius`, processed `frame.events` and director `pending_horde_events` into HUD notifications with 8-direction words and cooldown checks, and registered horde screen-edge indicators.
  - `game/tests/all/director_tests.rs`: Added integration test `director_hud_phase_change_events` validating PhaseChange events.
- **How to test**:
  - Run `tools/gate.sh --task tasks/R19-05-hud-horde-events.md` (or `cargo test hud` and `cargo test director_hud_phase_change_events`).
- **Wiring notes for Claude**:
  - `ZombieFrame.events` is cleared each frame and populated during `zombies.update_full`.
  - Direction words use Godot orientation convention where yaw 0 faces North (-Z) and East is +X.
  - HUD horde notifications are rate-limited per category using values in `data/hud.json`.
  - Near horde indicators place arrows at screen margins pointing toward the horde's world position.
- **Known issues**:
  - None. All clippy lints, unit/integration tests, and gate checks pass cleanly.

## Gate report
- date: 2026-11-22
- base: main
- files changed (12): data/hud.json, game/src/director/state.rs, game/tests/all/director_tests.rs, rotland/src/app/frame_sim.rs, rotland/src/hud/config.rs, rotland/src/hud/events.rs, rotland/src/hud/mod.rs, rotland/src/hud/notification.rs, rotland/src/play.rs, rotland/src/zombies/mod.rs, rotland/src/zombies/types.rs, tasks/R19-05-hud-horde-events.md
- tests passed: 28
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
