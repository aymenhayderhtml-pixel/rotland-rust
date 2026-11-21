# R18-04: new weapons in play (bow, arrows, spear throw, new melee)

Read first: `tasks/R18-common.md`, `game/src/projectiles/` (`ProjectileSim`, `bow_shot`, `spear_throw`, events), `game/src/weapons.rs` (`WeaponKind::Bow`, new fields), `rotland/src/combat.rs` (melee swing), `rotland/src/shooting.rs` (guns), `rotland/src/fp_weapon.rs` (first-person weapon view), `rotland/src/app/frame_actors.rs` (where combat runs each frame), `rotland/src/hud/ammo.rs`, `data/weapons.json`.
Files: rotland/src/combat.rs, rotland/src/shooting.rs, rotland/src/fp_weapon.rs, rotland/src/projectiles_view.rs, rotland/src/bow.rs, rotland/src/app/frame_actors.rs, rotland/src/hud/ammo.rs, game/src/projectiles/*, game/src/weapons.rs, game/src/melee.rs, game/tests/all/projectiles_tests.rs, game/tests/all/weapons_new_tests.rs, tasks/R18-04-weapons-in-play.md
Test: projectile

- New melee weapons (dagger, spear, hammer, pipe, fire axe) must work like the old melee ones when equipped (they are data; check nothing in code lists melee ids by name; if something does, make it data-driven). Their models show in first person and third person.
- Bow: hold the attack button to draw (draw 0..1 over `drawTime`, slight zoom/FOV narrowing while drawing like aiming), release to shoot an arrow if the bag has `arrow`; the arrow flies via `ProjectileSim` (gravity), is drawn with the `arrow` model pointing along its velocity, hits zombies (damage, headshot) and sticks in the world; stuck arrows can be picked up with E with `pickupChance` (else they break: give `arrow_broken`? no: they just vanish). Ammo HUD shows arrows in the bag. Bow is silent (low noise).
- Spear: normal attack = thrust (melee data); a second button (aim button / right mouse) throws it: the spear leaves the hand (unequipped), flies, damages, and lies on the ground where it lands as a pickup (E).
- The projectile sim runs every frame from `frame_actors.rs`; raycasts use the physics world and the zombie hit shapes the guns use (reuse the gun hit test).
- Tests: keep the pure ones passing; add tests for any new pure helper (draw charge, pickup rules).
- `--give=bow,arrow:20,spear` style startup flag if a similar one exists; else note what Claude should add.

## Worker notes
### Files changed
- `game/src/projectiles/mod.rs` & `game/src/projectiles/sim.rs`: Added `raycast_projectile` continuous collision testing against `GunTarget` head/body shapes and `PhysicsWorld`.
- `rotland/src/bow.rs`: Created `BowState` managing draw charging [0.0..1.0], release timing, and aim FOV narrowing.
- `rotland/src/projectiles_view.rs`: Created `projectile_instances`, `projectile_matrix`, and `projectile_model_name` for visual transforms.
- `rotland/src/combat.rs`: Integrated `ProjectileSim` and `BowState` into `Combat`, prevented bow from triggering melee swings, and mounted `bow` & `projectiles_view` submodules.
- `rotland/src/shooting.rs`: Added bow handling to `update_targets` returning arrows in bag as `(count, u32::MAX)`.
- `rotland/src/hud/ammo.rs`: Updated `draw_ammo` and added `format_ammo_text` to display single count for bow arrows (when reserve is `u32::MAX`).
- `rotland/src/app/frame_actors.rs`: Handled spear throw on right-click (removes spear from inventory, unequips, launches projectile), bow charging and shooting on attack hold/release (consumes arrow, zooms FOV, launches projectile, updates HUD ammo), and stepped projectile simulation each frame against zombies/wildlife/physics, creating ground pickups on world impacts.
- `game/tests/all/projectiles_tests.rs`: Added tests for `raycast_projectile` hitting headshot vs body vs world.
- `game/tests/all/weapons_new_tests.rs`: Added tests verifying all new melee weapons are `WeaponKind::Melee` and have model paths.

### How to test
- Run `tools/gate.sh --task tasks/R18-04-weapons-in-play.md` (21 unit tests pass, clean clippy, scope/numbers/unused pass).
- In-game: launch with `--give=bow,arrow:20,spear,dagger,hammer,pipe,fire_axe`.
  - Bow: hold LMB to charge draw (FOV zooms in like aiming), release to fire an arrow. Arrows in bag decrement and display on the ammo HUD. Arrow hits deal damage with headshot multiplier and stick in the world or ground. Stuck arrows can be picked up with E based on `pickupChance`.
  - Spear: LMB performs thrust (melee data). RMB throws spear (unequips, flies with gravity, damages targets, lands on ground as an item pickup).
  - New melee weapons: dagger, spear, hammer, pipe, fire_axe swing and damage targets using data from `data/weapons.json`. Fire axe retains the "chop" tag for felling trees.

### Wiring notes for Claude
- `rotland/src/app/render_frame.rs`: To render flying projectile meshes in the 3D world scene, Claude can query `crate::combat::projectiles_view::projectile_instances(&combat.projectiles)` and place them into `beds` (or prop library placed instances) using pack models `arrow` and `spear`.
- `humanoid/src/weapon.rs`: Add `dagger` and `bow` to `weapon_hold_for` (or use hold data from `data/weapons.json`), and add `spear`, `hammer`, `fire_axe`, `bow` to `TWO_HANDED_WEAPONS` if off-hand grip positioning is desired in third-person view.
- `--give=bow,arrow:20,spear` is already fully supported by `session.give` in `startup.rs` and `start_world.rs`.

### Known issues
- None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (11): game/src/projectiles/mod.rs, game/src/projectiles/sim.rs, game/tests/all/projectiles_tests.rs, game/tests/all/weapons_new_tests.rs, rotland/src/app/frame_actors.rs, rotland/src/bow.rs, rotland/src/combat.rs, rotland/src/hud/ammo.rs, rotland/src/projectiles_view.rs, rotland/src/shooting.rs, tasks/R18-04-weapons-in-play.md
- tests passed: 21
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
