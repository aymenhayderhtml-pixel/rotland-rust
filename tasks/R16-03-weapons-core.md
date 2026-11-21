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

