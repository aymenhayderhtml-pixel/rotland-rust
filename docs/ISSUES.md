# Owner issues

Problems the owner saw while playing. Written by the `report-issue` skill (a fresh Claude session: the owner pastes a screenshot + a few words). The orchestrator turns open issues into round tasks and marks them fixed.

Status: `open` (reported, cause not checked yet), `checked` (cause found, see notes), `planned` (in a round task), `fixed` (merged; commit named).

| # | Date | Status | Where | What the owner saw | Likely cause / notes | Picture |
|---|---|---|---|---|---|---|
| 001 | 2026-11-22 | checked | Camp, settler NPCs saying hi (`wave` animation) | When a settler waves, their arm sticks out flat and the hand swings left-right sideways (forward/back), like pointing. It should look like a real wave: hand raised up, waving. | The `wave` clip in `tools/blender/characters/poses_camp.py:211` lifts `upper_arm_R` straight out to the side (Z -95) and then bends `forearm_R` only on X (60–100°). With the upper arm horizontal, an X bend swings the forearm forward/back in a flat plane, so the hand never points up. Fix: bend the forearm *upward* (lower the shoulder to about -80, bend the forearm on the axis that lifts it toward the sky so the hand is above the head) and make the waving wiggle a small side-to-side swing of the raised forearm (or a hand/forearm twist), not the big X bend. Then re-export the characters and re-bake `models.pack`, and check a preview render of the clip. Same clip is also used by the player. | [pic](issues/001-npc-wave-camp.png) [pic](issues/001-npc-wave-close.png) |
