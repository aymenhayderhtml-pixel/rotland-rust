# R40-02: Overgrowth models (Blender: vines, weeds, trees, shrubs, moss)

Owns: tools/blender/overgrowth/ (new folder: one script per group, plus `overgrowth.py` with `build_all()`), assets/models/nature/ (only the new `vine_*`, `weeds_crack`, `tree_asphalt`, `sapling_city`, `shrub_wild`, `grass_tall_patch`, `moss_patch`, `roof_bush`, `roof_tree`, `leaves_litter` .glb files), assets/models/_previews/ (only their PNGs and `_top` PNGs). Not owned: every other file. The shared build script `tools/blender/build_r40_models.py` belongs to R40-05; add ONE line for this task in your Worker notes ("import overgrowth; ('overgrowth', overgrowth.build_all)" for its GROUPS list).
Read: tasks/R40-common.md ("R40 shared names", overgrowth bullets, "Owner decisions"), docs/ART_PIPELINE.md, tools/blender/common.py (`Model`, `add`, `add_collider`, `finalize_and_export`, `make_frustum`, `make_icosphere`, `make_box`, `paint_faces`, `run_model`), tools/blender/props/tree_birch.py and tree_broadleaf_01.py (tree shape, trunk and leaf clusters), tools/blender/props/bush_01.py, tools/blender/props/grass_tuft_a.py, tools/blender/props/swamp_moss_clump.py (moss look), tools/blender/towers/brick_parts.py (fire escape and window AC units: how the R39 kit writes thin parts), tools/blender/palette.py (`COLORS`, `COLORS_C`: use `leaf_dark`, `leaf_mid`, `leaf_light`, `pine_mid`, `grime_green`, `swamp_moss`, `bark_dark`, `bark_light` for dead leaves, `rock_dark`, `dirt`, `concrete_grey`). Budgets: data/budgets.json (`nature` 15000 per model).

## Origin and orientation (all models)
- Upright models: origin at the bottom centre of the footprint, Z=0 is the ground.
- Wall sheets (`vine_*`): the sheet is in the XZ plane, facing +Y (the front). Its back edge touches the wall at Y=0; the leaves sit at Y +0.02 to +0.05. Origin = bottom centre at the wall. R40-01 places them with the facade yaw, so the sheet faces out from the wall.
- Ground pieces (`weeds_crack`, `leaves_litter`, `grass_tall_patch`, `moss_patch`): lie on the ground, lowest point Z=0.01 to 0.02 so they never z-fight the road.

## Blender run and checks
- Run: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/overgrowth/overgrowth.py`. The subagent MAY run Blender this way (owner decision for R40). Read the log: no `Traceback`, no `FAILED`.
- Look at EVERY preview in `assets/models/_previews/` (3/4 view). If a model reads as a green blob, a grey box or a floating part, fix the script and run again.
- Y-up export, 1 unit = 1 m, origin bottom centre (the `Model` default), apply transforms (done by `Model`).

## Shapes
- **Vine sheets** (`vine_wall_s` 4 m tall, `vine_wall_m` 8 m, `vine_wall_l` 14 m; all 0.3 m wide and 0.3 m deep; ground origin, bottom centre): a leafy sheet that is flat against a facade. The leaf sheet sits at y = +0.02 to +0.05 (2-5 cm in front of the wall, +Y is the front), so it never z-fights the facade. Leaves are small flat diamonds in clusters (`leaf_dark`, `leaf_mid`, a few `leaf_light`), thick near the bottom and thinning with height, with 3-5 hanging strands (thin `leaf_dark` boxes, 0.6-1.5 m long) falling from the top third. Some gaps show `rock_dark` wall through the leaves (a 5% share of diamonds in `rock_dark`), so it does not look like a flat green wall.
- **vine_cornice**: ivy draped over a 3 m ledge: a sheet that hangs over the edge (1.2 m down) with strands, same colours.
- **weeds_crack**: a line of 3 m long weeds: 10-14 tufts (each 3-6 blades as thin boxes or flat leaves), `leaf_mid` and `grass` tones, a few `dirt` clods at the base. Lies flat on the ground: tufts stand up to 0.4 m.
- **tree_asphalt**: a young tree (about 4-5 m tall) breaking out of a raised asphalt slab (a 1.6 x 1.6 m, 0.15 m thick `rock_dark` slab that is cracked: 2 `bark_dark` crack lines are fine). The trunk comes out of the slab hole; the slab is part of the model. Crown: 4-6 leaf icospheres (`leaf_mid`, `leaf_dark`).
- **sapling_city**: a 1.5-2 m sapling from a dirt patch (in a broken pavement hole).
- **shrub_wild**: a 1.5-2 m wild shrub (lumpy icospheres, `leaf_dark` with `pine_mid` and dead `bark_light` twigs poking out).
- **grass_tall_patch**: a 1.2 m patch of tall dead-green grass blades (`grass`, `dirt` at the roots), about 2 x 2 m footprint.
- **moss_patch**: a flat 1 x 1 m moss mat, 1 cm thick, `swamp_moss` and `grime_green`, for a wall or a car roof.
- **roof_bush**: a bush (about 1.5 m) with a `bark_dark` stub, meant to sit on a flat roof, no collider.
- **roof_tree**: a small tree 3-4 m tall for a roof (short trunk 0.6 m, round crown), `leaf_dark` and `leaf_mid`.
- **leaves_litter**: a flat 1.5 x 1.5 m litter of 25-40 small dead leaves (`bark_light`, `dirt`) and 4-6 paper scraps (`white`, `concrete_grey`), lying 1-2 cm above ground.

## Budgets (triangles, LOD0 only; checked by Model)
- vine_wall_s 1500, vine_wall_m 3000, vine_wall_l 6000, vine_cornice 1500, weeds_crack 800, tree_asphalt 6000, sapling_city 1500, shrub_wild 2000, grass_tall_patch 1500, moss_patch 600, roof_bush 2000, roof_tree 6000, leaves_litter 800.
- Each model is one visual mesh. No colliders on any model (the player walks through leaves and weeds). `tree_asphalt` and `roof_tree` get a small trunk collider only if you need it for the city; default: NO colliders (say so in notes).
- Every number in the scripts gets `# new: DECISIONS 2026-11-26` (sizes) or `# data: budgets.json nature`.

## Done when
- All 13 models above exist as `assets/models/nature/<name>.glb`, each within its budget, with no `Traceback` or `FAILED` in the log.
- Every preview looks dense and believable: vines read as ivy, weeds in a crack, a tree breaking a slab, moss on a mat.
- Vine and decal sheets have no flat-grey gaps and no z-fighting in the preview (zoom in on one).
- `tools/blender/overgrowth/` has `overgrowth.py` with `build_all()` returning `[(name, ok)]` like the R39 scripts.

## Worker notes
