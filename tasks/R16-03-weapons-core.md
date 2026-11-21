# R16-03: new weapons, core (bow + arrows, spear, dagger, new melee): data + logic

Read first: `tasks/R16-common.md`, `game/src/weapons.rs`, `game/src/melee.rs`, `game/src/guns_fire.rs` (only to see how hits are reported), `data/weapons.json`, `data/items.json`.
Files: game/src/weapons.rs, game/src/projectiles/*, game/src/lib.rs, game/tests/all/projectiles_tests.rs, game/tests/all/weapons_new_tests.rs, game/tests/all/main.rs, data/weapons.json, data/items.json, tasks/R16-03-weapons-core.md
Test: projectile

- New melee weapons in `weapons.json` + `items.json` (same fields as `bat`/`knife`; numbers picked to fit between existing weapons, explain the choice in Worker notes): `dagger` (fast, low damage, short range, cheap stamina), `spear` (long reach 3 m, narrow arc, thrust), `hammer` (slow, high stagger), `pipe` (cheap, common), `fire_axe` (two-handed, high damage, slow). `model` fields point to `res://assets/models/weapons/<id>.glb` (models come this afternoon).
- New weapon kind `bow` (add to the kind enum; unknown kinds still fail the load): `bow` item + weapon: draw time (hold to draw, damage and speed scale with draw 0..1), `arrowSpeed` m/s at full draw, `gravity` 9.8, `damage`, `headshotMultiplier`, `ammoItem: "arrow"`, silent (low noiseRadius). New items `arrow` (stack 30) and `arrow_broken`.
- Spear throw: `spear` also has `throwable: true` with throw speed/damage (it becomes a projectile and drops where it lands, to pick up).
- `game/src/projectiles/`: `Projectile { pos, vel, kind (Arrow|ThrownSpear), damage, owner }`, `ProjectileSim::step(dt, raycast: impl FnMut(Vec3 from, Vec3 to) -> Option<Hit>) -> Vec<ProjectileEvent>` (events: HitZombie{id, damage, headshot}, HitWorld{point} -> stuck arrow that can be picked up with `pickupChance` (0.6), Expired after `maxFlightSeconds`). Pure: the caller supplies the ray test (so physics is not needed in tests). `bow_shot(draw, aim_origin, aim_dir, def) -> Projectile`.
Tests (`projectiles_tests.rs`, `weapons_new_tests.rs`, names start `projectile_` / `weapons_new_`): all weapons load; bow arc drops by gravity; longer draw goes further; a hit before the end of the step is found (no tunnelling at 60 m/s with 1/60 s steps); headshot multiplier; pickup chance roughly 60% over many seeds; spear throw produces a pickup; unknown kind still fails.

## Worker notes
- **Files changed**:
  - `data/weapons.json`: Added `dagger`, `spear`, `hammer`, `pipe`, `fire_axe`, and `bow`.
  - `data/items.json`: Added item definitions for all 6 new weapons plus `arrow` and `arrow_broken`.
  - `game/src/weapons.rs`: Added `WeaponKind::Bow`, `is_bow(&self)`, and fields for bow/projectile simulation (`gravity`, `draw_time`, `arrow_speed`, `throwable`, `throw_speed`, `throw_damage`, `pickup_chance`, `max_flight_seconds`).
  - `game/src/projectiles/`: New module with `types.rs`, `sim.rs`, `mod.rs` implementing `Projectile`, `ProjectileKind`, `Hit`, `ProjectileEvent`, `ProjectileSim`, `bow_shot`, and `spear_throw`.
  - `game/src/lib.rs`: Registered and exported `projectiles`.
  - `game/tests/all/main.rs`: Registered `projectiles_tests` and `weapons_new_tests`.
  - `game/tests/all/projectiles_tests.rs`: Comprehensive test suite for projectile physics and events (arc gravity drop, draw scaling, continuous collision detection / anti-tunnelling, headshot multiplier, 60% arrow pickup rate across 1000 seeds, spear drop pickups, expiration).
  - `game/tests/all/weapons_new_tests.rs`: Tests verifying weapons load, item registry validation, unknown kind rejection, and stat boundaries for all new weapons.

- **Weapon stat choices & rationale**:
  - `dagger`: Fast, low damage, short reach, cheap stamina. Sits below `knife`: damage 15 (knife 20), range 1.2 m (knife 1.5 m), windup 0.05 s (knife 0.08 s), cooldown 0.25 s (knife 0.35 s), staminaCost 4 (knife 6), stagger 0.10.
  - `spear`: Long reach 3.0 m (longest melee weapon in game), narrow thrust arc 35°, damage 30, windup 0.18 s, cooldown 0.7 s, staminaCost 14, twoHanded. Throwable with throwSpeed 20.0 m/s, throwDamage 45.0, and 100% pickup drop where it lands.
  - `hammer`: Heavy sledgehammer with highest stagger in game (0.8), damage 40, windup 0.28 s, cooldown 0.95 s, staminaCost 20, twoHanded.
  - `pipe`: Common improvised blunt weapon. Damage 22, range 1.7 m, arc 80°, windup 0.12 s, cooldown 0.55 s, staminaCost 10.
  - `fire_axe`: Heavy two-handed axe dealing massive damage (60 vs regular axe 45), range 2.1 m, arc 85°, windup 0.35 s, cooldown 1.2 s, staminaCost 24, maxTargets 2, tags ["chop"].
  - `bow`: Silent ranged weapon (`noiseRadius: 2.0` vs pistol 35.0). Base damage 50, arrowSpeed 45.0 m/s, drawTime 1.0 s, headshotMultiplier 2.5, gravity 9.8, pickupChance 0.6, ammoItem "arrow".
  - `arrow`: Ammo item for bows, weight 0.02 kg, maxStack 30.
  - `arrow_broken`: Material item from shattered arrows, weight 0.01 kg, maxStack 30.

- **Wiring notes for Claude (Day 2)**:
  - Firing bow: Call `game::projectiles::bow_shot(draw_ratio, aim_origin, aim_direction, &weapon_def)` -> returns `Projectile`. Set `projectile.owner = Some(player_id)`.
  - Throwing spear: Call `game::projectiles::spear_throw(aim_origin, aim_direction, &weapon_def)` -> returns `Projectile`.
  - Ticking projectiles: Maintain `ProjectileSim` and call `sim.step(dt, |from, to| -> Option<Hit>)`. The closure raycasts physics world for world impacts (`Hit::world(point)`) and zombie cylinders/spheres for enemy impacts (`Hit::zombie(point, zombie_id, headshot)`).
  - Handling events:
    - `ProjectileEvent::HitZombie { id, damage, headshot }`: Apply damage to zombie `id`.
    - `ProjectileEvent::HitWorld { point, pickup }`: When `pickup` is `Some("arrow")` or `Some("spear")`, spawn a collectible world item at `point`.
    - `ProjectileEvent::Expired`: Projectile despawns cleanly after flight timeout.

- **Note for Day 2 integration**:
  - `game/tests/all/data_integrity.rs:254` has an exact item count assertion (`assert_eq!(total_items, 43)`). With the 8 new items added to `data/items.json`, total items count is now 51. When updating `data_integrity.rs` on day 2, update 43 to 51.

## Gate report
- date: 2026-11-21
- base: main
- files changed (11): data/items.json, data/weapons.json, game/src/lib.rs, game/src/projectiles/mod.rs, game/src/projectiles/sim.rs, game/src/projectiles/types.rs, game/src/weapons.rs, game/tests/all/main.rs, game/tests/all/projectiles_tests.rs, game/tests/all/weapons_new_tests.rs, tasks/R16-03-weapons-core.md
- tests passed: 17
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
