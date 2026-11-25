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

## Neon notes

* **All 8 scripts created** in `tools/blender/props/` with `CATEGORY = "props"` (matching `assets/models/props/<name>.glb` in `tasks/R31-round.md` and `tasks/R31-06-models-desert.md`) and `main()` returning `0`/`1` via `common.run_model`:
* `pine_snow.py` (~404 tris, budget 450, ~9.75 m tall)
* `pine_snow_lod1.py` (~144 tris, budget 160)
* `pine_snow_lod2.py` (36 tris, budget 40)
* `rock_ice_01.py` (140 tris, budget 200)
* `snow_drift.py` (76 tris, budget 120, 4 × 2 × 0.6 m, **no collider**)
* `snow_shrub.py` (140 tris, budget 150)
* `boulder_big.py` (240 tris, budget 300, ~5.4 × 4.4 × 3.3 m)
* `rock_cliff_01.py` (282 tris, budget 350, ~8.0 × 6.0 × 3.0 m)


* **Multi-sheet materials verified in `common.py`:** `_to_object` in `tools/blender/common.py` already supports multiple palette materials per mesh via `palette.sheet_of_key(key)` and per-face `poly.material_index`. `pine_snow` (+ LODs), `boulder_big`, and `rock_cliff_01` mix Sheet A (`bark_dark`, `rock_dark`, `rock_mid`, `rock_light`) with Sheet C (`pine_snow_dark`, `snow_white`, `snow_shadow`); `rock_ice_01`, `snow_drift`, and `snow_shrub` use Sheet C only.
* **Check for R31-06:** make sure R31-06 updates the `sheets` list inside `_to_object()` in `tools/blender/common.py` to also append `"C"` when any face uses a Sheet C key (not only `palette_material(sheet="C")`), and does not assume Sheet A is always used if a model is Sheet-C-only (or keeps the existing `"A"`-first pattern if harmless).


* **Ground sink (~5 cm) & LOD alignment:** `common.Model.finalize_and_export()` auto-shifts `min(z)` to `0.0` whenever `grip is None`. Every script builds its geometry centered on `(0, 0)` with its base at `z = -0.05` and passes `grip=(0.0, 0.0, 0.0)` to `common.run_model` so the 5 cm ground sink is preserved on slopes and `pine_snow` LOD0/LOD1/LOD2 trunks stay pinned at the exact same `(0, 0)` origin.
* **R31-03 scatter note:** `snow_drift` exports **no collider** — R31-03 should treat `snow_drift` like grass (no collision). `snow_shrub` has a small 0.4 m core hull collider; remove it if shrubs are intended to be fully walkable.
* **Worker notes for `tasks/R31-07-models-snow-mountain.md`:** Not written to that task file because it is outside the allowed write list; copy the summary above into its `## Worker notes` section if needed.
