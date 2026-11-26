# R35-02: 12 biome detail props + build script + scatter

Owns: tools/blender/props/{tumbleweed,bleached_bones,fence_post_desert,frozen_log,ice_shards,snow_bush_small,cattails,lily_pads,swamp_moss_clump,scree_patch,alpine_flowers,cairn}.py (new), tools/blender/build_detail_props.py (new), data/biomes.json (scatter lists only)
Files: tools/blender/props/*, tools/blender/build_detail_props.py, data/biomes.json
Read only: tools/blender/props/mushrooms.py, grass_tuft_a.py, cattle_skull.py, rock_ice_01.py, reeds.py, tools/blender/build_biome_props.py, tools/blender/common.py, tools/blender/palette.py. Never run blender.

- Small low-poly props (40-250 tris each), palette sheets A/B/C, origin at ground centre.
- `build_detail_props.py`: copy `build_biome_props.py`, MODELS = all 28 names from "R35 shared names" (12 yours, 8 of R35-03, 8 of R35-06).
- `data/biomes.json`: add your 12 to the right biome scatter lists (keep the file's formatting; edit lines, don't re-dump the JSON). Moderate densities; R35-01 handles the dense layer.

## Worker notes
- **What changed:**
  - Created `tools/blender/build_detail_props.py` listing all 28 R35 detail prop names (12 biome props, 8 town clutter props for R35-03, 8 interior furniture props for R35-06), writing sheet C only (`palette.write_palette_png_c()`) and contact sheet `contact_sheet_detail.png`.
  - Created 12 low-poly biome detail prop scripts in `tools/blender/props/` (all outputting to `assets/models/props/<name>.glb`, 80-140 tris, palette sheets A/B/C, origin at ground centre):
    - Desert: `tumbleweed.py`, `bleached_bones.py`, `fence_post_desert.py`
    - Snow: `frozen_log.py`, `ice_shards.py`, `snow_bush_small.py`
    - Swamp: `cattails.py`, `lily_pads.py`, `swamp_moss_clump.py`
    - Mountains: `scree_patch.py`, `alpine_flowers.py`, `cairn.py`
  - Updated `data/biomes.json` scatter lists for `desert`, `snow`, `swamp`, and `mountains` with moderate densities for the 12 new props.
- **Shared-file lines:** None needed.
- **How to see it:** Run `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_detail_props.py > /tmp/r35-bld.log 2>&1` (once R35-03 and R35-06 props exist), bake with `baker`, then inspect `--biome=desert`, `--biome=snow`, `--biome=swamp`, `--biome=mountains`.
- **Notes:** Did not run `blender`, `cargo`, or `tools/gate.sh` per subagent rules.
