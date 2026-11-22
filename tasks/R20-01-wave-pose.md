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
