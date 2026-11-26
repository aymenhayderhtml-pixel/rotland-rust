# F3 clips: everyday actions (for job A12)

Files: tools/blender/characters/poses_use.py, tools/blender/characters/poses.py, tasks/F3-clips-actions.md

Read `tasks/F-clips-common.md` first. `chop`, `carry`, `sit` already exist; don't redo them.
Upper-body clips must still look right when the legs walk (only key spine, chest, neck, head and arms).
- `pickup_low` (no loop, 0.8 s, full body): bend knees, reach to the ground with the right hand, stand.
- `loot_search` (loop, 1.6 s, full body): crouched, both hands rummaging in front low.
- `open_door_push` (no loop, 0.6 s, upper body): right hand pushes forward at handle height.
- `eat` (no loop, 1.5 s, upper body): right hand to the mouth twice.
- `drink` (no loop, 1.5 s, upper body): right hand to mouth, head tilts back.
- `bandage` (loop, 1.2 s, upper body): right hand wraps around the left forearm in circles.
- `hammer_build` (loop, 0.7 s, upper body): right arm hammers down at waist height, left hand holds.
- `fish_cast` (no loop, 1.0 s, upper body): two hands back over the right shoulder, then whip forward.
- `fish_reel` (loop, 0.8 s, upper body): left hand holds rod forward, right hand circles reeling.
- `search_high` (loop, 1.4 s, upper body): hands reach to shelf height and move items.

## Worker notes
- Author: Gemini Worker (lane F3 - everyday actions for job A12)
- Created `tools/blender/characters/poses_use.py` exporting `CLIPS` with 10 action clips.
- Wired `poses_use.py` into `tools/blender/characters/poses.py` via `PLAYER_ONLY_CLIPS`.
- Clips implemented:
  - `pickup_low` (length: 0.8 s, loop: False, full body): crouches/squats with hips dropped to -0.42 m, right arm and hand reaching down to ground level beside right foot, then smoothly returns to upright standing rest pose. Feet remain planted on ground without sinking or floating.
  - `loot_search` (length: 1.6 s, loop: True, full body): stable crouched stance with both hands rummaging low in front in an alternating search/sift pattern. First and last frames match identically for a seamless loop.
  - `open_door_push` (length: 0.6 s, loop: False, upper body): right hand raises with cocked wrist to handle height, pushes forward with palm flat, then relaxes back to rest. Legs remain unkeyed for locomotion blending.
  - `eat` (length: 1.5 s, loop: False, upper body): right hand raises food to mouth twice with subtle head dips into each bite, lowering to chest height between bites.
  - `drink` (length: 1.5 s, loop: False, upper body): right hand brings container to mouth, head and neck tilt back significantly (+34 to +36 deg) with elbow elevated, followed by clean recovery to rest.
  - `bandage` (length: 1.2 s, loop: True, upper body): left forearm held across lower chest/abdomen, head looking down, right hand wrapping around left forearm in continuous circular path. First and last frames match identically.
  - `hammer_build` (length: 0.7 s, loop: True, upper body): left hand holds steady at waist height, right arm hammers down sharply to waist height, rebounds on impact, cocks high, and strikes again. First and last frames match identically.
  - `fish_cast` (length: 1.0 s, loop: False, upper body): both hands wind up over the right shoulder with torso bladed right, whip forward explosively toward water, followed by settle to rest.
  - `fish_reel` (length: 0.8 s, loop: True, upper body): left hand holds rod steady forward, right hand rotates through a clear circular reel crank. First and last frames match identically.
  - `search_high` (length: 1.4 s, loop: True, upper body): both arms raised to shelf height with head and chest tilted upward, hands dynamically shifting and feeling items across the shelf. First and last frames match identically.
- Verification:
  - Built player character in Blender via `flock /tmp/rotland-blender.lock blender -b --python tools/blender/characters/build_characters.py -- --only=human_player`.
  - Inspected all 10 contact strip images in `assets/models/_previews/human_player_<clip>.png`.
  - Refined keyframe timings and rotations to ensure correct reach depth, clean silhouettes, no joint hyperextension, and no torso penetrations.
  - `python3 -I tools/gate_checks.py --base main --task tasks/F3-clips-actions.md` passed with `scope: PASS`, `numbers: PASS`, `unused: PASS`.
- Anything that still looks off:
  - None. In Blender Workbench preview renders, the 10 clips have clear silhouettes, correct anatomical bends, seamless loop points for the looping clips, and clean standing returns for one-shot clips. Upper-body clips leave legs unkeyed so they blend cleanly with lower-body walk/run animations in the game engine.
