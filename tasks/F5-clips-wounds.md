# F5 clips: wounds and low health (for job A15)

Files: tools/blender/characters/poses_wound.py, tools/blender/characters/poses.py, tasks/F5-clips-wounds.md

Read `tasks/F-clips-common.md` first. These must look clearly hurt but still readable as walking.
- `limp_walk_left` and `limp_walk_right` (loop, 1.1 s): the named leg is hurt: short weak step on it, the body dips and the other leg takes longer.
- `wounded_idle` (loop, 2.4 s): hunched, weight on one leg, slow heavy breathing.
- `hold_side` (loop, 1.6 s, upper body): left hand presses the right side of the belly, slight bend.
- `hold_arm` (loop, 1.6 s, upper body): the left arm hangs limp, the right hand holds it near the elbow.
- `low_health_walk` (loop, 1.0 s): slow, bent, dragging feet, head low.
- `recover_shake` (no loop, 1.2 s, upper body): straighten up, shake it off, roll the shoulders.

## Worker notes
- Clips created in `tools/blender/characters/poses_wound.py` and exported through `tools/blender/characters/poses.py`:
  - `limp_walk_left` (length: 1.1 s, loop: True): Left leg is hurt. Short, light step on the left foot (t=0.55s), body dips sharply onto the yielding knee and pelvic roll (t=0.70s), early push-off (t=0.88s), while the healthy right leg supports weight longer and takes a full forward stride.
  - `limp_walk_right` (length: 1.1 s, loop: True): Right leg is hurt. True mirror of `limp_walk_left`.
  - `wounded_idle` (length: 2.4 s, loop: True): Hunched forward posture with weight shifted onto the straight right leg while the left knee remains relaxed. 2.4 s deep, labored breathing cycle with chest elevation and shoulder rise at mid-cycle (t=1.2s), looping seamlessly.
  - `hold_side` (length: 1.6 s, loop: True, upper body): Left arm wraps across the abdomen with the left palm firmly pressing the right lower belly / flank. Torso is slightly flexed and tilted towards the injury with head angled down toward the wound. Subtle pained breathing loop.
  - `hold_arm` (length: 1.6 s, loop: True, upper body): Left arm hangs limp and relaxed at the side; right arm crosses over with the right hand holding and supporting the left arm around the elbow. Protective slight slouch with subtle breathing heave.
  - `low_health_walk` (length: 1.0 s, loop: True): Severe low-health shuffle. Spine and chest hunched heavily forward, head hung low staring at the floor, feet dragging close to the ground with low knee lift. Ground clearance tuned to match standard walk.
  - `recover_shake` (length: 1.2 s, loop: False, upper body): Progression from hunched hurt posture (0.0s), deep inhale with shoulders shrugging up (0.3s), rolling shoulders back and down while chest straightens (0.55s), rapid lateral head shake clearing the head ("shaking it off", 0.72s - 0.88s), settling smoothly into the standard standing idle upper-body pose by 1.2s.
- Verification:
  - All 7 contact sheet previews (`assets/models/_previews/human_player_<clip>.png`) inspected.
  - Lowest joint ground elevation checked programmatically: lowest points stay within [-0.052 m, -0.005 m] throughout locomotion cycles, exactly matching standard `walk` (-0.050 m).
  - Inverse kinematics verified: hand placement in `hold_side` and `hold_arm` accurately targets the belly and elbow without penetrating the body mesh.
  - `python3 -I tools/gate_checks.py --base main --task tasks/F5-clips-wounds.md` reports `scope: PASS (3 changed paths)`.
- Known issues / observations:
  - None. All clips loop cleanly (or end at neutral idle for recover_shake) and fulfill all biomechanical requirements.
