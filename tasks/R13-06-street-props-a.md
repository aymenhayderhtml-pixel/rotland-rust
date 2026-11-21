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
