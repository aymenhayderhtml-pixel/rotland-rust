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
