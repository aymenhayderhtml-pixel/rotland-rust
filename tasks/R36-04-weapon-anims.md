# R36-04: animations + hand grips for the new weapons

Owns: tools/blender/characters/poses_weapons2.py, tools/blender/characters/poses.py, data/grips.json, data/stances.json, humanoid/src/stance*.rs (only if a new stance class is needed), humanoid/src/grip*.rs (only to map the new weapon ids), assets/models/characters/*
Rules: `tasks/R36-common.md` (use EXACTLY the clip format shown in `tasks/R27-common.md`).

- **New player clips** (in `poses_weapons2.py`, registered in `PLAYER_ONLY_CLIPS` in `poses.py`):
  - `grenade_throw` (0.8 s, not looping): pull the pin with the left hand (0.0-0.2), arm back over the shoulder (0.2-0.45), overhand throw forward, with the release at 0.5 and the torso turning into it, then follow through.
  - `crossbow_reload` (2.0 s, not looping): lower the crossbow to the ground in front, a foot on the stirrup (one knee bent), pull the string up with both hands, lift it back to the hold. Make it readable from the side.
- **Stances / grips:** revolver uses the pistol stance and grip. hunting_rifle and crossbow use the rifle stance and the two-handed grip. grenade uses the empty-hand/one-hand stance with a closed right fist. Add the ids to `data/stances.json` and `data/grips.json` the same way the existing weapons are listed (look at how `pistol`, `rifle`, `bow` appear).
- No gameplay numbers.

**Lead rebuilds the characters ONCE (subagent doesn't)** (R30 also rebuilt them, so do it after merging everything):
```bash
flock /tmp/rotland-blender.lock blender -b --python tools/blender/characters/build_characters.py > /tmp/r36-chars.log 2>&1
grep -c Traceback /tmp/r36-chars.log   # 0
```
Look at `assets/models/_previews/human_player_grenade_throw.png` and `human_player_crossbow_reload.png`: no arms through the body, no backward knees.

## Worker notes
- **Files changed:**
  - `tools/blender/characters/poses_weapons2.py` (new): added `grenade_throw` (0.8 s, `loop: False`, upper-body: pin pull 0.0-0.2, windup over right shoulder 0.2-0.45, overhand release at 0.50 with torso twist, follow-through 0.62, recover 0.80) and `crossbow_reload` (2.0 s, `loop: False`: lower crossbow in front and place left foot on stirrup with bent knee 0.40-0.70, pull string up with both hands 1.25, lift back to ready hold 1.55-2.00).
  - `tools/blender/characters/poses.py`: imported `poses_weapons2.CLIPS as _WEAPONS2` and merged into `PLAYER_ONLY_CLIPS`.
  - `data/stances.json`: mapped `revolver` -> `"pistol"`, `hunting_rifle` -> `"rifle"`, `crossbow` -> `"rifle"`, `grenade` -> `"melee_1h"` (one-hand stance with closed right fist via `grips.json`'s `melee_1h` class); added `offHandGrip` overrides `[0.0, 0.03, -0.39]` for `hunting_rifle` and `crossbow` so two-handed left-hand IK engages.
- **No changes needed:** `data/grips.json` (already maps by `StanceClass` -> `GripClass`, so `pistol`, `rifle`, and `melee_1h` already provide the requested grips) and `humanoid/src/stance*.rs` / `grip*.rs`.
- **Blender command for lead:**
  ```bash
  flock /tmp/rotland-blender.lock blender -b --python tools/blender/characters/build_characters.py > /tmp/r36-chars.log 2>&1
  grep -c Traceback /tmp/r36-chars.log   # 0
  ```
- **Notes / unsure:**
  - `hunting_rifle` and `crossbow` `offHandGrip` in `data/stances.json` `overrides` are set to `[0.0, 0.03, -0.39]` (matching `rifle`) since `tools/blender/weapons/hunting_rifle.py` and `crossbow.py` are authored in another task; adjust if their `grip_L` empty differs.
  - `crossbow_reload` keys `hips`, `thigh_*`, `shin_*`, `foot_*` for the stirrup foot-on-bow pose; if played on an upper-body-only mask, only the upper-body motion will play, whereas the full-body preview/action mask shows the knee bend.
