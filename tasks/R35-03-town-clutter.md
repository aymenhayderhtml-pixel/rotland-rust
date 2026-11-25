# R35-03: town clutter

Owns: tools/blender/props/{laundry_line,porch_chair,water_barrel,woodpile,sled,propane_tank,bench_wood,fire_barrel}.py (new), world/src/towns/clutter.rs (new), data/towns.json (a `clutter` block only)
Files: tools/blender/props/*, world/src/towns/clutter.rs, data/towns.json
Read only: world/src/towns/mod.rs, world/src/towns/lots.rs, world/src/generator_contents.rs (`append_town_contents`), tools/blender/props/barrel_rusty.py, crate_loot.py, common.py, palette.py. Never run blender.

- 8 props, 60-400 tris, palette colours.
- `clutter::place(town, data, seed) -> Vec<PropInstance>`: 2-5 pieces around each building lot (beside the door side, against walls, never inside a footprint, never on the main street or roads). Per-biome picks in data: desert `water_barrel`, `propane_tank`, `porch_chair`, `fire_barrel` + existing `barrel_rusty`, `car_wreck`, `tire_pile`; snow `woodpile`, `sled`, `bench_wood`; swamp `laundry_line`, `porch_chair`, `fire_barrel`, `rowboat_flipped`; mountains `woodpile`, `propane_tank`, `bench_wood`, `crate_loot`.
- Report to the lead the one line in `append_town_contents` that appends them (that file is the lead's).
- Tests: deterministic; no clutter inside a lot footprint or within 5 m of the street centre line.
