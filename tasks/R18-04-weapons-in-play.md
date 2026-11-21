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

