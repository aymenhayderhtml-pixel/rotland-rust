# R35-02: 12 biome detail props + build script + scatter

Owns: tools/blender/props/{tumbleweed,bleached_bones,fence_post_desert,frozen_log,ice_shards,snow_bush_small,cattails,lily_pads,swamp_moss_clump,scree_patch,alpine_flowers,cairn}.py (new), tools/blender/build_detail_props.py (new), data/biomes.json (scatter lists only)
Files: tools/blender/props/*, tools/blender/build_detail_props.py, data/biomes.json
Read only: tools/blender/props/mushrooms.py, grass_tuft_a.py, cattle_skull.py, rock_ice_01.py, reeds.py, tools/blender/build_biome_props.py, tools/blender/common.py, tools/blender/palette.py. Never run blender.

- Small low-poly props (40-250 tris each), palette sheets A/B/C, origin at ground centre.
- `build_detail_props.py`: copy `build_biome_props.py`, MODELS = all 28 names from "R35 shared names" (12 yours, 8 of R35-03, 8 of R35-06).
- `data/biomes.json`: add your 12 to the right biome scatter lists (keep the file's formatting; edit lines, don't re-dump the JSON). Moderate densities; R35-01 handles the dense layer.
