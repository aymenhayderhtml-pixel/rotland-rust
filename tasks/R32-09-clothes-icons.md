# R32-09: item icons for the warm clothes + the toxic puff prop (Blender)

Owns: tools/blender/icons/* (only new icon scripts + adding them to the icon build list), tools/blender/props/toxic_puff.py (new), tools/blender/build_biome_props.py (only adding `toxic_puff` to MODELS)
Files: tools/blender/icons/item_meshes.py, tools/blender/icons/warm_clothes_meshes.py, tools/blender/props/toxic_puff.py, tools/blender/build_biome_props.py
Read: tasks/R32-round.md, tasks/LEAD.md, tools/blender/icons/item_meshes.py, tools/blender/icons/render_icons.py, tools/blender/icons/make_sheet.py, tools/blender/build_biome_props.py, tools/blender/props/rock_01.py, tools/blender/common.py, tools/blender/palette.py
API: tools/blender/build_all.py
Data: data/items.json
Icon build list: `BUILDERS` in `item_meshes.py`. The four new builders go in the new `warm_clothes_meshes.py` (keeps item_meshes small); register them in `BUILDERS` in item_meshes.py.
Rules: `tasks/LEAD.md` subagent rules (never run blender). Use the "R32 shared names" in `tasks/R32-round.md` EXACTLY. Read `tools/blender/icons/` to see how item icons are made (the script per item, how they're listed and built, the output path and size) and copy that exactly. Read `tools/blender/props/rock_01.py`, `common.py` and `palette.py` (sheets A/B/C).

1. **Icons** for `warm_jacket` (a puffy jacket, `pine_snow_dark` or `cloth_olive` with a lighter collar), `wool_hat` (a beanie with a pom-pom, `red_paint` or `shirt_red`), `desert_scarf` (a folded scarf, `sand_dark` with stripes) and `hand_warmer` (a small rectangular pouch, `car_red`, with a white "heat" squiggle). Same camera, lighting and size as the existing item icons.
2. **`toxic_puff`** prop (≤ 80 tris): a lumpy low-poly cloud ball, 1 m, in `swamp_moss` and `crop_green` faces, origin at its centre (not the ground). It is added to the `build_biome_props.py` MODELS list.
3. Write the exact build commands the lead runs for the icons (from how icons are built today) in your notes.

## Worker notes
