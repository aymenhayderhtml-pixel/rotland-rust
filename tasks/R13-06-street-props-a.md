# R13-06: street props a for the city

Read first: `tasks/R13-common.md`, `docs/CITY_SPEC.md` section 5, `tools/blender/props/car_wreck.py`, `tools/blender/props/road_sign.py`, `tools/blender/common.py` (only the functions you use).
Files: tools/blender/props/street_lamp.py, tools/blender/props/bus_stop.py, tools/blender/props/mailbox.py, tools/blender/props/dumpster.py, tools/blender/props/trash_bags.py, tools/blender/props/barricade.py, assets/models/props/street_lamp*, assets/models/props/bus_stop*, assets/models/props/mailbox*, assets/models/props/dumpster*, assets/models/props/trash_bags*, assets/models/props/barricade*, assets/models/_previews/street_lamp*, assets/models/_previews/bus_stop*, assets/models/_previews/mailbox*, assets/models/_previews/dumpster*, assets/models/_previews/trash_bags*, assets/models/_previews/barricade*, tasks/R13-06-*.md

One script per prop, category `props`, low-poly, flat colors, ruined look. No LOD files needed. Base on the ground at z=0, real-world size.

| Model | What | Triangle budget |
|---|---|---|
| street_lamp | bent metal street lamp, 6 m pole, dead light head | 800 |
| bus_stop | bus shelter: roof on posts, back panel, a bench; one panel broken | 1500 |
| mailbox | blue street mailbox | 400 |
| dumpster | green metal dumpster with lid half open | 800 |
| trash_bags | pile of 4-5 black trash bags (lumpy icospheres) | 600 |
| barricade | police/army road barricade: wooden sawhorses plus a sandbag row, about 4 m wide | 1500 |

## Worker notes
- Implemented all 6 low-poly street props with individual Blender scripts in `tools/blender/props/`.
- All models use real-world scales, grounded at z=0, flat shading, low-poly aesthetics, and ruined post-apocalyptic detailing.
- Colors were strictly selected from `tools/blender/palette.py` (Sheets A and B), including `metal`, `rust`, `gun_black`, `brass`, `glass_tint`, `wood_crate`, `car_blue`, `pine_dark`, `canvas_khaki`, `cloth_olive`, `tire_black`, `pants_dark`, `red_paint`, `white`, and `dirt`.
- Triangle counts vs budgets:
  | Model | Printed Triangles | Budget | Status |
  |---|---|---|---|
  | `street_lamp` | 448 | 800 | OK |
  | `bus_stop` | 692 | 1500 | OK |
  | `mailbox` | 318 | 400 | OK |
  | `dumpster` | 288 | 800 | OK |
  | `trash_bags` | 552 | 600 | OK |
  | `barricade` | 980 | 1500 | OK |

- Details of props created:
  - `street_lamp`: 6m pole with octagonal base collar, bolts, rusted inspection hatch with loose cover and dark opening, dented mid-pole, curved outreach arm with rust band and support strut, cobra-head housing with unlit glass and broken cavity with socket/bulb/dangling wire, and box collider.
  - `bus_stop`: 3.6m x 1.6m x 2.4m passenger shelter on steel posts with rusted footings, horizontal rails, forward-sloping canopy roof with fascia trim, two intact tinted glass panels with schedule poster, one shattered rear glass bay with jagged edge shards and debris on the ground, intact side panel, wooden bench with backrest and steel legs, bus route sign on front-left post, and colliders for back wall, left wall, roof, and bench.
  - `mailbox`: USPS-style blue street mailbox on 4 steel legs with footpads, `car_blue` body with carrier door, lock, schedule placard with red stripe, arched hood along Y, angled drop visor, pull handle, dark drop cavity, corner rust, and full box collider.
  - `dumpster`: 4-yard alley dumpster in `pine_dark` green with trapezoidal slanted side walls, bottom steel runners and pads, forklift side sleeves, front ribs, interior trash pile with cardboard box and bag, hinge rod along rear rim, closed left lid with handle, half-open right lid tilted up ~55 degrees revealing interior, and box collider.
  - `trash_bags`: Clustered pile of 5 slumped lumpy trash bags (subdivided icospheres with jitter and squish in `gun_black`, `tire_black`, and `pants_dark`), tied neck knots with tabs, flattened ground contact, crushed cardboard carton and tin can debris, and pile collider.
  - `barricade`: 4.0m road barricade with two wooden A-frame sawhorses (left upright, right skewed), two 4.0m rails with alternating red/white warning stripes, splintered rail end, central metal warning sign plate, 3-course tiered sandbag wall (~20 bags in `canvas_khaki`, `cloth_olive`, `shirt_zombie`), burst sandbag with spilled dirt pile, and barrier box collider.

- Visual inspection:
  - Rendered all previews via Workbench into `assets/models/_previews/` and visually verified each image (`street_lamp.png`, `bus_stop.png`, `mailbox.png`, `dumpster.png`, `trash_bags.png`, `barricade.png`). Fixed initial placement offsets for visor and roof trim to ensure all parts are cleanly positioned and connected.
- Verification:
  - `python3 -I tools/gate_checks.py --base main --task tasks/R13-06-street-props-a.md` passes with `scope: PASS`.
- Known issues:
  - None. All models and colliders are clean and within budgets.
