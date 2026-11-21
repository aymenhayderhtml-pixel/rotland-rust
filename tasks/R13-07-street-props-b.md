# R13-07: street props b for the city

Read first: `tasks/R13-common.md`, `docs/CITY_SPEC.md` section 5, `tools/blender/props/car_wreck.py`, `tools/blender/props/road_sign.py`, `tools/blender/common.py` (only the functions you use).
Files: tools/blender/props/rubble_pile.py, tools/blender/props/car_wreck_sedan.py, tools/blender/props/car_wreck_van.py, tools/blender/props/sign_fallen.py, tools/blender/props/fence_broken.py, tools/blender/props/jersey_barrier.py, assets/models/props/rubble_pile*, assets/models/props/car_wreck_sedan*, assets/models/props/car_wreck_van*, assets/models/props/sign_fallen*, assets/models/props/fence_broken*, assets/models/props/jersey_barrier*, assets/models/_previews/rubble_pile*, assets/models/_previews/car_wreck_sedan*, assets/models/_previews/car_wreck_van*, assets/models/_previews/sign_fallen*, assets/models/_previews/fence_broken*, assets/models/_previews/jersey_barrier*, tasks/R13-07-*.md

One script per prop, category `props`, low-poly, flat colors, ruined look. No LOD files needed. Base on the ground at z=0, real-world size.

| Model | What | Triangle budget |
|---|---|---|
| rubble_pile | pile of broken concrete chunks and a bent rebar, about 3 x 3 m | 1500 |
| car_wreck_sedan | burned-out sedan wreck, no wheels on one side, rusty dark colors (look at car_wreck.py) | 2500 |
| car_wreck_van | rusty abandoned van, flat tires, open side door | 2500 |
| sign_fallen | street sign fallen over on the ground, bent pole | 300 |
| fence_broken | chain-link-look fence section 3 m, bent and torn (posts plus a frame, mesh drawn as thin bars) | 800 |
| jersey_barrier | concrete road barrier 3 m long, chipped | 300 |

## Worker notes
