# R17-07: lake & coast models: docks, boathouse, fish shack, boats, beach props

Read first: `tasks/R17-common.md`, `data/lake.json` (docks 4 x 20 m, village lot sizes), `tools/blender/buildings/cabin.py` + `kit.py`, `tools/blender/props/car_wreck.py` (prop style).
Files: tools/blender/buildings/boathouse.py, tools/blender/buildings/boathouse_lod1.py, tools/blender/buildings/fish_shack.py, tools/blender/buildings/fish_shack_lod1.py, tools/blender/props/dock_*.py, tools/blender/props/rowboat*.py, tools/blender/props/beach_*.py, tools/blender/props/fishing_*.py, tools/blender/props/buoy.py, assets/models/buildings/boathouse*, assets/models/buildings/fish_shack*, assets/models/props/dock_*, assets/models/props/rowboat*, assets/models/props/beach_*, assets/models/props/fishing_*, assets/models/props/buoy*, assets/models/_previews/boathouse*, assets/models/_previews/fish_shack*, assets/models/_previews/dock_*, assets/models/_previews/rowboat*, assets/models/_previews/beach_*, assets/models/_previews/fishing_*, assets/models/_previews/buoy*, tasks/R17-07-lake-models.md

- `dock_section` (4 x 4 m weathered plank deck on posts, posts go 3 m down so it stands in water; the game repeats 5 sections for a 20 m dock) 400; `dock_end` (end piece with a bench and a lamp post) 600.
- Buildings (with `_lod1` like the city buildings, `lot_size` + `loot_N` + `spawn_N` empties, enterable): `boathouse` (14 x 10 m, half over the water edge, big door to the water side) 4000 / 500; `fish_shack` (8 x 6 m, nets, buoys on the wall, smoke stack) 2500 / 400.
- Props: `rowboat` (upright) 500, `rowboat_flipped` (on the beach) 400, `beach_umbrella` (broken) 300, `beach_chair` 250, `fishing_rack` (drying fish on a frame) 400, `buoy` 150.

## Worker notes
- Implemented all 12 lake and coast models and their Blender generation scripts:
  - `dock_section`: 4 x 4 m weathered plank deck on pilings extending 3 m down into water with underwater bracing, rope collars, stringers, and deck collider.
  - `dock_end`: 4 x 4 m dock end piece with weathered wooden bench, nautical lamp post with lantern, life ring, mooring bollards, and colliders.
  - `boathouse`: 14 x 10 m enterable building half over water edge, central open boat slip, flanking walkways, underwater pilings, big boat door to water side (+Y), player door on land side (-Y), workbench with tackle box and lantern, wall shelves with supplies, crates, barrels, life ring, empties (`lot_size` [14x10], `door_1`, `door_2`, `loot_1`..`loot_4`, `spawn_1`..`spawn_2`).
  - `boathouse_lod1`: 14 x 10 m outer shell matching roof shape, boat door opening, and pilings, with single body collider and `lot_size`.
  - `fish_shack`: 8 x 6 m enterable building with gable roof, exterior draped fishing nets, colorful hanging buoys, metal smoke stack pipe with rain cap, fish cleaning table, potbelly stove, crates, barrel, storage shelf, empties (`lot_size` [8x6], `door_1`, `loot_1`..`loot_3`, `spawn_1`..`spawn_2`).
  - `fish_shack_lod1`: 8 x 6 m outer shell with simplified smoke stack, net/buoy panels, body collider, and `lot_size`.
  - `rowboat`: 3.6 m upright wooden dinghy with continuous hull, gunwales, stem, thwarts/seats, pair of resting oars with blades and oarlocks, coiled painter rope, hull collider.
  - `rowboat_flipped`: inverted wooden dinghy hull resting on beach sand, keel ridge, propped by driftwood log, trailing painter rope, hull collider.
  - `beach_umbrella`: tilted pole in sand anchor base, 8-rib sun canopy with collapsed/drooping broken section and exposed bent rib.
  - `beach_chair`: folding wooden scissor frame with cross rails and blue canvas fabric sling.
  - `fishing_rack`: wooden A-frame drying rack with 3 horizontal drying poles, diagonal brace, 6 hanging dried fish, and wooden salting crate.
  - `buoy`: floating navigation marker buoy with red/white band, black bumper waist, metal superstructure, brass top beacon, and mooring eye.
- Triangle counts vs budgets:
  - `dock_section`: 364 / 400 tris (OK)
  - `dock_end`: 512 / 600 tris (OK)
  - `boathouse`: 3000 / 4000 tris (OK)
  - `boathouse_lod1`: 208 / 500 tris (OK)
  - `fish_shack`: 2108 / 2500 tris (OK)
  - `fish_shack_lod1`: 124 / 400 tris (OK)
  - `rowboat`: 213 / 500 tris (OK)
  - `rowboat_flipped`: 77 / 400 tris (OK)
  - `beach_umbrella`: 72 / 300 tris (OK)
  - `beach_chair`: 144 / 250 tris (OK)
  - `fishing_rack`: 348 / 400 tris (OK)
  - `buoy`: 132 / 150 tris (OK)
- All preview PNG images in `assets/models/_previews/` were inspected and verified.
- Known issues: none.
