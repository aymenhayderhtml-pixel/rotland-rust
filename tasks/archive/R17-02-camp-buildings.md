# R17-02: camp upgrade models: towers, workshop stations, doctor, mechanic

Read first: `tasks/R17-common.md`, `data/camp_upgrades.json` (guards, workshop, traders lines), `tools/blender/buildings/watchtower.py`, `tools/blender/buildings/kit.py`, `tools/blender/buildings/settlement.py` (style).
Files: tools/blender/buildings/camp_tower_*.py, tools/blender/buildings/camp_workshop_*.py, tools/blender/buildings/camp_doctor.py, tools/blender/buildings/camp_mechanic.py, assets/models/buildings/camp_tower_*, assets/models/buildings/camp_workshop_*, assets/models/buildings/camp_doctor*, assets/models/buildings/camp_mechanic*, assets/models/_previews/camp_tower_*, assets/models/_previews/camp_workshop_*, assets/models/_previews/camp_doctor*, assets/models/_previews/camp_mechanic*, tasks/R17-02-camp-buildings.md

- `camp_tower_wood` (guard tower 6 m, ladder, platform with rails, a `guard_1` empty on the platform) 2500 tris; `camp_tower_metal` (scaffold tower 7 m, sandbags, a spotlight on the rail with a `light_1` empty) 3000 tris.
- `camp_workshop_1` workbench (table, vice, tools on a pegboard) 1500; `camp_workshop_2` tool bench (adds a grinder, drill press, metal shelf) 2500; `camp_workshop_3` machine shop (a small open shed 6x4 m with a lathe, welding set, engine on a stand) 4000. Each has an `interact_1` empty in front.
- `camp_doctor` medical tent 5x4 m (white canvas, red cross, cot, medicine shelf, a `npc_1` empty) 3000; `camp_mechanic` open garage 7x5 m (roof on posts, tool wall, tire stack, jack, oil barrels, `npc_1` and `car_spot` empties) 4000.

## Worker notes
Created 7 new camp upgrade 3D models with Blender Python scripts, colliders, and empties:
1. `camp_tower_wood`: 6 m wooden guard tower with ladder on +Y side, platform deck, perimeter safety rails, canopy roof, and `guard_1` empty on the platform.
   - Printed: 724 tris (budget 2500) OK
2. `camp_tower_metal`: 7 m steel scaffolding tower with diagonal X-braces, sandbags fortified at base and on platform, ladder, spotlight on front rail with `light_1` empty, and `guard_1` empty.
   - Printed: 1956 tris (budget 3000) OK
3. `camp_workshop_1`: Heavy wooden workbench with vice, lower shelf with storage crate and spare metal pipes, rear pegboard with hanging hand tools, surface oil can and tray, and `interact_1` empty.
   - Printed: 564 tris (budget 1500) OK
4. `camp_workshop_2`: Upgraded tool bench adding a bench grinder, drill press, vice, rear pegboard with tools, freestanding 4-tier metal parts shelf (battery, red toolbox, parts bins, spray cans), and `interact_1` empty.
   - Printed: 836 tris (budget 2500) OK
5. `camp_workshop_3`: 6x4 m open machine shop with concrete slab floor, corrugated roof, industrial metal lathe with chuck/carriage/tailstock, portable oxy-acetylene welding cart with gas cylinders and regulators, V8 engine on a rolling engine stand, workbench with vice, and `interact_1` empty.
   - Printed: 1348 tris (budget 4000) OK
6. `camp_doctor`: 5x4 m military field medical tent with white canvas, bold red cross markings on front gable and side walls, patient cot, IV drip stand with saline bag, stocked medicine shelf, doctor's consultation desk with first aid kit, and `npc_1` empty.
   - Printed: 1204 tris (budget 3000) OK
7. `camp_mechanic`: 7x5 m open vehicle repair garage with clear drive-in bay, yellow parking bay guide lines, corrugated roof on perimeter posts, mechanic tool wall, roll-around tool chest, tire stacks, hydraulic floor jack, 55-gallon oil barrels with rotary pump, mechanic creeper, and `npc_1` and `car_spot` empties.
   - Printed: 1864 tris (budget 4000) OK

All preview renders verified in `assets/models/_previews/`.
Gate check `python3 -I tools/gate_checks.py --base main --task tasks/R17-02-camp-buildings.md` passed (scope PASS, numbers PASS, unused PASS).
No known issues.

