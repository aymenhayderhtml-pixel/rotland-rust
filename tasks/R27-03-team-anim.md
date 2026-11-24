# R27-03 (A18): teammates and camp NPCs use the new animation system

Files: game/src/team_anim*.rs, game/src/lib.rs, game/tests/all/team_anim_tests.rs, game/tests/all/main.rs, data/team_anim.json, rotland/src/app/frame_team.rs, rotland/src/app/team_*.rs, rotland/src/team_*.rs, rotland/src/companions*.rs, tools/blender/characters/poses.py, tools/blender/characters/build_characters.py, assets/models/characters/*.glb, assets/models/_previews/*, tasks/R27-03-team-anim.md
Rules: `tasks/R27-common.md`.

**Goal (animation only, no AI or gameplay change):** teammates (R23 followers, `data/companions.json`) and camp guards hold their weapon in the right stance, aim and recoil when they shoot, swing melee with the right clip, walk with directional legs, and flinch when hit, by calling the same `HumanoidAnimator` API the player uses (`set_weapon`, `set_aiming`, `on_gun_fired`, `play_action`, `set_move_velocity`). Keep many NPCs cheap with `set_lod` / `set_update_interval` by distance (numbers in `data/team_anim.json`).
- First find how teammates and guards are drawn and animated today (grep `HumanoidAnimator`, `team_models`, `play_action` in rotland/src).
- NPC models only get clips from `CLIPS` in `tools/blender/characters/poses.py`, not `PLAYER_ONLY_CLIPS` (see `build_characters.py`). If an NPC needs a stance/aim clip it lacks, move it to the shared set and rebuild (rules in R27-common).
- Put the state → animator mapping (which clip, when to aim, LOD interval) in a pure module with tests; the rotland side only calls it.
- If a file you need isn't on `Files:`, write the exact edit under Worker notes.
