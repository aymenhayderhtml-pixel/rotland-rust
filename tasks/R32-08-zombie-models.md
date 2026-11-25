# R32-08: three new zombie models (Blender)

Owns: tools/blender/characters/build_characters.py (only adding 3 entries to the character list + any new colour names), tools/blender/characters/zombie_details.py (only new detail functions for these 3), tools/blender/palette.py (only if a colour must be added; prefer sheet C, which R31 added)
Files: tools/blender/characters/build_characters.py, tools/blender/characters/zombie_details.py, tools/blender/palette.py
Read: tasks/R32-round.md, tasks/LEAD.md, tools/blender/characters/build_characters.py, tools/blender/characters/zombie_details.py, tools/blender/palette.py, tools/blender/common.py
API: tools/blender/characters/humanoid.py
Rules: `tasks/LEAD.md` subagent rules (never run blender; the lead builds once). Use the "R32 shared names" in `tasks/R32-round.md` EXACTLY. Read `build_characters.py` fully: each character is a dict (name, skin, hair, hair_style, shirt, jacket, pants, shoes, blood, hunched, proportions, posture). The specials `zombie_brute`, `zombie_screamer` and `zombie_crawler` are the examples to follow. Read `zombie_details.py` for how extra parts are added, and `palette.py` for the colour names (sheet A, B and C).

- **`zombie_frozen`:** pale blue-white skin (use sheet C `ice_blue` or `snow_shadow` for the skin if the character pipeline can use sheet C colours; check how characters get their palette, since they write a per-character `<name>_palette.png`). A dark winter jacket (`pine_snow_dark` or `pants_dark`), frost: 4-6 small `snow_white`/`ice_blue` shards on the shoulders and head (a detail function). Slightly stiff posture, normal proportions (1.05 width).
- **`zombie_husk`:** dried, thin and tall (width 0.8, height 1.08), with tan leathery skin (`sand_dark` or `dry_brush`), torn khaki clothes (`canvas_khaki`), a bare chest with rib stripes (detail: 3-4 thin dark horizontal bars on the chest), a hunched lunging posture, and no hair.
- **`zombie_bloater`:** swollen (width 1.7, depth 1.8, height 1.0), with sickly green-yellow skin (`swamp_moss` / `crop_green`) and a big round belly (a detail sphere on the torso, attached to the spine bone so it moves with it). 3-4 small `swamp_mud` boils on the belly, a ragged shirt that only covers the shoulders.
- They must use the shared zombie rig and clips, like the other zombies, so they animate with no new clips. The husk can reuse crawler clips only if the crawler entry shows how; otherwise it uses the normal zombie clips.
- The lead checks the build log: 0 Traceback, and the CHARACTER count goes up by 3. Previews: `assets/models/_previews/zombie_frozen*.png` etc. Say in your notes which preview files to look at.

## Worker notes
