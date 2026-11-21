# R17-02: camp upgrade models: towers, workshop stations, doctor, mechanic

Read first: `tasks/R17-common.md`, `data/camp_upgrades.json` (guards, workshop, traders lines), `tools/blender/buildings/watchtower.py`, `tools/blender/buildings/kit.py`, `tools/blender/buildings/settlement.py` (style).
Files: tools/blender/buildings/camp_tower_*.py, tools/blender/buildings/camp_workshop_*.py, tools/blender/buildings/camp_doctor.py, tools/blender/buildings/camp_mechanic.py, assets/models/buildings/camp_tower_*, assets/models/buildings/camp_workshop_*, assets/models/buildings/camp_doctor*, assets/models/buildings/camp_mechanic*, assets/models/_previews/camp_tower_*, assets/models/_previews/camp_workshop_*, assets/models/_previews/camp_doctor*, assets/models/_previews/camp_mechanic*, tasks/R17-02-camp-buildings.md

- `camp_tower_wood` (guard tower 6 m, ladder, platform with rails, a `guard_1` empty on the platform) 2500 tris; `camp_tower_metal` (scaffold tower 7 m, sandbags, a spotlight on the rail with a `light_1` empty) 3000 tris.
- `camp_workshop_1` workbench (table, vice, tools on a pegboard) 1500; `camp_workshop_2` tool bench (adds a grinder, drill press, metal shelf) 2500; `camp_workshop_3` machine shop (a small open shed 6x4 m with a lathe, welding set, engine on a stand) 4000. Each has an `interact_1` empty in front.
- `camp_doctor` medical tent 5x4 m (white canvas, red cross, cot, medicine shelf, a `npc_1` empty) 3000; `camp_mechanic` open garage 7x5 m (roof on posts, tool wall, tire stack, jack, oil barrels, `npc_1` and `car_spot` empties) 4000.

## Worker notes

