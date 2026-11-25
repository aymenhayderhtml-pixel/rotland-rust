# R31-08: swamp models

Owns: tools/blender/props/{tree_swamp,tree_swamp_lod1,tree_swamp_lod2,reeds,swamp_log_mossy,swamp_stump,lily_mud_patch}.py (new)
Rules: `tasks/LEAD.md` subagent rules (never run blender). Use the "R31 shared names" in `tasks/R31-round.md` EXACTLY. Colours: sheet C names `swamp_moss, swamp_mud, reed_tan, swamp_bark` (R31-06 adds sheet C at the same time; see its step 1 for `palette_material(sheet="C")`). If a model needs sheet A colours too (`leaf_dark`, `bark_dark`), check whether `common.py` supports two materials in one model; if not, stay on sheet C and say so in notes.
Style: copy `tools/blender/props/tree_oak_big.py` (+ LODs), `log_fallen.py`, `stump.py`, `grass_tuft_a.py`, `veg.py`. Low-poly, flat colours.

- `tree_swamp` (≤ 500 tris) + `_lod1` (≤ 180) + `_lod2` (≤ 50): a cypress/mangrove-like tree, with a wide flared base and 4-6 visible roots arching into the ground, a twisted swamp_bark trunk, and a flat drooping swamp_moss canopy with 2-3 hanging moss strands (thin long boxes). 7-10 m.
- `reeds` (≤ 120): a clump of 8-12 thin tall blades (1.2-1.8 m) with 2-3 cattail heads (swamp_mud), reed_tan with a swamp_moss base. Grass-like, no collider.
- `swamp_log_mossy` (≤ 200): like `log_fallen`, half-sunk, with swamp_moss patches on top.
- `swamp_stump` (≤ 150): a broken rotten stump with jagged top, swamp_bark with moss.
- `lily_mud_patch` (≤ 80): a flat, slightly raised dark mud puddle (swamp_mud, 2-3 m wide, 3 cm high) with 4-6 small swamp_moss lily pads on it. No collider.

Origin at the ground centre, sunk ~5 cm. Every file has `main()` returning 0/1 via `common.run_model`, because `build_biome_props.py` imports them by name.

## Worker notes
