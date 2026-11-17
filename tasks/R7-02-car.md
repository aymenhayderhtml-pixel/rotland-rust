# R7-02: The car — Rust port, logic only

Read `tasks/R7-common.md` first.

Port to `rust/game/src/vehicles/` the logic of `scripts/Vehicles/` (`data/vehicles.json`): `Car.cs`, `CarDrive.cs`, `CarControls.cs`, `CarDriver.cs` (getting in/out with E, exit spot), `CarFuel.cs`, `CarHealth.cs`, `CarLights.cs` (on/off state only), `CarManager.cs`, `CarPoint.cs` + `CarPointKind.cs`, `CarRunOver.cs` (hitting zombies/animals), `CarSpawnRules.cs`, `CarGauge.cs` (speed/fuel numbers only), `CarCamera.cs` (the camera follow math only), `VehicleConfig.cs` + `VehicleConfigData.cs`. Skip `CarModel.cs`, `CarSound.cs` drawing/audio (but output sound events: engine rpm-ish value 0..1, crash, horn).
- Godot uses a RigidBody3D with hand-made raycast wheels (`CarDrive.cs`). The Rust physics crate has no rigid bodies for you, so **simulate the body yourself** in `vehicles/body.rs`: position, rotation (`glam::Quat`), linear + angular velocity, mass and a box inertia from the config, gravity, integrate once per fixed step (use 1/60 s sub-steps inside `update(dt)`). Each wheel ray uses `PhysicsWorld::raycast(origin, dir, max) -> Option<Hit>` (open `rust/physics/src/world.rs`). Port the spring, damper, drive, brake, steer and tyre side-grip numbers from `CarDrive.cs` exactly.
- The car also must not drive through buildings/trees: after integration, push the body out with a few horizontal raycasts at the car's corners (simple, documented), and say so in the report.
- API: `Cars` manager with `spawn_rules`, `update(dt, &CarInput, &PhysicsWorld) -> CarFrame` (input: throttle, brake, steer, handbrake, horn, lights toggle, from the player only when driving), `try_enter(player_pos, facing) -> Option<car_id>`, `exit(car_id, &PhysicsWorld) -> Vec3 exit spot`, `draw_states()` (id, model name, world matrix, 4 wheel matrices with spin and steer, lights on), `run_over(&[(u64, Vec3)]) -> Vec<(u64, DamageInfo, Vec3 knockback)>`, fuel use and refuel from a `fuel_can` item, health and wreck.
- Save: exact keys of `CarManager.cs`/`Car.cs` Save/Load; say the system id in the report.
Tests (`rust/game/tests/vehicles_tests.rs`, flat physics world like `rust/game/tests/animals_tests.rs`): config loads, a parked car settles on the ground at the right ride height (trace it from the spring numbers), throttle moves it forward and fuel drops, steering turns it, brake stops it, running over a zombie at speed reports damage, enter/exit spots, save round trip.


## Worker notes
- **Files created/changed**:
  - `rust/game/src/vehicles/mod.rs`
  - `rust/game/src/vehicles/config.rs`
  - `rust/game/src/vehicles/fuel.rs`
  - `rust/game/src/vehicles/health.rs`
  - `rust/game/src/vehicles/controls.rs`
  - `rust/game/src/vehicles/point.rs`
  - `rust/game/src/vehicles/gauge.rs`
  - `rust/game/src/vehicles/spawn_rules.rs`
  - `rust/game/src/vehicles/camera.rs`
  - `rust/game/src/vehicles/run_over.rs`
  - `rust/game/src/vehicles/body.rs`
  - `rust/game/src/vehicles/car.rs`
  - `rust/game/src/vehicles/manager.rs`
  - `rust/game/src/vehicles/save.rs`
  - `rust/game/src/lib.rs` (added `pub mod vehicles;` and `pub use vehicles::*;`)
  - `rust/game/tests/vehicles_tests.rs`
- **Public API**:
  - `Cars`: `new(cfg)`, `from_repo()`, `spawn_car()`, `update(dt, input, physics) -> CarFrame`, `try_enter(player_pos, facing) -> Option<String>`, `exit(car_id, physics) -> Vec3`, `draw_states() -> Vec<CarDrawState>`, `run_over(targets) -> Vec<(u64, DamageInfo, Vec3)>`, `refuel(car_id, inventory) -> bool`, `respawn_chunk(coord)`, `store_chunk(coord)`, `to_save() -> Value`, `from_save(data)`.
  - `Car`: `body`, `tank`, `condition`, `trunk`, `speed_kmh()`, `forward_speed()`, `is_engine_on()`, `is_driven()`, `to_record()`, `apply_record()`, `draw_state()`.
  - `CarBody`: custom rigid body simulation, box inertia, suspension raycasts, corner obstacle pushout.
  - `CarInput`, `CarControls`, `CarFuel`, `CarHealth`, `CarPoint`, `CarPointKind`, `CarCamera`, `CarRunOver`, `CarSpawnRules`, `describe_gauge`.
- **Godot Save Keys & System ID**:
  - System ID: `"cars"`
  - Car keys: `"id"`, `"colour"`, `"x"`, `"y"`, `"z"`, `"qx"`, `"qy"`, `"qz"`, `"qw"`, `"fuel"`, `"health"`, `"trunk"` (with `"stacks"` array of `{ "slot", "id", "count" }`).
- **Obstacle pushout**:
  - In `CarBody::pushout_obstacles`, horizontal raycasts are cast from the vehicle center (y = 0.4 m) in 8 directions (front, rear, left, right, and 4 corners) against static colliders in `PhysicsWorld`. If a penetration is detected, the body position is offset along the hit normal and velocity into the obstacle is cancelled.
- **Traced Numbers**:
  - Config verification: all 43 fields verified against `data/vehicles.json`.
  - Parked settling equilibrium:
    - Mass $M = 1100$ kg, $g = 9.8$ m/s$^2 \implies W_{\text{total}} = 10780$ N.
    - $W_{\text{wheel}} = 10780 / 4 = 2695$ N.
    - Stiffness $k = 27000$ N/m $\implies$ compression $c = 2695 / 27000 = 0.0998148$ m.
    - Height at ground $y=0$: body origin settles to $y = -c = -0.0998148$ m.
  - Run-over damage: speed 40 km/h $\times$ 1.8 damage/km/h $= 72.0$ damage. Car condition damage $= 2.0$.
  - Fuel consumption: full throttle at $0.001$ L/m; 3s driving covers $> 20$ m and consumes $\sim 0.02 - 0.05$ L ($< 5.0$ L).
  - Gauge: 50% fuel with 10 cells $\implies [#####-----]$.
  - POI parking offset: $x = 10.0 + (5.0 + 3.5) \times 1.0 = 18.5$.
