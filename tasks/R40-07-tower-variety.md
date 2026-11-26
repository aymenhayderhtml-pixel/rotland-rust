# R40-07: Tower variety (damaged floors, better crowns, roof props)

Owns: tools/blender/towers/ (new: `damaged_parts.py` (shared damage helpers), `roof_props.py` (the four roof props), `towers_r40.py` (`build_all()` for this task); edits to the five style scripts `brick.py`, `stone.py`, `glass.py`, `slab.py`, `deco.py` ONLY to ADD a `build_r40_all()` function and to change their crown builders; edits to `roof_parts.py` only to add new helpers), assets/models/buildings/ (only the new `tw_<style>_<size>_floor_c` and `_floor_c_lod1`, the rebuilt `_crown`/`_crown_lod1`, and the four `roof_*` .glb files), assets/models/_previews/ (only their PNGs), world/src/metro/tower_parts.rs, world/src/metro/tower_parts_tests.rs (NEW, the tests for this task), world/src/metro/config.rs (ONLY the new `damaged_share` field in `TowersConfig`), data/metro.json (ONLY the key `towers.damagedShare`). Lines for the lead: the `mod tower_parts_tests;` line in world/src/metro/mod.rs; the one line for `build_r40_models.py` (`("towers_r40", towers_r40.build_all)`) goes in your Worker notes (R40-05 owns that script).
Not owned: `brick_parts.py`, `stone_parts.py` (read them only), base and `_floor`/`_floor_b` models (do NOT rebuild or change them), `shell.py`, `landmarks.py`, `lobby_props*.py`, `stack_preview.py`, `build_r39_models.py`, every other file.
Read: tasks/R40-common.md ("Tower variety", "R40 shared names", "Owner decisions"), tasks/archive/R39-02-kit-brick-stone.md and R39-03/04 (the rules block: origin, exact heights, slab bands, LOD1, budgets: `floor` 16 m 3000/300, 24 m 4500/450, 32 m 6000/600; crown 8000/800), tools/blender/towers/shell.py, tools/blender/towers/roof_parts.py (`water_tank`, `ac_unit`, `vent_stack`, `antenna`, `roof_deck`), tools/blender/towers/brick.py, stone.py, glass.py, slab.py, deco.py (each `build_all` and its floor/crown builders), tools/blender/towers/stack_preview.py (stack check), world/src/metro/tower_parts.rs (`tower_parts`, `PartKind`, the floor alternation by seed), world/src/metro/towers.rs (`TowerSpot`), world/src/metro/config.rs (`TowersConfig`), data/metro.json (`styles`: sizes per style).

## Origin and orientation
- Every tower module (`_floor_c`, crown, roof props): origin at the bottom centre of the footprint, Z=0 at the module bottom, front = +Y, `grip=(0,0,0)` as the R39 kit does. Overhangs at most 1.2 m on +Y.

## Blender run and checks
- Run (the subagent MAY run Blender this way): `flock /tmp/rotland-blender.lock blender -b --python tools/blender/towers/towers_r40.py`. Read the log: no `Traceback`, no `FAILED`. Look at every preview PNG (`_floor_c` next to `_floor`, every crown, every roof prop); fix and run again if needed.

## The list (from data/metro.json `styles`)
- brick: 16, 24. stone: 24, 32. glass: 24, 32. slab: 24, 32. deco: 24, 32. = 10 size/style pairs.
- Each pair gets `tw_<style>_<size>_floor_c` and `tw_<style>_<size>_floor_c_lod1` (20 files). Crown is rebuilt for each pair (10 `_crown` and 10 `_crown_lod1`). Landmarks are not changed.

## Blender: damaged floor (`_floor_c`)
- Same footprint, same height (3.8 m, slab bands at top and bottom), same window grid as the style's `_floor`, so stacks have no gap and it is the same size.
- Damage: 25-40% of panes are dark `glass_dark` or missing (a hole shows the inside `soot` box), 2-4 shattered panes (a thin `glass_dark` diagonal cross), a missing window frame or two (a bare hole with a torn `metal` edge), soot streaks (`soot`) running down from the top of 3-4 openings, rust streaks (`rust_dark`) under sills, and a torn corner piece of facade (the wall at one corner is 0.3 m shorter, showing the `soot` inside).
- Helpers go in `damaged_parts.py` (for example `shattered_pane`, `soot_streak`, `rust_streak`, `missing_frame`): one set for all styles, so all five style scripts call the same helpers.
- `_floor_c_lod1`: same silhouette and colours, plain boxes with 2-3 dark recesses, about 1/10 of LOD0 triangles, no colliders.
- Budgets: as the R39 floor budgets above (3000 / 4500 / 6000 LOD0, 300 / 450 / 600 LOD1).

## Blender: better crowns and roof props
- Rebuild each style's `_crown` and `_crown_lod1` (same height: brick 3.5, stone 5.0, glass 4.5, slab 3.0, deco 5.5 m; same footprint). Crown roofs get: a bigger water tank on steel legs (the New York wooden-barrel tank, built by `roof_props.roof_water_tank_ny` and placed 1-2 times), rooftop machinery (`roof_machinery`: a compressor block with pipes and a fan grille), 1 stair hut (`roof_stair_hut`: a small box with a sloped roof and a door), 1-2 antennas (`roof_antenna`: a lattice mast, 3-5 m), AC boxes (existing `ac_unit`), and a vent stack. Keep every crown inside its footprint and its height.
- The four roof props are ALSO standalone glbs in `assets/models/buildings/` (`roof_water_tank_ny`, `roof_machinery`, `roof_stair_hut`, `roof_antenna`, each 1500 triangles or less, origin bottom centre) so R40-08 can place them on low roofs and crowns. The copies inside the crowns are built in the crown model itself.

## Rust: `_floor_c` in the pick (world/src/metro/tower_parts.rs)
- `towers.damagedShare` (data, 0.25) in `TowersConfig` as `damaged_share: f32` (`#[serde(default)]`, value 0.25).
- Floor pick: when a floor is placed, with the same seeded RNG, a share of floors becomes `_floor_c` (the size is the tower's size). Keep the existing `_floor` / `_floor_b` alternation for the rest.
- New function `pub fn tower_parts_with_damage(spot, style, ground_y, rng_seed, damage: DamageMode) -> Vec<TowerPart>` where `pub enum DamageMode { Share(f32), All }`. `tower_parts` keeps its R39 signature and calls it with `Share(config value)`. Burnt blocks (R40-03) will call it with `All`. Doc comment on it says that `All` is for the burnt block.
- Tests (in `tower_parts_tests.rs`): with `Share(0.25)` over many seeds, the `_floor_c` share is between 15% and 35%; with `All`, every floor is `_floor_c`; floors stack gap-free (same `pos.y` steps as before); a `_floor_c` has the same height as the style's `_floor`; `Share(0.0)` gives no `_floor_c`; the R39 tests (same seed identical, different seed varies `_floor`/`_floor_b`) still pass.

## Done when
- The 20 `_floor_c` files and 10 rebuilt crown pairs and 4 roof props exist, all within budget, with no `Traceback` or `FAILED` (run `build_r40_models.py` once via `towers_r40.build_all()`).
- Look at every `_floor_c` preview next to its `_floor` (same size, same grid, clearly more broken). Look at each crown preview: roof props read at a glance.
- Stack check: `stack_preview.py` for brick 16 with a `_floor_c` in the middle shows no gap.
- The Rust tests are written; they are run by the lead's one build and test (do not run cargo yourself). `world/tests/golden.rs` passes unedited.

## Worker notes
