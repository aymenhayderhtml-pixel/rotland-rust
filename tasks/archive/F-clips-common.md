# F clip lanes (Gemini Flash): common rules. Read this, then your own task file.

You write **animation clips only** (Python keyframe tables) for the player character, for later jobs of
the animation plan (`docs/ANIMATION_PLAN.md`). No Rust, no cargo, no `tools/gate.sh`.

## How
1. Read `tools/blender/characters/poses.py` (format rules), `poses_air.py` and `poses_move.py` (good examples).
   Arm X positive swings an arm forward; hips `loc` Y is up (negative drops the body).
2. Create **only** your file `tools/blender/characters/poses_<lane>.py` exporting `CLIPS`.
3. To preview, add your import to `poses.py` and add `**_<LANE>` to the `PLAYER_ONLY_CLIPS = {...}` line.
   That one-line change is allowed (Claude resolves it at merge).
4. Build the player (one Blender at a time across all lanes):
   `flock /tmp/rotland-blender.lock blender -b --python tools/blender/characters/build_characters.py -- --only=human_player`
5. Open every `assets/models/_previews/human_player_<clip>.png` (8-frame strips) as an image and fix:
   feet sinking into or floating above the ground (unless the clip is a jump/fall), legs crossing,
   arms passing through the body, joints bent backwards, a loop whose first and last frame don't match.
   Rebuild and look again until each clip looks right.
6. `python3 -I tools/gate_checks.py --base main --task tasks/<your task>.md` must print `scope: PASS`
   for your files (ignore files outside your task that you did not touch).

## Rules
- **Never end your turn while a command is still running.** A Blender build takes 1-3 minutes; wait for it.
  A turn that ends with "I'll wait" counts as a failed task.
- All clips are **in place**: the game moves the character; the clip never travels forward by itself,
  except the hips `loc` used to lower/raise the body.
- Clip names must not clash with existing ones (`grep -n '"<name>"' tools/blender/characters/poses_*.py`).
- Do not commit `assets/models/characters/human_player.glb` or previews. `git add` only your
  `poses_<lane>.py`, `poses.py` and your task file, commit once to your branch. Never push, never touch `main`.
- Fill `## Worker notes` in your task file: the clips you made, their lengths and loop flags, anything
  that still looks off.
