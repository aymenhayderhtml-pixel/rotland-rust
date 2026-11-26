# R37-05: gunsmith bench + town gun shop models

Owns: tools/blender/props/gunsmith_bench.py (new), tools/blender/buildings/gun_shop.py (new), tools/blender/build_r37_models.py (new: builds ONLY R37 models: these 2 + R37-06's 9 attachments + icons)
Read: tasks/R37-common.md, tools/blender/building_parts/ (base_workbench), tools/blender/buildings/house_small.py, desert_store.py, kit.py, tools/blender/props/crate_loot.py, tools/blender/common.py, palette.py, tools/blender/build_detail_props.py (copy its runner), tools/blender/icons/ (how icons are rendered). Never run blender.

- `gunsmith_bench`: a sturdy workbench (~2.2 × 0.9 m), vise, scattered tools, a wall rack with 3 rifles silhouettes, a hanging lamp, ammo boxes; 600-1500 tris.
- `gun_shop`: one-storey shop, walk-in exactly like house_small (door size, floor height, wall thickness), big crossed-guns sign on the front, barred window, counter inside, wall racks; neutral colours so it fits all 4 biomes; 1500-3000 tris.
- Add `gun_shop` footprint to the town data via the lead (report the line for `data/towns.json` footprints).
