# R35-06: furniture inside town buildings

Owns: tools/blender/props/{int_table,int_chair,int_bed,int_shelf,int_stove,int_fridge,int_cabinet,int_rug}.py (new), world/src/towns/interior.rs (new), data/towns.json (an `interiors` block only; R35-03 adds `clutter`: edit different lines)
Files: tools/blender/props/*, world/src/towns/interior.rs, data/towns.json
Read only: world/src/towns/mod.rs, world/src/generator_contents.rs (`append_town_contents`), tools/blender/buildings/house_small.py (inside size, floor height, door side), tools/blender/props/crate_loot.py, common.py, palette.py, and the 8 walk-in town building scripts' size lines (adobe_house, desert_store, log_cabin_snow, research_station, ranger_station, stilt_shack, bait_shop, motel). Never run blender.

- 8 furniture props, 40-300 tris, real sizes (table 0.75 m high, bed 2 m long...).
- `interior::place(lot, data, seed) -> Vec<PropInstance>`: per building model in data, 3-6 pieces placed against inner walls in building-local space, rotated with the lot yaw, at the building's floor height, never blocking the door (keep a 1.2 m clear zone in front of it).
- Report to the lead the one line in `append_town_contents` that appends them.
- Tests: every piece is inside the footprint shrunk by 0.3 m; none in the door zone; deterministic.
