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

## Neon notes

### What was created and wired

* **`tools/blender/icons/warm_clothes_meshes.py`**: Added the 4 new item icon mesh builders (`warm_jacket`, `wool_hat`, `desert_scarf`, `hand_warmer`) using existing palette entries across sheets A, B, and C (`pine_snow_dark`, `pine_dark`, `snow_white`, `metal`, `brass`, `red_paint`, `shirt_red`, `white`, `sand_dark`, `sand_light`, `sandstone`, `sandstone_dark`, `car_red`).
* **`tools/blender/icons/item_meshes.py`**: Imported `warm_clothes_meshes` and registered `warm_jacket`, `wool_hat`, `desert_scarf`, and `hand_warmer` in `BUILDERS`.
* **`tools/blender/props/toxic_puff.py`**: Created the low-poly `toxic_puff` prop (`60` tris <= `80` budget, ~1 m across) from three overlapping `subdiv=0` icosahedra painted with `swamp_moss` and `crop_green` faces. Passes `grip=(0.0, 0.0, 0.0)` to `common.run_model` so `Model.finalize_and_export()` keeps the origin at `(0, 0, 0)` (center of the cloud ball) rather than shifting Z to ground level. Exports to `assets/models/nature/toxic_puff.glb`.
* **`tools/blender/build_biome_props.py`**: Added `"toxic_puff"` to `MODELS`.

### Exact Blender build commands for the lead

Run after R32-05 adds `warm_jacket`, `wool_hat`, `desert_scarf`, and `hand_warmer` to `data/items.json`:

1. **Render all item icons (`assets/icons/<id>.png`) and regenerate the contact sheet (`assets/icons/_sheet.png`):**
```bash
flock /tmp/rotland-blender.lock blender -b --python tools/blender/icons/render_icons.py > /tmp/r32-icons.log 2>&1

```


*(Optional fast incremental render of just the 4 new icons, followed by rebuilding `_sheet.png`:)*
```bash
flock /tmp/rotland-blender.lock blender -b --python tools/blender/icons/render_icons.py -- --only=warm_jacket,wool_hat,desert_scarf,hand_warmer > /tmp/r32-icons.log 2>&1
python3 -I tools/blender/icons/make_sheet.py

```


2. **Build biome props including `toxic_puff` (`assets/models/nature/toxic_puff.glb`):**
```bash
flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_biome_props.py > /tmp/r32-props.log 2>&1

```



### Known risks / dependencies

* `render_icons.py` iterates over `data/items.json`; running it before R32-05 adds the 4 item keys to `data/items.json` will skip them, while running `--only=...` before R32-05 would not match any keys in `items.json`. Run the icon build after R32-05 lands.
* `toxic_puff.py` exports to `assets/models/nature/toxic_puff.glb` (matching `CATEGORY = "nature"` used by `tools/blender/props/*.py` and `build_biome_props.py`). R32-07 should load `nature/toxic_puff.glb`.
