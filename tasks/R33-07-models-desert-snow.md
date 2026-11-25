# R33-07: desert and snow town buildings + the town buildings build script

Owns: tools/blender/buildings/{adobe_house,motel,water_tower,desert_store,log_cabin_snow,research_station}.py (new), tools/blender/buildings/build_town_buildings.py (new)
Files: tools/blender/buildings/adobe_house.py, tools/blender/buildings/motel.py, tools/blender/buildings/water_tower.py, tools/blender/buildings/desert_store.py, tools/blender/buildings/log_cabin_snow.py, tools/blender/buildings/research_station.py, tools/blender/buildings/build_town_buildings.py
Read: tasks/R33-round.md, tasks/LEAD.md, docs/WORLD.md, tools/blender/buildings/cabin.py, tools/blender/buildings/house_small.py, tools/blender/buildings/gas_station.py, tools/blender/buildings/kit.py, tools/blender/buildings/watchtower.py, tools/blender/buildings/build_buildings.py, tools/blender/common.py, tools/blender/palette.py
Overlap: `build_town_buildings.py` names the 4 R33-08 buildings (`radio_tower`, `ranger_station`, `stilt_shack`, `bait_shop`) in its BUILDINGS list. Write only the list entries. R33-08 writes their scripts.
Rules: `tasks/LEAD.md` subagent rules (never run blender). Use the "R33 shared names" in `tasks/R33-round.md` EXACTLY. Copy the style and helpers of `tools/blender/buildings/cabin.py`, `house_small.py`, `gas_station.py`, `kit.py` and `tools/blender/common.py`. Colours: palette sheets A, B, C (`tools/blender/palette.py`; sheet C has the sand/snow colours). Read how buildings get doors/openings and interiors today (can the player walk in? copy exactly what `house_small` does: same door size, floor height, wall thickness), so loot spots and colliders work the same way.

**`build_town_buildings.py`:** like `build_buildings.py`, but `BUILDINGS` = ONLY the 10 new R33 buildings from the shared names (yours plus R33-08's: `radio_tower`, `ranger_station`, `stilt_shack`, `bait_shop`). It must never rebuild old buildings.

Models (low-poly, flat palette colours, the same scale as `house_small`; tri budgets like similar existing buildings):
- `adobe_house`: a one-storey flat-roof mud house, sand_light/sand_dark walls with rounded corners (bevel), wooden roof beams (vigas) sticking out, a small doorway, two small windows. Walk-in like house_small.
- `motel`: a long one-storey building with 5 doors in a row, a covered walkway on posts, a tall pole sign ("MOTEL" as raised boxes or just a sign board).
- `water_tower`: a wooden tank on 4 legs with cross braces and a ladder, 10 m tall. Not enterable.
- `desert_store`: a general store with a false front (taller flat facade), a porch with posts, faded sandstone colours. Walk-in.
- `log_cabin_snow`: a log cabin (stacked log look via alternating bark faces) with a steep roof under thick snow (snow_white roof slab + a snow edge overhang) and a chimney. Walk-in.
- `research_station`: 2-3 connected box modules on short stilts, metal panels (white/grey), a small antenna and a dish, a door with steps. Walk-in for at least one module.

Each script has `main()` returning 0/1 via the same runner the other buildings use. Add an `_lod1` only if the similar existing building has one (e.g. if `house_small` has none, don't make one).

## Worker notes
