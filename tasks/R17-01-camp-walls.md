# R17-01: camp upgrade models: walls and gates (3 tiers)

Read first: `tasks/R17-common.md`, `data/camp_upgrades.json` (walls line), `tools/blender/buildings/settlement.py` + `settlement_parts.py` (today's camp, its palisade style and size), `tools/blender/buildings/kit.py`.
Files: tools/blender/buildings/camp_wall_*.py, tools/blender/buildings/camp_gate_*.py, assets/models/buildings/camp_wall_*, assets/models/buildings/camp_gate_*, assets/models/_previews/camp_wall_*, assets/models/_previews/camp_gate_*, tasks/R17-01-camp-walls.md

Wall SECTIONS that tile in a row (the game places many along the camp edge): each section 4 m long, centered on x, built along x, thickness along y, colliders like `kit.solid`, budget 1200 triangles per section, 2500 per gate.
- `camp_wall_wood` (2.6 m sharpened log palisade with a walkway plank inside), `camp_wall_metal` (3 m corrugated sheets on posts, rusty patches, barbed top wire), `camp_wall_concrete` (3.2 m concrete blocks with a slight lean, rebar showing at one broken top, a painted stripe).
- `camp_gate_wood`, `camp_gate_metal`, `camp_gate_concrete`: 6 m wide opening between two posts/towers matching each wall; the gate leaves are a separate child mesh named `gate_leaf_l` / `gate_leaf_r` (the game can swing them later); a `door_1` empty in the middle.
- A `damaged` look is NOT needed now.

## Worker notes

