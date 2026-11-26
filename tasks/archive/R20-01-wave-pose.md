# R20-01: camp settlers wave with a raised hand (issue 001)

Read first: `tasks/R20-common.md`, `docs/ISSUES.md` row 001 (+ pictures `docs/issues/001-*.png`), `tools/blender/characters/poses_camp.py` (the `wave` clip, ~line 211), `tools/blender/characters/poses_shared.py` and `build_characters.py` (read only: how clips are keyed and exported).
Files: tools/blender/characters/poses_camp.py, assets/models/characters/*, assets/models/_previews/human_*, assets/models/_previews/npc_*, tasks/R20-01-wave-pose.md
Test: (Blender lane: no cargo)

Today the arm sticks straight out sideways and the forearm swings forward/back flat, like pointing. Make a real friendly wave:
1. Upper arm raised out and up (elbow about shoulder height or a little higher), forearm bent UP so the hand is above the head, palm toward the viewer.
2. The waving motion: the raised forearm swings a small amount side to side (about 20-30 degrees), 2-3 swings per loop. The rest of the body (other arm, legs, spine) stays relaxed as now.
3. Find the right bone axes by testing: render a still of the key frames (front and side views) with a small throwaway script in `/tmp` and look at it. Write in Worker notes which axis lifts the forearm.
4. Re-export all characters: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/characters/build_characters.py` (it re-exports the player and NPC glbs; that's expected).
5. Make sure the other clips did not change (only `wave`).

## Worker notes

- **Files changed**:
  - `tools/blender/characters/poses_camp.py`: updated the `wave` clip table.
  - `assets/models/characters/human_player.glb`, `npc_banker.glb`, `npc_guard.glb`, `npc_mechanic.glb`, `npc_scout.glb`, `npc_survivor.glb`, `npc_trader.glb`, `npc_villager.glb`: re-exported glbs with the updated wave action.
  - `assets/models/_previews/human_*.png`, `assets/models/_previews/npc_*.png`: updated contact sheets (verified visually in `assets/models/_previews/human_player_wave.png`).

- **Bone axes finding**:
  - In rest A-pose, the arm points downward 30° below horizontal along `(0.866, 0, -0.5)`.
  - When `upper_arm_R` is raised to wave position `(5, 0, -95)`:
    - **Local Z axis** is the axis that lifts the forearm toward the sky. Negative Z rotation (`rz = -75°`) bends the elbow upward so the wrist reaches Z=1.70 m and hand tip reaches Z=1.86 m (above the 1.80 m head top).
    - **Local Y axis** is the bone roll/twist axis. A positive Y rotation (`ry = +80°`) rotates the forearm so that the palm faces forward (+Y in world coordinates, directly toward the viewer) with the thumb pointing inward toward the head (-X in world coordinates).
    - **Waving motion**: swinging `forearm_R` local Z between -60° (outward swing) and -90° (inward swing) creates a clean 30° side-to-side arc in the coronal plane, performing 2 full back-and-forth swings across the clip while keeping the palm facing the viewer.

- **Verification**:
  - Verified with throwaway front, side, and 3/4 test renders in `/tmp`.
  - Visual inspection of the 8-frame contact sheet (`assets/models/_previews/human_player_wave.png`) confirms smooth arm raise, raised hand waving side to side with palm forward, and return to rest.
  - Verified through binary glTF inspection that all other 23 animation clips (`idle`, `walk`, `run`, `die`, `melee_1h`, etc.) are 100% identical and only `wave` changed.
  - Ran `python3 -I tools/gate_checks.py --base main --task tasks/R20-01-wave-pose.md`: scope PASS, numbers PASS, unused PASS.

- **Wiring notes for Claude**:
  - Please re-bake `assets/models.pack` using `cargo run --release -p baker` so the updated character models and wave animation are included in the binary asset pack.

- **Known issues**:
  - None.
