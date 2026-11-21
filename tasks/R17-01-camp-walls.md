# R17-01: camp upgrade models: walls and gates (3 tiers)

Read first: `tasks/R17-common.md`, `data/camp_upgrades.json` (walls line), `tools/blender/buildings/settlement.py` + `settlement_parts.py` (today's camp, its palisade style and size), `tools/blender/buildings/kit.py`.
Files: tools/blender/buildings/camp_wall_*.py, tools/blender/buildings/camp_gate_*.py, assets/models/buildings/camp_wall_*, assets/models/buildings/camp_gate_*, assets/models/_previews/camp_wall_*, assets/models/_previews/camp_gate_*, tasks/R17-01-camp-walls.md

Wall SECTIONS that tile in a row (the game places many along the camp edge): each section 4 m long, centered on x, built along x, thickness along y, colliders like `kit.solid`, budget 1200 triangles per section, 2500 per gate.
- `camp_wall_wood` (2.6 m sharpened log palisade with a walkway plank inside), `camp_wall_metal` (3 m corrugated sheets on posts, rusty patches, barbed top wire), `camp_wall_concrete` (3.2 m concrete blocks with a slight lean, rebar showing at one broken top, a painted stripe).
- `camp_gate_wood`, `camp_gate_metal`, `camp_gate_concrete`: 6 m wide opening between two posts/towers matching each wall; the gate leaves are a separate child mesh named `gate_leaf_l` / `gate_leaf_r` (the game can swing them later); a `door_1` empty in the middle.
- A `damaged` look is NOT needed now.

## Worker notes
- **Files created**:
  - `tools/blender/buildings/camp_wall_wood.py` -> `assets/models/buildings/camp_wall_wood.glb` (672 tris / budget 1200)
  - `tools/blender/buildings/camp_wall_metal.py` -> `assets/models/buildings/camp_wall_metal.glb` (876 tris / budget 1200)
  - `tools/blender/buildings/camp_wall_concrete.py` -> `assets/models/buildings/camp_wall_concrete.glb` (480 tris / budget 1200)
  - `tools/blender/buildings/camp_gate_wood.py` -> `assets/models/buildings/camp_gate_wood.glb` (1340 tris / budget 2500)
  - `tools/blender/buildings/camp_gate_metal.py` -> `assets/models/buildings/camp_gate_metal.glb` (1740 tris / budget 2500)
  - `tools/blender/buildings/camp_gate_concrete.py` -> `assets/models/buildings/camp_gate_concrete.glb` (1068 tris / budget 2500)
  - Previews generated in `assets/models/_previews/` for all 6 models.

- **Models implementation details**:
  - `camp_wall_wood`: 4 m section centered on X (-2.0 to +2.0) and Y, grounded at Z=0. 16 vertical sharpened logs with varied heights (~2.6 m average), lashing cross-rails on inside (-Y), outside sleeper log sill (+Y), inside walkway plank (z=1.3 m) with vertical timber support posts, diagonal struts, and access ladder. Colliders for palisade wall and walkway deck.
  - `camp_wall_metal`: 4 m section, 3.0 m tall corrugated steel panels with accordion profile, 3 steel square column posts (x = -2.0, 0.0, +2.0), horizontal girts and diagonal rear kickers, weathered rust patches (`rust` and `red_paint`), outrigger brackets with 3 strands of barbed top wire and barbs. Solid metal wall collider.
  - `camp_wall_concrete`: 4 m section, 3.2 m tall heavy concrete blocks in running bond with chamfered joints, progressive settling lean along Y (~2.1° tilt), bold painted red/white hazard stripe across Course 1, and a broken top corner on Course 3 with jagged fracture, debris chunks, and exposed twisted steel rebars (`rust`, `gun_black`). Solid concrete wall collider.
  - `camp_gate_wood`: 6 m clear opening between inner faces of towers (-3.0 to +3.0). Two palisade lookout towers on left (-4.0 to -3.0) and right (+3.0 to +4.0) with deck at z=2.8m and parapets to 3.8m. Overhead lintel log with camp sign at z=3.65m. Separate child meshes `gate_leaf_l` and `gate_leaf_r` (2.6 m sharpened logs with horizontal rails, diagonal Z-brace, strap hinges at -3.0/+3.0, center latch). `door_1` empty at (0, 0, 0). Colliders on towers only.
  - `camp_gate_metal`: 6 m clear opening between two industrial steel truss towers (-4.0 to -3.0 and +3.0 to +4.0). Overhead steel truss gantry with red/white hazard sign at z=3.90m. Separate child meshes `gate_leaf_l` and `gate_leaf_r` (3.0 m corrugated panels on steel tube frames, rust patches, barbed top wire, hinges at -3.0/+3.0, center latch). `door_1` empty at (0, 0, 0). Colliders on towers only.
  - `camp_gate_concrete`: 6 m clear opening between two massive concrete block piers (-4.2 to -3.0 and +3.0 to +4.2) up to 4.0m height. Left pier top corner broken with exposed twisted rebars; painted hazard stripe matching the concrete wall. Separate child meshes `gate_leaf_l` and `gate_leaf_r` (3.2 m heavy steel blast doors with armored viewing slits, iron grating bars, diagonal stiffener ribs, top anti-climb spikes, painted hazard stripe). `door_1` empty at (0, 0, 0). Colliders on piers only.

- **How to test**:
  - Run Blender on any script: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/buildings/<script>.py`
  - Inspect previews in `assets/models/_previews/camp_wall_*.png` and `assets/models/_previews/camp_gate_*.png`.
  - Run scope check: `python3 -I tools/gate_checks.py --base main --task tasks/R17-01-camp-walls.md`

- **Wiring notes for Claude**:
  - Add to `tools/blender/buildings/build_buildings.py`'s `BUILDINGS` list if desired: `"camp_wall_wood"`, `"camp_wall_metal"`, `"camp_wall_concrete"`, `"camp_gate_wood"`, `"camp_gate_metal"`, `"camp_gate_concrete"`.
  - Gate models have clear 6 m opening between inner tower faces; gate leaves are child meshes `gate_leaf_l` and `gate_leaf_r` hinged at x = -3.0 and x = +3.0; `door_1` empty is at (0, 0, 0).
  - Wall sections tile seamlessly along X (length 4.0 m, -2.0 to +2.0).

- **Known issues**: None. All 6 models build cleanly and are well within polygon budgets.


