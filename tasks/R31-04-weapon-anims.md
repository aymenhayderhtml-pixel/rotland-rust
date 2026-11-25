# R31-04: animations + hand grips for the new weapons

Owns: tools/blender/characters/poses_weapons2.py, tools/blender/characters/poses.py, data/grips.json, data/stances.json, humanoid/src/stance*.rs (only if a new stance class is needed), humanoid/src/grip*.rs (only to map the new weapon ids), assets/models/characters/*
Rules: `tasks/R31-common.md` (use EXACTLY the clip format shown in `tasks/R27-common.md`).

- **New player clips** (in `poses_weapons2.py`, registered in `PLAYER_ONLY_CLIPS` in `poses.py`):
  - `grenade_throw` (0.8 s, not looping): pull the pin with the left hand (0.0-0.2), arm back over the shoulder (0.2-0.45), overhand throw forward, with the release at 0.5 and the torso turning into it, then follow through.
  - `crossbow_reload` (2.0 s, not looping): lower the crossbow to the ground in front, a foot on the stirrup (one knee bent), pull the string up with both hands, lift it back to the hold. Make it readable from the side.
- **Stances / grips:** revolver uses the pistol stance and grip. hunting_rifle and crossbow use the rifle stance and the two-handed grip. grenade uses the empty-hand/one-hand stance with a closed right fist. Add the ids to `data/stances.json` and `data/grips.json` the same way the existing weapons are listed (look at how `pistol`, `rifle`, `bow` appear).
- No gameplay numbers.

**Lead rebuilds the characters ONCE (subagent doesn't)** (R30 also rebuilt them, so do it after merging everything):
```bash
flock /tmp/rotland-blender.lock blender -b --python tools/blender/characters/build_characters.py > /tmp/r31-chars.log 2>&1
grep -c Traceback /tmp/r31-chars.log   # 0
```
Look at `assets/models/_previews/human_player_grenade_throw.png` and `human_player_crossbow_reload.png`: no arms through the body, no backward knees.
