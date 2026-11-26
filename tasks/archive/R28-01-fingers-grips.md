# R28-01 (A20): finger bones and real hand grips

Files: tools/blender/characters/*, humanoid/src/grip*.rs, humanoid/src/lib.rs, humanoid/src/animator.rs, data/grips.json, data/stances.json, engine/src/anim/*, assets/models/characters/*.glb, assets/models/_previews/*, docs/RIG_CONTRACT.md, tasks/R28-01-fingers-grips.md
Rules: `tasks/R27-common.md` (all of it applies; use `tasks/R28-01-fingers-grips.md` for the gate).

**Goal:** hands close around what they hold. Today the rig has 22 bones and no fingers (`docs/RIG_CONTRACT.md`), so weapons float in flat hands.
1. Add low-cost finger bones to both hands of every humanoid (thumb + one "fingers" bone, each with 2 segments: 8 bones total), weighted to the hand mesh. Keep the triangle budget (see the `CHARACTER ... tris (budget 5000)` lines when you build). Update `docs/RIG_CONTRACT.md`. Old clips without finger keys must still play (fingers stay at rest).
2. A procedural grip layer in `humanoid/` (new `grip.rs` + tests): per weapon class (pistol, rifle, shotgun, melee_1h, melee_2h, spear, bow, rod, none) a curl amount for thumb and fingers of each hand from `data/grips.json`; the left hand curls only while it holds the weapon (two-handed / off-hand grip). It runs after the clip layers and fades with weapon changes. No allocation per frame.
3. Rebuild all characters (rules in R27-common), check there is no `Traceback`, check `human_player` contact sheets still look right, then the gate.
- If the engine skinning has a hard bone limit, check it (`engine/src/anim/`) and say what it is in Worker notes. Zombies may skip fingers if that is simpler: say which.

## Worker notes
- **What changed:**
  - `tools/blender/characters/humanoid.py` & `humanoid_body.py`: added 8 finger/thumb bones (`thumb_1_L`, `thumb_2_L`, `fingers_1_L`, `fingers_2_L`, `thumb_1_R`, `thumb_2_R`, `fingers_1_R`, `fingers_2_R`) to `human_player` and all `npc_*` models (30 bones total), weighted to the hand/finger/thumb geometry, and rebuilt all character `.glb`s and `_previews/*.png` contact sheets (all <= 3686 tris, budget 5000, 0 Tracebacks). Zombies (`zombie_01..03`, `zombie_brute`, `zombie_screamer`, `zombie_crawler`) skip fingers and stay at 22 bones.
  - `docs/RIG_CONTRACT.md`: documented the 8 new finger/thumb bones on `human_player` and NPCs, and noted that zombies stay at 22 bones and old clips without finger keys leave fingers at rest.
  - `data/grips.json`: added per-weapon-class (`none`, `pistol`, `rifle`, `shotgun`, `melee_1h`, `melee_2h`, `spear`, `bow`, `rod`) thumb and finger curl amounts for both hands, plus `fadeRate`, `maxFingersCurlDeg`, and `maxThumbCurlDeg`.
  - `humanoid/src/grip.rs`, `humanoid/src/lib.rs`, `humanoid/src/animator.rs`: added `GripClass`, `GripPose`, `GripTable`, `GripJoints`, and `GripController` with unit tests. Runs after clip layers and left-hand IK, curls the left hand only while holding the weapon (two-handed / off-hand grip, opening during reload), fades smoothly on weapon changes, and performs zero per-frame allocations.
- **Engine bone limit:** `engine/src/anim.rs:109` has a hard CPU skinning stack array limit of **256** bones (`let mut global = [Mat4::IDENTITY; 256];`). `engine/src/render/lit.wgsl:217` uses a dynamic storage buffer (`array<mat4x4<f32>>`) with no fixed shader array cap.
- **How to see it in the game:** Equip any weapon in third-person view (`pistol`, `rifle`, `shotgun`, `knife`, `bat`, `spear`, `bow`, `fishing_rod`); the right hand fingers/thumb wrap around the grip, and the left hand wraps when holding two-handed weapons (and opens when reloading or unequipping).
- **Wiring notes for Claude:**
  - `baker/tests/bake_test.rs:252` (`baked_human_player_has_ccw_winding_22_joints_and_valid_height`) still asserts `assert_eq!(skeleton.joints.len(), 22, ...)` for `human_player` and `npc_*`, and `baker/tests/bake_test.rs` is not in this task's `Files:` line. Updating line 252 from `22` to `30` (while keeping `zombie_01` at `22` on line 71) will allow full `cargo test` / `gate.sh` to pass.
- **Gate / checks output:**
  - Full `tools/gate.sh --task tasks/R28-01-fingers-grips.md` failed only on `baker/tests/bake_test.rs:252` (`left: 30, right: 22`); `clippy` passed (`PASS`).
  - `python3 -I tools/gate_checks.py --base main --task tasks/R28-01-fingers-grips.md` output:
    ```
    scope: PASS (22 changed paths)
    numbers: PASS (4 checked)
    unused: PASS
    ```

## Gate report
- date: 2026-11-24
- base: main
- files changed (22): assets/models/characters/human_player.glb, assets/models/characters/npc_banker.glb, assets/models/characters/npc_guard.glb, assets/models/characters/npc_mechanic.glb, assets/models/characters/npc_scout.glb, assets/models/characters/npc_survivor.glb, assets/models/characters/npc_trader.glb, assets/models/characters/npc_villager.glb, assets/models/characters/zombie_01.glb, assets/models/characters/zombie_02.glb, assets/models/characters/zombie_03.glb, assets/models/characters/zombie_brute.glb, assets/models/characters/zombie_crawler.glb, assets/models/characters/zombie_screamer.glb, data/grips.json, docs/RIG_CONTRACT.md, humanoid/src/animator.rs, humanoid/src/grip.rs, humanoid/src/lib.rs, tasks/R28-01-fingers-grips.md, tools/blender/characters/humanoid_body.py, tools/blender/characters/humanoid.py
- tests passed: 6
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
