# R20-09: stop the flicker in props (street lamps, bus stop, car wrecks, tire pile...)

Read first: `tasks/R20-common.md`, `tasks/R19-08-zfight-camp-lake.md` (same job and rules), `tools/blender/common.py` (read only).
Files: tools/blender/props/barricade.py, tools/blender/props/bus_stop.py, tools/blender/props/car_wreck_sedan.py, tools/blender/props/car_wreck_van.py, tools/blender/props/dock_end.py, tools/blender/props/dock_section.py, tools/blender/props/dumpster.py, tools/blender/props/jersey_barrier.py, tools/blender/props/mailbox.py, tools/blender/props/road_sign.py, tools/blender/props/sign_fallen.py, tools/blender/props/street_lamp.py, tools/blender/props/stump.py, tools/blender/props/tire_pile.py, assets/models/props/*, assets/models/_previews/*, tasks/R20-09-zfight-props.md
Test: (Blender lane: no cargo)

Checker today (`python3 -I tools/find_coplanar.py assets/models/props/<name>.glb`), m2: car_wreck_van 1.87, tire_pile 1.56, car_wreck_sedan 0.82, bus_stop 0.47, road_sign 0.42, street_lamp 0.38, jersey_barrier 0.34, dumpster 0.33, mailbox 0.32, sign_fallen 0.29, barricade 0.17, stump 0.09, dock_end 0.08, dock_section 0.06. Biggest first. Goal: under 0.05 m2 each. Only rebuild the props you edit (don't commit other glbs/previews that changed by accident: `git checkout --` them; NOT rowboat, another lane owns it). Same rules: fix, rebuild, re-check, look at the preview; undo a model that got worse; before/after in Worker notes; commit as you go.

## Worker notes

### Summary of fixes
Resolved coplanar z-fighting across all 14 models targeted by this task. Every single model is well under the 0.05 m² threshold, with 8 models achieving a perfect 0.00 m² (OK) result.

| Model | Initial Area | Final Area | Notes / Fix |
|---|---|---|---|
| `car_wreck_van` | 1.87 m² | **0.02 m²** | Thinned door rust decal (4 mm proud of door); partitioned cabin bulkhead to prevent overlap with side pillars; recessed rims inside tires; trimmed rear doors width & headlight fascia |
| `tire_pile` | 1.56 m² | **0.00 m² (OK)** | Rebuilt tyre geometry into a clean single manifold bmesh with recessed cavity; offset vertical tyre stack levels by 2 cm |
| `car_wreck_sedan` | 0.82 m² | **0.00 m² (OK)** | Offset door panel proud of rocker sill by 2 cm; recessed wheel rims inside tire geometry; lifted floorpan up by 2 cm off SILL_Z; stepped headlights |
| `bus_stop` | 0.47 m² | **0.01 m²** | Post shafts now stop at top of footing z=0.08 rather than sharing ground z=0; timetable poster layers stepped forward with 1-2 cm gaps; crossbeams trimmed to fit between posts |
| `road_sign` | 0.42 m² | **0.00 m² (OK)** | Reduced white face plate thickness to 0.02 m so its back face does not coplanar with back plate at y=+0.015 |
| `street_lamp` | 0.38 m² | **0.01 m²** | Widened lower pole rust band sleeve from 0.122/0.118 to 0.132/0.128 m; separated luminaire bottom levels and glass frame/intact steps |
| `jersey_barrier` | 0.34 m² | **0.00 m² (OK)** | Removed redundant -Z ground face from `notch1` and `notch2` forklift slots |
| `dumpster` | 0.33 m² | **0.00 m² (OK)** | Trimmed container floor and front/back walls to width 1.72 m inside side quads; lifted skids to z=0.03 atop caster pads; stepped front ribs/rust patch; adjusted rim collar Y |
| `mailbox` | 0.32 m² | **0.01 m²** | Adjusted base rim collar to z=0.31-0.37 avoiding body bottom at 0.34; leg posts start at z=0.025 on foot pads; rust corner stepped out by 2 cm; decals layered cleanly |
| `sign_fallen` | 0.29 m² | **0.00 m² (OK)** | Stepped plate layers on octagonal stop sign by >= 12 mm (`stop_back` z: 0.0-0.015, `stop_white` z: 0.015-0.028, `stop_inner` z: 0.028-0.042, bracket & holes offset) |
| `barricade` | 0.17 m² | **0.00 m² (OK)** | Offset warning sign proud of rails and decals proud of sign face; staggered sandbag course heights and rotations; sank spilled dirt base to z=-0.01 and lifted burst bag to z=0.01 |
| `stump` | 0.09 m² | **0.00 m² (OK)** | Sank root bases to z=-0.01 so root bottom caps do not coplanar with trunk bottom cap at z=0 |
| `dock_end` | 0.08 m² | **0.00 m² (OK)** | Trimmed deck planks to width 3.88 m to fit inside rub rails; raised rub rail curb profile to z=0.11; trimmed end bumper to 3.80 m; separated bench legs, braces, and slats |
| `dock_section` | 0.06 m² | **0.00 m² (OK)** | Trimmed deck planks to width 3.88 m to fit inside rub rails; raised rub rail curb profile to z=0.11 proud of planks |

### Verification
Run `python3 -I tools/find_coplanar.py assets/models/props/{car_wreck_van,tire_pile,car_wreck_sedan,bus_stop,road_sign,street_lamp,jersey_barrier,dumpster,mailbox,sign_fallen,barricade,stump,dock_end,dock_section}.glb`
All 14 models pass:
- `car_wreck_van.glb`: 0.02 m²
- `tire_pile.glb`: 0.00 m² (OK)
- `car_wreck_sedan.glb`: 0.00 m² (OK)
- `bus_stop.glb`: 0.01 m²
- `road_sign.glb`: 0.00 m² (OK)
- `street_lamp.glb`: 0.01 m²
- `jersey_barrier.glb`: 0.00 m² (OK)
- `dumpster.glb`: 0.00 m² (OK)
- `mailbox.glb`: 0.01 m²
- `sign_fallen.glb`: 0.00 m² (OK)
- `barricade.glb`: 0.00 m² (OK)
- `stump.glb`: 0.00 m² (OK)
- `dock_end.glb`: 0.00 m² (OK)
- `dock_section.glb`: 0.00 m² (OK)

### Scope & Constraints
- `python3 -I tools/gate_checks.py --base main --task tasks/R20-09-zfight-props.md` -> `scope: PASS (29 changed paths)`.
- No cargo was run.
- No unrelated models modified (`rowboat` untouched).
- Known issues: None.
