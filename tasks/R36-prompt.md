# R36 prompt (paste everything inside the box into Hydrogen in Antigravity)

```
You are the LEAD for round R36 of Rotland (/home/aymen/Documents/rotland-rust): WEAPONS + GUN FEEL. QUALITY FIRST: the owner wants this round to look and feel great, not quick.

First read: AGENTS.md, tasks/R36-common.md (rules + "R36 shared names": follow them exactly), tasks/R36-round.md and its 5 task files. Read docs/CODEMAP.md once, and look at how the existing weapons work (bat, pistol, rifle, bow: data/weapons.json, game_core/src/weapons.rs, rotland/src/shooting*.rs, rotland/src/fp_weapon.rs, rotland/src/bow.rs) before you hand out work, so your instructions are precise.

1. Setup from R36-common.md (ONE worktree ../rotland-r36, branch r36, shared CARGO_TARGET_DIR). The game builds only in your steps, never in subagents.
2. Start about 8 subagents at once. Suggested split:
   a. gun feel logic (game_player/src/gun_feel.rs, data/gun_feel.json, tests)
   b. gun feel view (view kick, FP gun kick, muzzle flash + light)
   c. weapon models + icons (revolver, hunting_rifle, crossbow, crossbow_bolt, grenade): detailed low-poly, same scale and grip points as the existing guns
   d. new weapons data + logic (stats, ammo, loot tables, crossbow bolts that can be picked up again, grenade fuse/bounce/damage falloff)
   e. new weapons wiring in rotland (fire, reload, aim, bolt flight, grenade throw arc preview)
   f. animations + grips (grenade throw, crossbow reload, one-hand revolver vs two-hand rifle grips, stances)
   g. explosion effect (flash, smoke puff, debris, camera shake by distance, zombies knocked back)
   h. visible backpack on the player (fits the body, follows the spine, hidden in first person)
   Give each the subagent rules and "R36 shared names" from R36-common.md word for word, its task file, and the exact existing files it should copy the style of. Ask each to write tests for its logic and full "## Worker notes".
3. When they report, REVIEW each diff yourself: names match the shared names, numbers in data files, no half-done stubs. Send a subagent back with clear notes if its work is not good enough.
4. Add the shared-file lines. Blender under `flock /tmp/rotland-blender.lock`: weapons + icons, then characters (commands in the task files; 0 Traceback; old models unchanged except characters). Bake.
5. `tools/gate.sh --task tasks/R36-round.md` in the foreground until GATE: PASS (a scope-only failure about the Files: line is OK; note it).
6. `cargo build --profile play`. Check shots with target/play/rotland, `--save-dir=/tmp/r36-saves`, into /tmp/r36/: the 4 in R36-round.md, plus each new weapon held in third person and first person, a grenade explosion (use --shot-frame to catch it), a crossbow bolt stuck in a zombie or the ground, the backpack from behind and the side, and the muzzle flash at night (--hour=22). Look carefully at every picture. Fix and re-shoot until it looks right: guns in the hand, no clipping, flash at the muzzle, explosion readable, backpack on the back.
7. `tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland`: zombies scene should stay close to R35 (68.7 avg).
8. Fill "## Lead notes" in tasks/R36-round.md (each task done or not, what you fixed in review, last 5 gate lines, every picture path, FPS, anything Claude must check). ONE commit on r36 (git add only changed files). Never push, never touch main.
9. Tell me "R36 done" plus a short summary.

Safety: never `pkill -f`; never write to real save folders; heavy commands under `flock /tmp/rotland-heavy.lock`; never add sounds.
```
