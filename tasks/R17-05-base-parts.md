# R17-05: player-built base part models (structure)

Read first: `tasks/R17-common.md`, `data/build_parts.json` (grid 2 m, floor height 3 m, the parts and their snap: cell parts vs edge parts), `game/src/building/state.rs` (`world_transform`: where the origin of each part is and which way it faces; MATCH IT), `tools/blender/buildings/kit.py`.
Files: tools/blender/building_parts/*, assets/models/building_parts/*, assets/models/_previews/base_*, tasks/R17-05-base-parts.md

Make a new folder `tools/blender/building_parts/` with one script per part, model ids prefixed `base_` (to avoid clashing with city props), category `building_parts` (writes to `assets/models/building_parts/`; if `common.run_model` needs a known category, check how categories map to folders and use it), and a `build_parts.py` that builds them all.
- Structure, wood plank style (the player builds from wood): `base_floor` (2x2 m, 0.1 m thick), `base_wall` (2 m wide, 3 m tall, 0.15 m thick, on a cell edge), `base_wall_window`, `base_wall_door` (frame + a separate child mesh `door_leaf` hinged at one side, so the game can swing it), `base_half_wall` (1.2 m), `base_roof` (2x2 flat with a slight slope and overhang), `base_stairs` (2x2 footprint rising 3 m).
- Seams: parts placed next to each other on the 2 m grid must meet without gaps or z-fighting (walls overlap the floor edge by the right amount). Render a preview of a small test assembly too (a 4x4 m room with a door, window, roof and stairs) as `base_preview_room.png` (a preview image only, not a model).
- Budget 300 triangles per part (door 400). Colliders on walls/floors/roof/stairs (use the same collider helpers as `kit.py`).

## Worker notes
- Created `tools/blender/building_parts/` with one script per part, `parts_common.py`, and `build_parts.py`:
  - `base_floor.py`: 2x2 m, 0.1 m thick wooden floor tile, 144 tris (budget 300). Box collider.
  - `base_wall.py`: 2 m wide, 3 m tall, 0.15 m thick wooden wall on cell edge, 144 tris (budget 300). Box collider.
  - `base_wall_window.py`: 2 m wide, 3 m tall, 0.15 m thick wall with centered window opening, sill and mullion cross, 252 tris (budget 300). 4 convex colliders leaving window opening unblocked.
  - `base_wall_door.py`: 2 m wide, 3 m tall, 0.15 m thick wall with doorway opening + separate child mesh `door_leaf` with strap hinges, handle, and `door_hinge` marker, 360 tris (budget 400). Frame colliders leaving doorway walkable.
  - `base_half_wall.py`: 2 m wide, 1.2 m tall, 0.15 m thick wooden railing / half wall, 96 tris (budget 300). Box collider.
  - `base_roof.py`: 2x2 m flat roof with ceiling joists, slight slope (Z 0.06 to 0.12), front eave overhang with fascia drip edge, 168 tris (budget 300). Sloped slab collider.
  - `base_stairs.py`: 2x2 m footprint rising 3 m (10 steps) with side stringers and handrail, 288 tris (budget 300). Smooth sloped ramp collider.
  - `base_preview_room.py`: 4x4 m test assembly room (floors, door, window, half wall, roof, stairs) rendered to `assets/models/_previews/base_preview_room.png` (image only).
  - `build_parts.py`: builds all 7 parts + test assembly room preview; supports `--only=<names>`.
- Wiring notes for Claude:
  - Category `building_parts` writes to `assets/models/building_parts/`.
  - Origin conventions match `game/src/building/state.rs`:
    - Cell parts (`base_floor`, `base_roof`, `base_stairs`): centered at origin `(0, 0, 0)` in X and Y, bottom at Z = 0.
    - Edge parts (`base_wall`, `base_wall_window`, `base_wall_door`, `base_half_wall`): origin at `(0, 0, 0)`. Sits along cell edge (width along X [-1.0, 1.0], thickness along Y [-0.075, 0.075]). Overlaps floor edges by 0.075 m symmetrically without z-fighting.
    - `base_wall_door`: frame colliders keep doorway open. `door_leaf` child mesh node and `door_hinge` empty marker at `(-0.46, 0.0, 0.0)` allow game-driven door swinging.
    - `base_stairs`: smooth ramp collider prevents snagging on step edges.
- How to test:
  - `flock /tmp/rotland-blender.lock blender -b --python tools/blender/building_parts/build_parts.py`
  - `python3 -I tools/gate_checks.py --base main --task tasks/R17-05-base-parts.md`
- Known issues:
  - None. All triangle budgets pass, all preview images visually verified.

