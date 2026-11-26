# R20-04: zombies attack parked cars; car health shows and can break (issue 004)

Read first: `tasks/R20-common.md`, `docs/ISSUES.md` row 004 (+ picture), `game/src/vehicles/*` (`health.rs`, `manager.rs` ~260 ramming and ~270 repair, `gauge.rs`), `data/vehicles.json`, `rotland/src/car_solid.rs`, `rotland/src/app/frame_sim.rs` (drowning damage ~530, how zombies and cars tick), `game/src/zombie/brain_actions.rs` (read only: how zombies attack the player).
Files: game/src/vehicles/*, data/vehicles.json, rotland/src/car_solid.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_world.rs, game/tests/all/*, tasks/R20-04-car-vs-zombies.md
Test: vehicle

1. A zombie that is chasing/attacking and is within reach of a parked (or occupied) car hits the car: lower the car's condition by a damage-per-hit and hit interval from `data/vehicles.json` (new keys, e.g. `zombieHitDamage`, `zombieHitSeconds`; pick values so 3 zombies wreck a 300 HP car in about a minute and write them in Worker notes). Pure function in `game/src/vehicles/` (e.g. `zombie_damage.rs`) that takes the car positions/half sizes and the zombie positions and returns damage per car; `frame_sim.rs` calls it.
2. At 0 the car is wrecked (existing state); the existing `[T]` repair with scrap metal must work on a wrecked parked car.
3. Show the car's health when the player looks at / stands next to a parked car: add it to the car's E-prompt text, e.g. "[E] Drive  (Car 240/300)", or "Wrecked: [T] Repair (scrap metal)".
4. Zombies must not end up standing on the car roof: check the car box in `car_solid.rs` (height, and whether zombies' step-up lets them climb it). Fix it (taller box or stop zombies stepping onto it) and explain the cause in Worker notes.
5. Tests: damage from 0/1/3 zombies in reach, zombies out of reach do nothing, wrecked at 0, no damage below 0.

## Worker notes
- **What changed:**
  1. `data/vehicles.json` & `game/src/vehicles/config.rs`: Added data keys for zombie car damage: `zombieHitDamage: 2.0`, `zombieHitSeconds: 1.2`, `zombieHitReach: 1.5`, and car box dimensions `carHalfWidth: 0.8`, `carHalfHeight: 1.2`, `carHalfLength: 1.9`.
  2. Tuned values: With `zombieHitDamage: 2.0` and `zombieHitSeconds: 1.2`, 1 zombie deals `2.0 / 1.2 = 1.667` DPS (100 HP per minute). Exactly 3 zombies deal `3 * 1.667 * 60 = 300` damage in 60 seconds (1 minute), wrecking a 300 HP car in about a minute.
  3. `game/src/vehicles/zombie_damage.rs`: Created pure functions `is_zombie_in_reach`, `count_zombies_in_reach`, and `calculate_zombie_damage` taking car target boxes and zombie positions.
  4. `game/src/vehicles/manager.rs`: Added `damage_boxes` and `apply_zombie_damage` to apply zombie damage to live cars.
  5. `rotland/src/app/frame_sim.rs`: Wires zombie attacks against live cars each simulation frame for zombies in `Chase` or `Attack` states within reach.
  6. `rotland/src/app/frame_world.rs`: Added parked car health display to the interaction prompt: `[E] Drive  (Car {hp}/{max_hp})` and `Wrecked (Car 0/{max_hp}): [T] Repair (scrap metal)`.
  7. `rotland/src/car_solid.rs`: Fixed zombies climbing onto car roof. Cause: The static car collider box had height `max_y = 1.4` (and sill gap `min_y = 0.2`). When parked on uneven/sloping ground or crowded by other zombies, the height delta fell below the zombies' autostep threshold (`max_step_height = 0.35`), allowing zombies to step directly onto the car roof. Fix: Elevated the collider roof to `max_y = 2.4` and sealed the base (`min_y = 0.0`), creating an impassable barrier that prevents any zombie from stepping onto the car.
  8. `game/tests/all/vehicle_zombie_damage_tests.rs`: Added comprehensive unit tests covering 0/1/3 zombies in reach, out-of-reach zombies, rotated cars, wrecked car damage immunity (clamping at 0), and scrap metal repair reviving wrecked cars.
- **How to test:**
  - Run `tools/gate.sh --task tasks/R20-04-car-vs-zombies.md`.
  - In-game: park a car near zombies; zombies surround and hit it, condition drops, E-prompt shows decreasing health, car becomes wrecked at 0 HP, and `[T]` with scrap metal repairs it back to 50 HP.
- **Wiring notes for Claude:**
  - None required; all changes are self-contained within allowed files.
- **Known issues:**
  - None.

## Gate report
- date: 2026-11-22
- base: main
- files changed (13): data/vehicles.json, game/src/vehicles/car.rs, game/src/vehicles/config.rs, game/src/vehicles/manager.rs, game/src/vehicles/mod.rs, game/src/vehicles/zombie_damage.rs, game/tests/all/main.rs, game/tests/all/vehicles_tests.rs, game/tests/all/vehicle_zombie_damage_tests.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_world.rs, rotland/src/car_solid.rs, tasks/R20-04-car-vs-zombies.md
- tests passed: 29
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
