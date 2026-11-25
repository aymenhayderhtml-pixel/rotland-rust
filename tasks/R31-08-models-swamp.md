# R31-08: swamp models

Owns: tools/blender/props/{tree_swamp,tree_swamp_lod1,tree_swamp_lod2,reeds,swamp_log_mossy,swamp_stump,lily_mud_patch}.py (new)
Files: tools/blender/props/tree_swamp.py, tools/blender/props/tree_swamp_lod1.py, tools/blender/props/tree_swamp_lod2.py, tools/blender/props/reeds.py, tools/blender/props/swamp_log_mossy.py, tools/blender/props/swamp_stump.py, tools/blender/props/lily_mud_patch.py
Read: tasks/R31-round.md, tasks/LEAD.md, tools/blender/palette.py, tools/blender/common.py, tasks/R31-06-models-desert.md, tools/blender/props/tree_oak_big.py, tools/blender/props/tree_oak_big_lod1.py, tools/blender/props/tree_oak_big_lod2.py, tools/blender/props/log_fallen.py, tools/blender/props/stump.py, tools/blender/props/grass_tuft_a.py, tools/blender/props/veg.py
Rules: `tasks/LEAD.md` subagent rules (never run blender). Use the "R31 shared names" in `tasks/R31-round.md` EXACTLY. Colours: sheet C names `swamp_moss, swamp_mud, reed_tan, swamp_bark` (R31-06 adds sheet C at the same time; see its step 1 for `palette_material(sheet="C")`). If a model needs sheet A colours too (`leaf_dark`, `bark_dark`), check whether `common.py` supports two materials in one model; if not, stay on sheet C and say so in notes.
Style: copy `tools/blender/props/tree_oak_big.py` (+ LODs), `log_fallen.py`, `stump.py`, `grass_tuft_a.py`, `veg.py`. Low-poly, flat colours.

- `tree_swamp` (≤ 500 tris) + `_lod1` (≤ 180) + `_lod2` (≤ 50): a cypress/mangrove-like tree, with a wide flared base and 4-6 visible roots arching into the ground, a twisted swamp_bark trunk, and a flat drooping swamp_moss canopy with 2-3 hanging moss strands (thin long boxes). 7-10 m.
- `reeds` (≤ 120): a clump of 8-12 thin tall blades (1.2-1.8 m) with 2-3 cattail heads (swamp_mud), reed_tan with a swamp_moss base. Grass-like, no collider.
- `swamp_log_mossy` (≤ 200): like `log_fallen`, half-sunk, with swamp_moss patches on top.
- `swamp_stump` (≤ 150): a broken rotten stump with jagged top, swamp_bark with moss.
- `lily_mud_patch` (≤ 80): a flat, slightly raised dark mud puddle (swamp_mud, 2-3 m wide, 3 cm high) with 4-6 small swamp_moss lily pads on it. No collider.

Origin at the ground centre, sunk ~5 cm. Every file has `main()` returning 0/1 via `common.run_model`, because `build_biome_props.py` imports them by name.

## Worker notes

## Neon notes

* **What was written**: All 7 swamp prop scripts in `tools/blender/props/`, exporting to `assets/models/props/<name>.glb` (`CATEGORY = "props"` per `tasks/R31-round.md` shared names) with `main()` returning `0`/`1` via `common.run_model`:
* `tree_swamp` (~468 / 500 tris, ~8.7 m tall, trunk collider)
* `tree_swamp_lod1` (~164 / 180 tris, trunk collider)
* `tree_swamp_lod2` (~44 / 50 tris, trunk collider)
* `reeds` (~112 / 120 tris, 10 tall `reed_tan` blades 1.25-1.78 m + 3 `swamp_mud` cattail heads + `swamp_moss` base, no collider)
* `swamp_log_mossy` (~176 / 200 tris, half-sunk with moss patches, snapped stubs and mud flanks, log collider)
* `swamp_stump` (~138 / 150 tris, jagged broken top with recessed rotten core, 4 roots and moss, body collider)
* `lily_mud_patch` (~78 / 80 tris, ~2.6×2.3 m raised 3 cm mud puddle with 5 closed manifold notched lily pads, no collider)


* **Origin & ground sink**: `common.Model.finalize_and_export()` lifts `min(z)` to `0.0` whenever `grip is None`. To keep each prop's origin at ground centre `(0, 0, 0)` with bases/skirts sunk ~5-8 cm below ground (and `swamp_log_mossy` half-sunk), every script builds centered around `(0, 0)` and passes `grip=(0.0, 0.0, 0.0)` to `common.run_model`. Since every model's bounding radius exceeds 0.5 m, the `_is_weapon` preview framing floor (`0.12` vs `0.5`) has no effect on preview framing.
* **Palette & multi-material**: `common.py` already supports multiple material slots per mesh (`sheets` + `poly.material_index`). `tree_swamp` and `tree_swamp_lod1` pair Sheet C (`swamp_bark`, `swamp_moss`) with Sheet A (`leaf_dark`) for deeper canopy shading; all other 5 models stay strictly on Sheet C (`swamp_moss`, `swamp_mud`, `reed_tan`, `swamp_bark`).
* **What the caller/lead must do**:
* Apply R31-06 first so Sheet C (`COLORS_C`, `write_palette_png_c`, and `sheet="C"` handling in `palette.py`/`common.py`) and `tools/blender/build_biome_props.py` are in place.
* Run `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_biome_props.py > /tmp/r31-props.log 2>&1`, verify 0 tracebacks and `BUILD OK`, inspect the generated previews in `assets/models/_previews/`, and run `cargo run --release -p baker`.


* **Known risks**:
* In `tree_swamp` and `tree_swamp_lod1`, mixing Sheet A (`leaf_dark`) and Sheet C assumes R31-06 extends `common._to_object`'s `sheets` list the same way Sheet B is handled (keeping multi-sheet indexing working). If R31-06 forces single-sheet meshes, switch `"leaf_dark"` to `"swamp_moss"` in those two files.
