# R27-06: z-fighting check on the city buildings

Files: tools/blender/city/*, tools/blender/buildings/office*.py, tools/blender/buildings/police*.py, tools/blender/buildings/hospital*.py, tools/blender/buildings/shop*.py, tools/blender/buildings/city_*.py, assets/models/city/*, assets/models/buildings/office*, assets/models/buildings/police*, assets/models/buildings/hospital*, assets/models/buildings/shop*, assets/models/buildings/city_*, assets/models/_previews/*, tasks/R27-06-zfight-city.md
Rules: `tasks/R27-common.md` (Blender lane: no cargo, no gate; build with `flock /tmp/rotland-blender.lock blender -b --python <script>`; check scope with `python3 -I tools/gate_checks.py --base main --task tasks/R27-06-zfight-city.md`).

**Goal:** flat details placed flush on walls flicker in game (z-fighting). The apartment was fixed by moving its window details 2 cm out from the wall. Check the other city building scripts (find them: grep the city building model names used by `world/src` city code; e.g. office glass bands, police stripes, hospital stripes, shop signs) for any detail plane/box face that sits exactly on (or within 1 cm of) a wall, roof or floor face, and move it out 2 cm (same fix as the apartment; find that commit with `git log --oneline -- tools/blender | grep -i apartment`). Rebuild each changed model, look at its preview PNG, list every building checked and what changed in Worker notes. Do not change sizes, colours or layout.

## Worker notes

### Buildings checked & what changed
1. `office.py` (`assets/models/buildings/office.glb`):
   - Moved upper-floor glass bands (front, back, west, east) from 1 cm proud of the exterior walls to 2 cm proud (`7.22`, `-7.62`, `-7.62`, `7.62`), and moved their mullions from 2 cm to 4 cm proud (`7.24`, `-7.64`, `-7.64`, `7.64`) so mullions stay proud of the glass.
   - Note: ground-floor storefront windows (`y in [7.08, 7.18]`) are currently inside `wall_front_gf` (`y in [7.05, 7.2]`, which has only `front_door` in `openings`); left untouched per "Do not change sizes, colours or layout".
2. `office_lod1.py` (`assets/models/buildings/office_lod1.glb`):
   - Moved front spandrel bands under upper-floor windows from 1 cm proud (`7.21`) to 2 cm proud (`7.22`).
3. `police.py` (`assets/models/buildings/police.glb`):
   - Raised parking bay lines top from 1 cm (`0.05`) to 2 cm (`0.06`) above the asphalt (`0.04`).
   - Expanded parking barrier arm red stripes from 1 cm proud (`6.93..7.07`, `0.81..0.95`) to 2 cm proud (`6.92..7.08`, `0.80..0.96`) of the white arm (`6.94..7.06`, `0.82..0.94`).
   - Checked blue accent stripes (already 2 cm proud of walls) and upper window panes (already 4 cm proud).
4. `police_lod1.py` (`assets/models/buildings/police_lod1.glb`):
   - Moved front entrance door glass panel from 1 cm proud of the door (`7.03`) to 2 cm proud (`7.04`).
   - Checked blue stripes and flat painted windows (already 2 cm proud of walls).
5. `hospital.py` (`assets/models/buildings/hospital.glb`):
   - Raised ambulance bay red parking box top from 1 cm (`0.05`) to 2 cm (`0.06`) above asphalt (`0.04`), and raised the white border lines and cross on top of it from `0.05..0.06` to `0.06..0.08` (2 cm above the red box).
   - Checked red accent stripes (already 2 cm proud), red cross facade sign (already 6–9 cm proud), and helipad markings (already 2–3 cm proud).
6. `hospital_lod1.py` (`assets/models/buildings/hospital_lod1.glb`):
   - Raised ambulance bay red parking box top from 1 cm (`0.05`) to 2 cm (`0.06`) above asphalt (`0.04`).
   - Moved front entrance door glass panel from 1 cm proud of the door (`9.03`) to 2 cm proud (`9.04`).
   - Checked red stripes and flat painted windows (already 2 cm proud of walls).
7. `shop.py`:
   - Checked exterior walls, sign boards (6–8 cm proud), awnings, roof details, and interior fixtures; no detail faces within 1 cm of a wall, roof, or floor face. Unchanged.
8. `shop_lod1.py` (`assets/models/buildings/shop_lod1.glb`):
   - Moved front door and window colored panels from 1.5 cm proud (`3.205..3.215`) to 2 cm proud (`3.21..3.22`) of the front wall (`3.2`).

### How to see it in the game
- Visit the city biome (`distanceFromCamp: 1500.0` in `data/city.json`) and inspect `office`, `police`, `hospital`, `shop` and their distant LOD1 variants (`office_lod1`, `police_lod1`, `hospital_lod1`, `shop_lod1`); glass bands, spandrels, door glass, parking markings, and LOD window/door panels no longer flicker at distance.
- Also previewed in `assets/models/_previews/{office,office_lod1,police,police_lod1,hospital,hospital_lod1,shop_lod1}.png`.
