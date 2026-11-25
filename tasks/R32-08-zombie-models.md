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

## Neon notes

What was changed:

* `tools/blender/characters/build_characters.py`:
* Added 3 entries to `VARIANTS` (bringing the total from 14 to 17):
* `zombie_frozen`: `ice_blue` skin, `pine_snow_dark` winter jacket (`long_sleeves: True`), `pants_dark` shirt/pants, `tire_black` shoes, slightly stiff upright posture, proportions `1.05 x 1.05 x 1.0`.
* `zombie_husk`: `sand_dark` skin and bare torso (`shirt: "sand_dark"`), `canvas_khaki` torn pants, `leather` shoes, `hair_style: "none"`, hunched forward lunging posture, proportions `0.80 x 0.82 x 1.08`. Uses normal `ZOMBIE_CLIPS` (not `zombie_crawler` belly-drag clips, since crawler clips drop `hips` to the floor and override `posture`, which would break the tall hunched lunging silhouette).
* `zombie_bloater`: `swamp_moss` skin and bare lower torso (`shirt: "swamp_moss"`), `pants_dark` pants, `shoe_dark` shoes, `hair_style: "none"`, proportions `1.7 x 1.8 x 1.0`, mild forward lumbering slump.


* Added `palette.write_palette_png_c()` in `build_one_character` alongside sheets A and B, and added wider preview framing for `zombie_bloater` (`target_z=0.85, cam_dist_mult=1.18`).


* `tools/blender/characters/zombie_details.py`:
* Added `_frozen_frost` (6 faceted `ice_blue`/`snow_white` shards on shoulders and head + frosted `snow_shadow` winter collar).
* Added `_husk_ribs` (4 pairs of thin `bark_dark` horizontal rib stripes on the bare chest) and `_husk_torn_clothes` (torn `canvas_khaki`/`dry_brush` waistband shreds and left-shoulder rag).
* Added `_bloater_belly` (big round `crop_green` belly ellipsoid attached to `"spine"` with 4 `swamp_mud` boils placed parametrically on its front surface) and `_bloater_ragged_shoulders` (tattered `shirt_zombie` upper-chest/shoulder mantle and ripped sleeve caps stopping above the belly).


* `tools/blender/palette.py`: unchanged (all required colours already exist on sheets A, B, and C).

Known risk / caller check before running Blender:

* **`tools/blender/characters/humanoid.py` was empty in the task pack**, so I could not inspect how `humanoid.py` resolves palette colours or builds `<name>_palette.png`. The 3 new entries use R31 Sheet C colours (`ice_blue`, `snow_white`, `snow_shadow`, `pine_snow_dark`, `sand_dark`, `dry_brush`, `swamp_moss`, `swamp_mud`). Before running the Blender build, verify that `humanoid.py` includes `palette.COLORS_C` / `palette.PNG_PATH_C` / sheet `"C"` (mirroring how it handles sheet `"B"`). If `humanoid.py` only reads `COLORS` and `COLORS_B`, either add `COLORS_C` to `humanoid.py`'s palette lookup or map the Sheet C colours to Sheet A/B equivalents.

Build command (for the lead):

* `flock /tmp/rotland-blender.lock blender -b --python tools/blender/characters/build_characters.py > /tmp/r32-chars.log 2>&1`
* Check `/tmp/r32-chars.log` for 0 `Traceback` lines and 17 `CHARACTER` lines (`zombie_frozen`, `zombie_husk`, `zombie_bloater` all under the 5000-tri budget).

Preview files to inspect:

* Rest pose previews (384x384):
* `assets/models/_previews/zombie_frozen.png`
* `assets/models/_previews/zombie_husk.png`
* `assets/models/_previews/zombie_bloater.png`


* Animation contact sheets (1600x260) to verify posture, frost shard attachment, bare ribbed chest, and bloater belly tracking with the spine:
* `assets/models/_previews/zombie_frozen_idle.png`, `assets/models/_previews/zombie_frozen_zombie_shamble.png`, `assets/models/_previews/zombie_frozen_zombie_attack.png`
* `assets/models/_previews/zombie_husk_idle.png`, `assets/models/_previews/zombie_husk_zombie_runner_sprint.png`, `assets/models/_previews/zombie_husk_zombie_attack.png`
* `assets/models/_previews/zombie_bloater_idle.png`, `assets/models/_previews/zombie_bloater_zombie_shamble.png`, `assets/models/_previews/zombie_bloater_die.png`
