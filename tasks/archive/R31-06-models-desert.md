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

## Neon notes

* **Sheet C (`palette.py` & `common.py`):**
* Added `COLORS_C` (all 16 R31 shared names in exact order, unused cells 16–31 filled with `GREY_FILL`), `NAMES_C`, `PNG_PATH_C` (`assets/textures/palette_c.png`), `KEY_C_OFFSET = CELLS * 2`, and `write_palette_png_c()`. Extended `sheet_of`, `index_of`, `key_of`, `name_of_key`, and `sheet_of_key` to support `"C"`.
* Updated `common.palette_material(sheet="A")` with `MATERIAL_NAME_C = "palette_c"`, and updated `_to_object` so models without sheet C colors keep the exact previous slot layout (`["A"]` or `["A", "B"]`) for byte-for-byte identity, while sheet C models attach only the sheets they actually use (`["C"]`, or `["A", "C"]` for two-sheet models like `tree_desert_dead`).
* `common.paint` / `paint_faces` automatically support sheet C via `palette.key_of`.


* **Origin & ground sink:**
* All 9 desert props pass `grip=(0.0, 0.0, 0.0)` to `common.run_model` so `finalize_and_export()` keeps the origin strictly at ground centre `(0, 0, 0)` instead of shifting `min(z)` up to `0`. Every prop extends 4.5–8 cm below `z = 0` so it seats cleanly on sloped desert dunes without floating.


* **Desert models (`CATEGORY = "props"`, exporting to `assets/models/props/<name>.glb`) & exact triangle counts:**
* `cactus_tall`: 266 tris (budget 300) — 4.1 m saguaro, 2 bent arms, 8-sided alternating `cactus_green`/`cactus_dark` faces + trunk collider.
* `cactus_round`: 134 tris (budget 200) — 0.72 m ribbed barrel cactus with 6 small `bone_white` 4-sided pyramid spines + hull collider.
* `desert_shrub`: 128 tris (budget 150) — 0.8 m low dry tumbleweed-like shrub in `dry_brush` + `sandstone_dark`.
* `rock_sandstone_01`: 172 tris (budget 200) — 2.7 m wide flat-topped 3-slab layered outcrop with `sandstone`/`sandstone_dark` strata + 2 base chunks + collider.
* `rock_sandstone_02`: 164 tris (budget 200) — 2.6 m tall layered hoodoo/butte with overhanging flat cap + side step-shelf + collider.
* `cattle_skull`: 100 tris (budget 120) — 0.6 m `bone_white` skull with curved horns and recessed dark eye-socket facets, resting tilted on the ground.
* `tree_desert_dead`: 304 tris (budget 400) — 4.2 m bare twisted desert tree using `bark_light` (sheet A) + `dry_brush` (sheet C), roots, 5 bent limbs, 12 twigs, 2 snags + trunk collider.
* `tree_desert_dead_lod1`: 116 tris (budget 150) — matching twisted silhouette, 5-sided trunk, 4 bent limbs + crown limb + 3 forks + trunk collider.
* `tree_desert_dead_lod2`: 36 tris (budget 40) — matching 4-sided bent trunk (20 tris) + 4 tapered limbs (16 tris) + trunk collider.


* **Build script (`tools/blender/build_biome_props.py`):**
* Calls `palette.write_palette_png_c()` only (never rewrites A or B), builds all 24 R31 biome models (9 desert, 8 snow/mountain, 7 swamp including `_lod1`/`_lod2`), writes `contact_sheet_biomes.png`, and exits 1 on any failure.


* **Caller / Lead actions:**
* Wait for R31-07 (`pine_snow*`, `rock_ice_01`, `snow_drift`, `snow_shrub`, `boulder_big`, `rock_cliff_01`) and R31-08 (`tree_swamp*`, `reeds`, `swamp_log_mossy`, `swamp_stump`, `lily_mud_patch`) scripts to land in `tools/blender/props/`.
* Run `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_biome_props.py > /tmp/r31-props.log 2>&1` once and inspect `assets/models/_previews/contact_sheet_biomes.png` and `/tmp/r31-props.log` (verify 0 `Traceback` and `BUILD OK`), then run `cargo run --release -p baker`.
* Fill `## Worker notes` in `tasks/R31-06-models-desert.md` if tracking on disk.


* **Known risks:**
* `tree_desert_dead` (+ LODs) exports with 2 materials (`palette` + `palette_c`), matching how existing sheet-B multi-material trees (e.g., `tree_autumn`) export. If any downstream pipeline step assumes single-material props for R31, switching `"bark_light"` to `"sandstone_dark"` or `"swamp_bark"` (sheet C) in those 3 files makes them single-material sheet C only.
