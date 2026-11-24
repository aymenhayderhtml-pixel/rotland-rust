# R28-01 (A20): finger bones and real hand grips

Files: tools/blender/characters/*, humanoid/src/grip*.rs, humanoid/src/lib.rs, humanoid/src/animator.rs, data/grips.json, data/stances.json, engine/src/anim/*, assets/models/characters/*.glb, assets/models/_previews/*, docs/RIG_CONTRACT.md, tasks/R28-01-fingers-grips.md
Rules: `tasks/R27-common.md` (all of it applies; use `tasks/R28-01-fingers-grips.md` for the gate).

**Goal:** hands close around what they hold. Today the rig has 22 bones and no fingers (`docs/RIG_CONTRACT.md`), so weapons float in flat hands.
1. Add low-cost finger bones to both hands of every humanoid (thumb + one "fingers" bone, each with 2 segments: 8 bones total), weighted to the hand mesh. Keep the triangle budget (see the `CHARACTER ... tris (budget 5000)` lines when you build). Update `docs/RIG_CONTRACT.md`. Old clips without finger keys must still play (fingers stay at rest).
2. A procedural grip layer in `humanoid/` (new `grip.rs` + tests): per weapon class (pistol, rifle, shotgun, melee_1h, melee_2h, spear, bow, rod, none) a curl amount for thumb and fingers of each hand from `data/grips.json`; the left hand curls only while it holds the weapon (two-handed / off-hand grip). It runs after the clip layers and fades with weapon changes. No allocation per frame.
3. Rebuild all characters (rules in R27-common), check there is no `Traceback`, check `human_player` contact sheets still look right, then the gate.
- If the engine skinning has a hard bone limit, check it (`engine/src/anim/`) and say what it is in Worker notes. Zombies may skip fingers if that is simpler: say which.
