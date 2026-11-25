# R31-07: snow and mountain models

Owns: tools/blender/props/{pine_snow,pine_snow_lod1,pine_snow_lod2,rock_ice_01,snow_drift,snow_shrub,boulder_big,rock_cliff_01}.py (new)
Files: tools/blender/props/pine_snow.py, tools/blender/props/pine_snow_lod1.py, tools/blender/props/pine_snow_lod2.py, tools/blender/props/rock_ice_01.py, tools/blender/props/snow_drift.py, tools/blender/props/snow_shrub.py, tools/blender/props/boulder_big.py, tools/blender/props/rock_cliff_01.py
Read: tasks/R31-round.md, tasks/LEAD.md, tools/blender/palette.py, tools/blender/common.py, tasks/R31-06-models-desert.md, tools/blender/props/tree_pine_01.py, tools/blender/props/tree_pine_01_lod1.py, tools/blender/props/tree_pine_01_lod2.py, tools/blender/props/rock_01.py, tools/blender/props/veg.py
Rules: `tasks/LEAD.md` subagent rules (never run blender). Use the "R31 shared names" in `tasks/R31-round.md` EXACTLY. Colours: sheet C names (R31-06 adds sheet C at the same time, so use the names exactly as listed: `snow_white, snow_shadow, ice_blue, pine_snow_dark`) and sheet A names (`rock_dark, rock_mid, rock_light, bark_dark, pine_dark`). Look at `tasks/R31-06-models-desert.md` step 1 for how `palette_material(sheet="C")` works. If one model needs colours from two sheets, check whether `common.py` supports two materials in one model. If not, use ONLY sheet C colours plus rock-looking ones made from `snow_shadow`, and write that in your notes.
Style: copy `tools/blender/props/tree_pine_01.py` (+ `_lod1`, `_lod2`), `rock_01.py`, `veg.py`. Low-poly, flat colours.

- `pine_snow` (≤ 450 tris) + `_lod1` (≤ 160) + `_lod2` (≤ 40): like `tree_pine_01`, but in `pine_snow_dark` with snow_white caps on top of each layer (a thinner cone sitting on the upper half of each tier) and a snowy tip. 8-11 m.
- `rock_ice_01` (≤ 200): an angular icy rock, ice_blue with snow_white on the top faces, 1-2 m.
- `snow_drift` (≤ 120): a low, long, smooth snow mound, 4 × 2 × 0.6 m, snow_white with snow_shadow on one side. No collider needed (tell the lead in notes: R31-03 should treat it like grass, no collider).
- `snow_shrub` (≤ 150): a small bush under snow, pine_snow_dark branches poking out of a snow_white dome.
- `boulder_big` (≤ 300): a huge rounded boulder, 4-6 m, rock_mid/rock_dark faces (sheet A) or, if mixing sheets isn't supported, snow_shadow/snow_white with a snow-dusted top.
- `rock_cliff_01` (≤ 350): a tall cliff slab, 8 m high × 6 m wide × 3 m deep, with angular facets and ledge stripes, meant to stand on mountain slopes. Rock colours, with snow on the top ledges.

Origin at the ground centre, sunk ~5 cm. Every file has `main()` returning 0/1 via `common.run_model`, like the others, because `build_biome_props.py` imports them by name.

## Worker notes
