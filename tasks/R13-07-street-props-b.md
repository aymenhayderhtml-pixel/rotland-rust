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

### Models Created
1. **`rubble_pile`** (`tools/blender/props/rubble_pile.py` -> `assets/models/props/rubble_pile.glb`):
   - 3x3 m pile of broken concrete slabs, masonry rubble chunks, brick accents, and bent rusted rebar rods looping across the mound.
   - Convex box collider `rubble`.
   - Triangles: **684** (budget: 1500).

2. **`car_wreck_sedan`** (`tools/blender/props/car_wreck_sedan.py` -> `assets/models/props/car_wreck_sedan.glb`):
   - Burned-out sedan wreck with 3-box notchback body, crumpled hood exposing scorched engine block/radiator, stripped cabin with dash/steering/seats, wheelless left side propped on a concrete block with bare brake rotor/caliper, flat tires on the right, dangling bumpers, and popped trunk.
   - Convex box collider `sedan`.
   - Triangles: **696** (budget: 2500).

3. **`car_wreck_van`** (`tools/blender/props/car_wreck_van.py` -> `assets/models/props/car_wreck_van.glb`):
   - Abandoned cargo van with 4 flat squashed tires, front cab with windshield pillars, dash and bucket seats, open sliding side door showing hollow interior cargo bay with a loot crate and rusty barrel, rear doors, and ribbed roof.
   - Convex box collider `van`.
   - Triangles: **1104** (budget: 2500).

4. **`sign_fallen`** (`tools/blender/props/sign_fallen.py` -> `assets/models/props/sign_fallen.glb`):
   - Fallen street sign lying on the ground: concrete collar with sheared stub, continuous bent metal pole lying along the ground, octagonal STOP sign with white border rim and bullet holes, plus a green street name plate.
   - Low box collider `fallen`.
   - Triangles: **200** (budget: 300).

5. **`fence_broken`** (`tools/blender/props/fence_broken.py` -> `assets/models/props/fence_broken.glb`):
   - 3 m chain-link fence section with vertical left post, leaning knocked-askew right post, buckled snapped top rail, diamond criss-cross mesh strips, center breach with curled torn wire ends, and barbed wire strands on top.
   - Box colliders `left_panel` and `right_post`.
   - Triangles: **576** (budget: 800).

6. **`jersey_barrier`** (`tools/blender/props/jersey_barrier.py` -> `assets/models/props/jersey_barrier.glb`):
   - 3 m concrete Jersey barrier with standard profile, base forklift/drainage notches, chipped fractured top corner exposing bent rusty rebar, and debris chunks.
   - Convex box collider `barrier`.
   - Triangles: **134** (budget: 300).

### Verification
- All previews rendered and visually inspected in `assets/models/_previews/`.
- Scope check passed: `python3 -I tools/gate_checks.py --base main --task tasks/R13-07-street-props-b.md` (`scope: PASS`).
- Known issues: None.

