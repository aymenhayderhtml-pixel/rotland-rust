# R31-06: desert models + colour sheet C + the biome props build script

Owns: tools/blender/palette.py (add sheet C only), tools/blender/common.py (only the sheet C support in `palette_material`/`paint`), tools/blender/build_biome_props.py (new), tools/blender/props/{cactus_tall,cactus_round,desert_shrub,rock_sandstone_01,rock_sandstone_02,cattle_skull,tree_desert_dead,tree_desert_dead_lod1,tree_desert_dead_lod2}.py (new)
Files: tools/blender/palette.py, tools/blender/common.py, tools/blender/build_biome_props.py, tools/blender/props/cactus_tall.py, tools/blender/props/cactus_round.py, tools/blender/props/desert_shrub.py, tools/blender/props/rock_sandstone_01.py, tools/blender/props/rock_sandstone_02.py, tools/blender/props/cattle_skull.py, tools/blender/props/tree_desert_dead.py, tools/blender/props/tree_desert_dead_lod1.py, tools/blender/props/tree_desert_dead_lod2.py
Read: tasks/R31-round.md, tasks/LEAD.md, tools/blender/palette.py, tools/blender/common.py, tools/blender/build_world_props.py, tools/blender/props/tree_pine_01.py, tools/blender/props/tree_pine_01_lod1.py, tools/blender/props/tree_pine_01_lod2.py, tools/blender/props/rock_01.py, tools/blender/props/veg.py, tools/blender/props/tree_dead.py, tools/blender/props/tree_dead_lod1.py, tools/blender/props/tree_dead_lod2.py
Rules: `tasks/LEAD.md` subagent rules (never run blender; the lead builds once). Use the "R31 shared names" in `tasks/R31-round.md` EXACTLY. Style guide: the `blender-art` skill if your tool has it, otherwise copy the style of `tools/blender/props/tree_pine_01.py`, `tree_pine_01_lod1.py`, `rock_01.py`, `veg.py` and `tools/blender/common.py`. Low-poly, flat colours from the palette only, no textures.

1. **Sheet C** in `palette.py`: `COLORS_C`, exactly the 16 names from the shared names in that order, with these RGB values:
   `sand_light 226,200,148 · sand_dark 196,165,110 · sandstone 190,120,80 · sandstone_dark 150,90,60 · cactus_green 88,130,70 · cactus_dark 60,95,50 · dry_brush 150,125,80 · bone_white 225,215,190 · snow_white 236,240,245 · snow_shadow 190,200,215 · ice_blue 170,205,230 · pine_snow_dark 40,75,60 · swamp_moss 70,90,45 · swamp_mud 75,62,42 · reed_tan 170,150,95 · swamp_bark 70,58,45`.
   Unused cells are grey like sheet B. Add `PNG_PATH_C` (`assets/textures/palette_c.png`) and `write_palette_png_c()`. Copy how sheet B works (names, offsets, `KEY_B_OFFSET`) for a `KEY_C_OFFSET`.
   In `common.py`, `palette_material(sheet="C")` and `paint()` must support sheet C names the same way sheet B works. **Sheets A and B and every existing model must stay byte-for-byte the same:** add, don't change.
2. **Models** (the budget is the max triangles, passed to `common.run_model`; category `props`):
   - `cactus_tall` (≤ 300 tris): saguaro, 3-5 m, a trunk plus 1-2 bent arms, with vertical ribs suggested by an 8-sided cross-section with alternating cactus_green/cactus_dark faces.
   - `cactus_round` (≤ 200): a barrel cactus, 0.6-0.9 m, squashed sphere, with a few tiny bone_white spines (small pyramids, ≤ 6).
   - `desert_shrub` (≤ 150): a low dry tumbleweed-like shrub, dry_brush + sandstone_dark twigs, 0.8 m wide.
   - `rock_sandstone_01` / `_02` (≤ 200 each): flat-topped layered rocks (2-3 stacked, slightly offset slabs), sandstone with sandstone_dark bands, 1.5-3 m.
   - `cattle_skull` (≤ 120): a small skull with horns, bone_white, lying on the ground, 0.6 m.
   - `tree_desert_dead` (≤ 400) + `_lod1` (≤ 150) + `_lod2` (≤ 40): a bare twisted desert tree, bleached grey-brown (use sheet A `bark_light` and sheet C `dry_brush`; one model may use two sheets only if `common` already supports two materials. Otherwise sheet C only). Copy how the existing tree LODs are made.
   Origin at the ground centre, and sink ~5 cm into the ground like the other props. Preview PNGs come out of `run_model` as usual.
3. **`tools/blender/build_biome_props.py`:** copy `build_world_props.py`, with `MODELS` = ALL new R31 models from the shared names (desert, snow/mountain and swamp, including the `_lod1`/`_lod2` ones). It writes sheet C first (`write_palette_png_c()`), never rewrites A or B, makes a contact sheet `contact_sheet_biomes.png`, and exits 1 on any failure. R31-07 and R31-08 write the other model scripts at the same time, with exactly those names.

## Worker notes
