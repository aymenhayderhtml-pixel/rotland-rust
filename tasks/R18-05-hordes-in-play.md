# R18-05: smarter zombies in play: director, hordes, special zombies

Read first: `tasks/R18-common.md`, `game/src/director/`, `game/src/horde/`, `data/director.json`, `data/zombies.json` (brute, screamer, crawler), `rotland/src/play.rs` (`spawn_tick`, ~line 302), `rotland/src/zombies/` (all), `game/src/spawner.rs`, `game/src/flowfield.rs`, `game/src/zombie/` (zombie brain/behaviour).
Files: rotland/src/play.rs, rotland/src/zombies/*, game/src/director/*, game/src/horde/*, game/src/spawner.rs, game/src/spawner_config.rs, game/src/zombie/*, game/tests/all/director_tests.rs, game/tests/all/horde_tests.rs, game/tests/all/spawner_city_tests.rs, game/tests/all/zombie_*.rs, data/director.json, data/zombies.json, data/spawning.json, tasks/R18-05-hordes-in-play.md
Test: horde

- The director ticks every frame (inputs: night, player health, in city, in camp, recent kills, zombies near) and its multipliers scale the spawner's rate and cap.
- Wave events spawn a horde: N zombies out of view at ~70-90 m, grouped, moving as a `Horde` toward the player (or, for the night `CampAttack`, toward the camp gate). Keep zombie count limits: total active zombies never above today's cap x the director's multiplier, and never above 60.
- Special zombies spawn by `specialChance` (night/city): `brute` uses `zombie_brute` model and its stats; `screamer` when it first sees the player plays a scream and calls zombies within `callRadius` to the player's position (they switch to chasing); `crawler` uses its model (its clips are named `zombie_crawler_*`: make the zombie animator pick those clips for this type, data-driven: an optional `clips` map per type in zombies.json).
- Expose two events for the HUD/camp (Claude hooks them): `HordeIncoming { size }` and `CampAttack { size }` as a small Vec of events returned from your tick (note the exact call in Worker notes).
- FPS: hordes must reuse the shared flow field (no per-zombie paths). Note anything that might cost frames.
- Startup flag idea: `--horde-test` spawns a 12-zombie horde 60 m away at start (note it for Claude if you can't add it in your files).

## Worker notes
- Files changed:
  - `data/spawning.json`: added `"hardCap": 60`
  - `data/zombies.json`: added data-driven `"clips"` map for crawler type
  - `game/src/director/state.rs`: exposed `HordeEvent` enum (`HordeIncoming { size }`, `CampAttack { size }`)
  - `game/src/director/system.rs`: populated `horde_events` on `DirectorOutput`
  - `game/src/spawner_config.rs`: added `hard_cap: usize` to `SpawnConfig` (default 60 from data)
  - `game/src/spawner.rs`: scaled spawn rate and cap by director multipliers, enforced hard cap of 60, added special zombie spawn rolls
  - `game/src/zombie/defs.rs`: added optional `clips` map to `ZombieDef`
  - `game/src/zombie/brain.rs`: added `is_screamer`, `has_screamed`, and `horde_steer` to `ZombieBrain`
  - `game/src/zombie/brain_actions.rs`: implemented screamer first-sight scream, and horde boids steering in `think_chase`
  - `game/src/zombie/brain_extras.rs`: added `is_screamer`, `set_screamer`, `has_screamed`, `set_horde_steer`, `alert_chase`
  - `rotland/src/zombies/types.rs`: added `name` to `Look`, data-driven animation clip resolution in `clip_name` from `z.def.clips`
  - `rotland/src/zombies/mod.rs`: loaded all zombie models dynamically from `defs` (including `zombie_brute`, `zombie_screamer`, `zombie_crawler`), added `hordes: HordeSet`, `total_kills`, updated active hordes with boids steering, handled screamer call radius (60m) alerting nearby zombies to player
  - `rotland/src/zombies/spawning.rs`: matched model look dynamically, initialized crawler and screamer brains
  - `rotland/src/zombies/combat_glue.rs`: tracked `total_kills`
  - `rotland/src/play.rs`: wired `Director` into `PlayState`, updated `spawn_tick` to tick director every frame with live inputs, scaled spawner, handled horde wave spawns (70-90m out of view, grouped, boids formation toward player or camp gate), exposed `Vec<HordeEvent>` return value and `pending_horde_events`, supported `--horde-test` startup flag
  - `game/tests/all/horde_tests.rs`: added unit tests for director multipliers scaling cap/rate, hard cap at 60, screamer/crawler special behavior, crawler clips loading, and horde events emission

- How to test:
  - `tools/gate.sh --task tasks/R18-05-hordes-in-play.md` (passes all 12 horde tests, clippy, scope, numbers, unused)
  - Startup flag `--horde-test` spawns a 12-zombie horde 60m away moving in formation toward player

- Wiring notes for Claude:
  - `spawn_tick` signature and call:
    `let horde_events: Vec<HordeEvent> = play_state.spawn_tick(dt, actors.player, forward, night, z, &world.physics, w);`
    `play_state.pending_horde_events` also preserves any unhandled events.
    Events emitted are:
    - `HordeEvent::HordeIncoming { size }`: Wave event triggered, horde spawned heading toward player.
    - `HordeEvent::CampAttack { size }`: Night camp attack wave triggered, horde spawned heading toward camp gate.
  - Startup flag `--horde-test`:
    Already handled internally in `spawn_tick` if `--horde-test` is in `std::env::args()`, or Claude can explicitly call `play_state.spawn_test_horde(player, forward, zombies, physics)`.

- FPS / Performance notes:
  - Hordes reuse the shared player `FlowField`. Followers use O(1) leader-offset boids steering calculations without per-zombie flow field or pathfinding calls, keeping frame times low.
  - Total active zombies are strictly bounded by `cap * multiplier` and never exceed 60.

- Known issues: none.

## Gate report
- date: 2026-11-21
- base: main
- files changed (17): data/spawning.json, data/zombies.json, game/src/director/state.rs, game/src/director/system.rs, game/src/spawner_config.rs, game/src/spawner.rs, game/src/zombie/brain_actions.rs, game/src/zombie/brain_extras.rs, game/src/zombie/brain.rs, game/src/zombie/defs.rs, game/tests/all/horde_tests.rs, rotland/src/play.rs, rotland/src/zombies/combat_glue.rs, rotland/src/zombies/mod.rs, rotland/src/zombies/spawning.rs, rotland/src/zombies/types.rs, tasks/R18-05-hordes-in-play.md
- tests passed: 12
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
