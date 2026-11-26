# R35-03: town clutter

Owns: tools/blender/props/{laundry_line,porch_chair,water_barrel,woodpile,sled,propane_tank,bench_wood,fire_barrel}.py (new), world/src/towns/clutter.rs (new), data/towns.json (a `clutter` block only)
Files: tools/blender/props/*, world/src/towns/clutter.rs, data/towns.json
Read only: world/src/towns/mod.rs, world/src/towns/lots.rs, world/src/generator_contents.rs (`append_town_contents`), tools/blender/props/barrel_rusty.py, crate_loot.py, common.py, palette.py. Never run blender.

- 8 props, 60-400 tris, palette colours.
- `clutter::place(town, data, seed) -> Vec<PropInstance>`: 2-5 pieces around each building lot (beside the door side, against walls, never inside a footprint, never on the main street or roads). Per-biome picks in data: desert `water_barrel`, `propane_tank`, `porch_chair`, `fire_barrel` + existing `barrel_rusty`, `car_wreck`, `tire_pile`; snow `woodpile`, `sled`, `bench_wood`; swamp `laundry_line`, `porch_chair`, `fire_barrel`, `rowboat_flipped`; mountains `woodpile`, `propane_tank`, `bench_wood`, `crate_loot`.
- Report to the lead the one line in `append_town_contents` that appends them (that file is the lead's).
- Tests: deterministic; no clutter inside a lot footprint or within 5 m of the street centre line.

## Worker notes
- **Files changed:**
  - `tools/blender/props/laundry_line.py`, `porch_chair.py`, `water_barrel.py`, `woodpile.py`, `sled.py`, `propane_tank.py`, `bench_wood.py`, `fire_barrel.py` (8 new prop scripts, 150–260 tris, palette colors, colliders).
  - `data/towns.json` (added `"clutter"` block with placement parameters and per-biome model lists for `desert`, `snow`, `swamp`, `mountains`).
  - `world/src/towns/clutter.rs` (new: `ClutterData`, `place(town, data, seed)`, `place_in_chunk(...)`, unit tests for determinism, lot-footprint exclusion, >= 5 m street-centre clearance, and biome model lists).
- **Shared-file lines for lead:**
  1. `world/src/towns/mod.rs`: add `pub mod clutter;` alongside `pub mod lots;` / `pub mod place;`.
  2. `world/src/generator_contents.rs` inside `append_town_contents` (`for town in &self.towns {` loop):
     ```rust
     props.extend(crate::towns::clutter::place_in_chunk(town, coord, self.seed, |x, z| self.height(x, z), |p| self.roads.distance_to_road(p)));
     ```
- **How to see it:** Run `blender -b --python tools/blender/build_detail_props.py` (once task 02 adds it), bake `assets/models.pack`, and take shots of `--town=dusthollow`, `--town=frostmere`, etc.
- **Known issues / skipped:** `cargo`, `tools/gate.sh`, and `blender` were not run (subagent rule); Python syntax checked via `py_compile`.

