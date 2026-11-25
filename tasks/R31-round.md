# R31 round: weapons + gun feel (lead file for the gate)

Files: game/src/gun_feel.rs, data/gun_feel.json, game/tests/all/*, rotland/src/gun_feel_view.rs, rotland/src/shooting*.rs, rotland/src/fp_weapon.rs, game/src/camera_rig.rs, rotland/src/light.rs, tools/blender/weapons/*, assets/models/weapons/*, tools/blender/icons/*, assets/icons/*, data/weapons.json, data/items.json, data/loot_tables.json, game/src/weapons.rs, game/src/grenade.rs, rotland/src/grenade_play.rs, rotland/src/bow.rs, rotland/src/projectiles_view.rs, rotland/src/app/*, tools/blender/characters/poses_weapons2.py, tools/blender/characters/poses.py, data/grips.json, data/stances.json, humanoid/src/*, assets/models/characters/*, rotland/src/explosion_view.rs, rotland/src/backpack_view.rs, rotland/src/player_visual.rs, data/effects.json, tools/blender/props/explosion_puff.py, assets/models/props/explosion_puff.glb, game/src/lib.rs, rotland/src/main.rs, tasks/R31-*
Rules: `tasks/R31-common.md` (lead section and "R31 shared names").

Tasks (all at the same time, about 8 subagents; split the big ones into logic + wiring):
- `tasks/R31-01-gun-feel.md`: view kick, FP gun kick, muzzle flash
- `tasks/R31-02-weapon-models.md`: 5 new models + icons (lead builds weapons + icons once)
- `tasks/R31-03-new-weapons.md`: revolver, hunting rifle, crossbow, grenade (data, logic, wiring, loot)
- `tasks/R31-04-weapon-anims.md`: grenade throw + crossbow reload clips, grips/stances (lead rebuilds characters once)
- `tasks/R31-05-explosion-backpack.md`: explosion effect + visible backpack

Lead checks after the gate passes (release build, `--save-dir=/tmp/r31-saves`), and puts the pictures' paths in Lead notes:
- `--weapon=revolver --cam-yaw=120 --shot=/tmp/r31-revolver.png`
- `--weapon=crossbow --cam-yaw=90 --shot=/tmp/r31-crossbow.png`
- `--pose=grenade_throw@0.45 --cam-yaw=90 --shot=/tmp/r31-throw.png`
- one shot with a backpack equipped from behind (`--cam-yaw=180` is the front, `0` is behind)

Run the gate as `tools/gate.sh --task tasks/R31-round.md`.

## Lead notes
