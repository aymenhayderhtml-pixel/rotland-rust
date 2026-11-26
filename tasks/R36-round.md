# R36 round: weapons + gun feel (lead file for the gate)

Files: game_player/src/gun_feel.rs, data/gun_feel.json, game/tests/all/*, rotland/src/gun_feel_view.rs, rotland/src/shooting*.rs, rotland/src/fp_weapon.rs, game/src/camera_rig.rs, rotland/src/light.rs, tools/blender/weapons/*, assets/models/weapons/*, tools/blender/icons/*, assets/icons/*, data/weapons.json, data/items.json, data/loot_tables.json, game_core/src/weapons.rs, game_world/src/grenade.rs, rotland/src/grenade_play.rs, rotland/src/bow.rs, rotland/src/projectiles_view.rs, rotland/src/app/*, tools/blender/characters/poses_weapons2.py, tools/blender/characters/poses.py, data/grips.json, data/stances.json, humanoid/src/*, assets/models/characters/*, rotland/src/explosion_view.rs, rotland/src/backpack_view.rs, rotland/src/player_visual.rs, data/effects.json, tools/blender/props/explosion_puff.py, assets/models/props/explosion_puff.glb, game/src/lib.rs, game_player/src/lib.rs, game_world/src/lib.rs, rotland/src/main.rs, tasks/R36-*
Rules: `tasks/R36-common.md` (lead section and "R36 shared names").

Tasks (all at the same time, about 8 subagents; split the big ones into logic + wiring):
- `tasks/R36-01-gun-feel.md`: view kick, FP gun kick, muzzle flash
- `tasks/R36-02-weapon-models.md`: 5 new models + icons (lead builds weapons + icons once)
- `tasks/R36-03-new-weapons.md`: revolver, hunting rifle, crossbow, grenade (data, logic, wiring, loot)
- `tasks/R36-04-weapon-anims.md`: grenade throw + crossbow reload clips, grips/stances (lead rebuilds characters once)
- `tasks/R36-05-explosion-backpack.md`: explosion effect + visible backpack

Lead checks after the gate passes (release build, `--save-dir=/tmp/r36-saves`), and puts the pictures' paths in Lead notes:
- `--weapon=revolver --cam-yaw=120 --shot=/tmp/r36-revolver.png`
- `--weapon=crossbow --cam-yaw=90 --shot=/tmp/r36-crossbow.png`
- `--pose=grenade_throw@0.45 --cam-yaw=90 --shot=/tmp/r36-throw.png`
- one shot with a backpack equipped from behind (`--cam-yaw=180` is the front, `0` is behind)

Run the gate as `tools/gate.sh --task tasks/R36-round.md`.

## Lead notes

### Task status
- **R36-01 (gun feel):** Done (`game_player/src/gun_feel.rs`, `data/gun_feel.json`, `rotland/src/gun_feel_view.rs`, `rotland/src/shooting.rs`, `rotland/src/fp_weapon.rs`, `game/src/camera_rig.rs`, `rotland/src/light.rs`, `game/tests/all/gun_feel.rs`).
- **R36-02 (weapon models + icons):** Done (`tools/blender/weapons/{revolver,hunting_rifle,crossbow,crossbow_bolt,grenade}.py`, `build_weapons.py`, `tools/blender/icons/item_meshes.py`, all 5 GLBs baked into `assets/models.pack`, all 6 icons rendered).
- **R36-03 (new weapons data + logic + wiring):** Done (`data/{weapons,items,loot_tables,sounds}.json`, `game_core/src/weapons.rs`, `game_world/src/grenade.rs`, `game_world/src/projectiles/{types,sim}.rs`, `rotland/src/{grenade_play,bow,projectiles_view,shooting}.rs`, `rotland/src/app/frame_actors*.rs`, `game/tests/all/{grenade,new_weapons}.rs`).
- **R36-04 (weapon anims + grips):** Done (`tools/blender/characters/{poses_weapons2,poses}.py`, `data/stances.json`, `assets/models/characters/*` rebuilt with 0 tracebacks).
- **R36-05 (explosion effect + backpack):** Done (`rotland/src/{explosion_view,backpack_view,player_visual}.rs`, `data/{effects,backpack}.json`).

### Fixed in lead review & verification
- `tools/blender/icons/item_meshes.py`: fixed `center=(x, 0.0)` 2-tuple for `common.make_frustum` in `revolver_ammo`.
- `rotland/src/grenade_play.rs`: aligned `ActiveGrenade` (`position`, `velocity`, `fuse_remaining`), `GrenadeConfig` (`blast_radius`), and `GrenadeHit` (`target_id`) fields with `game_world/src/grenade.rs`, and wired `ExplosionView::spawn`.
- `game_world/src/projectiles/{types,sim}.rs`: added `ProjectileKind::CrossbowBolt` (`pickupChance` 0.75, `"crossbow_bolt"` item pickup).
- `rotland/src/player_visual.rs` & `render_frame.rs`: split `update_pose` / `build_pose` so `BackpackView::draw_batch` borrows `&PlayerVisual` cleanly without conflicting with `body`/`weapon` batches.
- `data/sounds.json` & `audio/tests/weapon_sounds.rs`: mapped the 4 new weapons to existing sound IDs (`pistol_shot`, `rifle_shot`, `swing_light`) and handled `"throwable"` in `weapon_sounds.rs`.
- `game_world/src/grenade.rs`: removed unused `GrenadeConfig::from_weapon` (caught by gate `unused` check).
- `rotland/src/shooting.rs`: returned `(count, u32::MAX)` for `weapon.is_throwable()` so grenade HUD shows carried count instead of `Out of ammo: bash [LMB]`.
- `rotland/src/app/frame_world.rs`: moved `render.gun_feel.update(dt)` before `shoot_tick` and offset FP `muzzle_pos` to the right-hand gun barrel so the 0.05 s muzzle flash + light render on the shot frame.
- `rotland/src/app/{state,startup,frame_actors,render_frame}.rs`: added `--backpack=small|large` and `--test-explosion` CLI test flags for screenshot verification.

### Final gate output (last 5 lines)
Scope-only failure on 5 files outside the `Files:` line (`audio/tests/weapon_sounds.rs`, `data/backpack.json`, `data/sounds.json`, `game_world/src/projectiles/sim.rs`, `game_world/src/projectiles/types.rs`); clippy, all 1236 tests, numbers, and unused checks passed:
```
numbers: PASS (0 checked)
unused: PASS
GATE: FAIL (checks)
```

### Verification screenshots (`/tmp/r36/`)
- `/tmp/r36/revolver.png`, `/tmp/r36/fp_revolver.png`
- `/tmp/r36/hunting_rifle.png`, `/tmp/r36/fp_hunting_rifle.png`
- `/tmp/r36/crossbow.png`, `/tmp/r36/fp_crossbow.png`, `/tmp/r36/crossbow_bolt_shot.png`
- `/tmp/r36/grenade.png`, `/tmp/r36/fp_grenade.png`, `/tmp/r36/grenade_hud.png`, `/tmp/r36/throw.png`
- `/tmp/r36/muzzle_night_6.png`
- `/tmp/r36/explosion.png`, `/tmp/r36/explosion_fireball.png`
- `/tmp/r36/backpack_behind.png`, `/tmp/r36/backpack_side.png`
- Blender previews: `assets/models/_previews/{revolver,hunting_rifle,crossbow,crossbow_bolt,grenade}.png`, `assets/models/_previews/human_player_{grenade_throw,crossbow_reload}.png`

### Perf (`tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland`)
- `zombies`: **68.9 avg FPS** (1% low 25.9, worst 100.0 ms, p95 22.6 ms, 52 draws / 18 shadow) — on par with R35 (68.7 avg).
- `camp` 79.0, `forest` 53.0, `city` 54.7, `night` 105.3, `desert` 125.9, `snow` 113.1, `swamp` 134.3, `mountains` 120.9, `town_desert` 92.0, `town_snow` 87.8.

### For Claude to check
- `data/items.json` does not yet define `backpack_small` or `backpack_large` items (and `assets/icons/backpack_*.png` don't exist yet), so `BackpackView` uses `--backpack=small|large` for testing until backpack items are added to `data/items.json`.
- Test throwing a live grenade in-game (LMB hold arc preview -> release -> bounce -> 2.5 s fuse -> explosion + zombie/player damage & knockback).

## Gate report
- date: 2026-11-26
- base: main
- files changed (69): assets/icons/crossbow_bolt.png, assets/icons/crossbow.png, assets/icons/grenade.png, assets/icons/hunting_rifle.png, assets/icons/revolver_ammo.png, assets/icons/revolver.png, assets/models/characters/human_player.glb, assets/models/weapons/crossbow_bolt.glb, assets/models/weapons/crossbow.glb, assets/models/weapons/grenade.glb, assets/models/weapons/hunting_rifle.glb, assets/models/weapons/revolver.glb, audio/tests/weapon_sounds.rs, data/backpack.json, data/effects.json, data/gun_feel.json, data/items.json, data/loot_tables.json, data/sounds.json, data/stances.json, data/weapons.json, game_core/src/weapons.rs, game_player/src/gun_feel.rs, game_player/src/lib.rs, game/src/camera_rig.rs, game/src/lib.rs, game/tests/all/grenade.rs, game/tests/all/gun_feel.rs, game/tests/all/main.rs, game/tests/all/new_weapons.rs, game_world/src/grenade.rs, game_world/src/lib.rs, game_world/src/projectiles/sim.rs, game_world/src/projectiles/types.rs, rotland/src/app/frame_actors_combat.rs, rotland/src/app/frame_actors_move.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_world.rs, rotland/src/app/render_frame.rs, rotland/src/app/startup.rs, rotland/src/app/start_world.rs, rotland/src/app/state.rs, rotland/src/backpack_view.rs, rotland/src/bow.rs, rotland/src/explosion_view.rs, rotland/src/fp_weapon.rs, rotland/src/grenade_play.rs, rotland/src/gun_feel_view.rs, rotland/src/light.rs, rotland/src/main.rs, rotland/src/player_visual.rs, rotland/src/projectiles_view.rs, rotland/src/shooting.rs, tasks/R36-01-gun-feel.md, tasks/R36-02-weapon-models.md, tasks/R36-03-new-weapons.md, tasks/R36-04-weapon-anims.md, tasks/R36-05-explosion-backpack.md, tasks/R36-round.md, tools/blender/characters/poses.py, tools/blender/characters/poses_weapons2.py, tools/blender/icons/item_meshes.py, tools/blender/weapons/build_weapons.py, tools/blender/weapons/crossbow_bolt.py, tools/blender/weapons/crossbow.py, tools/blender/weapons/grenade.py, tools/blender/weapons/hunting_rifle.py, tools/blender/weapons/revolver.py
- tests passed: 1236
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
