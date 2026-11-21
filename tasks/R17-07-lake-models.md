# R17-07: lake & coast models: docks, boathouse, fish shack, boats, beach props

Read first: `tasks/R17-common.md`, `data/lake.json` (docks 4 x 20 m, village lot sizes), `tools/blender/buildings/cabin.py` + `kit.py`, `tools/blender/props/car_wreck.py` (prop style).
Files: tools/blender/buildings/boathouse.py, tools/blender/buildings/boathouse_lod1.py, tools/blender/buildings/fish_shack.py, tools/blender/buildings/fish_shack_lod1.py, tools/blender/props/dock_*.py, tools/blender/props/rowboat*.py, tools/blender/props/beach_*.py, tools/blender/props/fishing_*.py, tools/blender/props/buoy.py, assets/models/buildings/boathouse*, assets/models/buildings/fish_shack*, assets/models/props/dock_*, assets/models/props/rowboat*, assets/models/props/beach_*, assets/models/props/fishing_*, assets/models/props/buoy*, assets/models/_previews/boathouse*, assets/models/_previews/fish_shack*, assets/models/_previews/dock_*, assets/models/_previews/rowboat*, assets/models/_previews/beach_*, assets/models/_previews/fishing_*, assets/models/_previews/buoy*, tasks/R17-07-lake-models.md

- `dock_section` (4 x 4 m weathered plank deck on posts, posts go 3 m down so it stands in water; the game repeats 5 sections for a 20 m dock) 400; `dock_end` (end piece with a bench and a lamp post) 600.
- Buildings (with `_lod1` like the city buildings, `lot_size` + `loot_N` + `spawn_N` empties, enterable): `boathouse` (14 x 10 m, half over the water edge, big door to the water side) 4000 / 500; `fish_shack` (8 x 6 m, nets, buoys on the wall, smoke stack) 2500 / 400.
- Props: `rowboat` (upright) 500, `rowboat_flipped` (on the beach) 400, `beach_umbrella` (broken) 300, `beach_chair` 250, `fishing_rack` (drying fish on a frame) 400, `buoy` 150.

## Worker notes

