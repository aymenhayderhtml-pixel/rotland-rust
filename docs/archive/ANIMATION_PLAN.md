# Character animation system plan (A01-A20)

Goal: a deep animation system that can keep growing up to v2. Neon writes one job at a time (task files `tasks/A<NN>-*.md`), Haiku applies it and runs the gate, and Claude checks a picture after each phase.

## What is wrong today (checked 2026-11-23)
- **Guns dangle.** `pistol_aim`, `rifle_aim`, `pistol_fire`, `rifle_fire`, `reload` and more exist in the player model, but the game never calls `set_aim_pose` and only plays melee swings. The arms just keep their walk pose with a gun attached.
- All weapons share one hold offset, and only some get a left-hand grip, and only while swinging.
- Movement has only idle, walk, run, jump and crouch. There is no strafing, walking backward, turning, leaning, landing or foot placement.
- Ladders, cars, swimming, picking up, wounds and fishing have no animation. Death is one clip.
- The rig has 22 bones and no fingers, so hands cannot wrap around a grip yet (A20).

## Rules for every job
- Clips are written as keyframe tables in `tools/blender/characters/poses_*.py` and baked. Procedural motion lives in `humanoid/`.
- Numbers live in `data/animation.json` (or a data file named by the job).
- The player costs almost nothing per frame: no allocation in the pose path. Zombies stay cheap.
- Every job leaves the game playable. Each new layer has a test.
- To keep helpers fast, Neon writes the wiring too: wiring files are listed in the job's `Files:`.

## Jobs
**Phase 1: foundation and holding**
- A01 Animation core: layers, bone masks, crossfades, full-body states, clip events, no per-frame allocation.
- A02 Weapon holding: a stance per weapon class (pistol, rifle, shotgun, melee 1H/2H, bow, spear, tools), both hands on two-handed guns, per-weapon grip data.
- A03 Aim and shoot: upper body follows the camera up/down, aim in/out, fire, recoil, reload tied to real reload time.

**Phase 2: movement**
- A04 Directional movement: strafe, backward, start/stop, lean into turns, turn in place.
- A05 Feet on the ground: foot and hip IK on slopes and stairs, no foot sliding.
- A06 Jump, fall, hard landing, roll.
- A07 Crouch walk, sprint style, tired and low-stamina body language.
- A08 Wading, swimming, treading water, climbing out.

**Phase 3: using the world**
- A09 Ladder climbing with hands and feet on rungs, getting on and off.
- A10 Stairs and vaulting over low walls and fences.
- A11 Cars: walk to door, open, sit, drive with hands on the wheel, get out, thrown out.
- A12 Actions: pick up, loot, open doors, eat, drink, bandage, chop, build, fish.

**Phase 4: combat and damage**
- A13 Melee v2: combos per weapon, swings with weight, blocked and missed hits.
- A14 Hit reactions by direction and body part, stagger, knockback.
- A15 Wounds: limp, holding a wound, low-health posture, recovery.
- A16 Deaths: several by direction and cause, falling to the ground properly, downed and revive.

**Phase 5: everyone else and v2**
- A17 Zombies on the new system (with LOD so 100+ stay cheap), attacks, grabs, stagger, deaths.
- A18 Teammates and camp NPCs on the new system, guns and melee.
- A19 First-person arms that match, weapon sway, reload and inspect.
- A20 v2 rig: finger bones and real hand grips on every weapon, final polish pass.
