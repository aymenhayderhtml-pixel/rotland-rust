# R17-05: player-built base part models (structure)

Read first: `tasks/R17-common.md`, `data/build_parts.json` (grid 2 m, floor height 3 m, the parts and their snap: cell parts vs edge parts), `game/src/building/state.rs` (`world_transform`: where the origin of each part is and which way it faces; MATCH IT), `tools/blender/buildings/kit.py`.
Files: tools/blender/building_parts/*, assets/models/building_parts/*, assets/models/_previews/base_*, tasks/R17-05-base-parts.md

Make a new folder `tools/blender/building_parts/` with one script per part, model ids prefixed `base_` (to avoid clashing with city props), category `building_parts` (writes to `assets/models/building_parts/`; if `common.run_model` needs a known category, check how categories map to folders and use it), and a `build_parts.py` that builds them all.
- Structure, wood plank style (the player builds from wood): `base_floor` (2x2 m, 0.1 m thick), `base_wall` (2 m wide, 3 m tall, 0.15 m thick, on a cell edge), `base_wall_window`, `base_wall_door` (frame + a separate child mesh `door_leaf` hinged at one side, so the game can swing it), `base_half_wall` (1.2 m), `base_roof` (2x2 flat with a slight slope and overhang), `base_stairs` (2x2 footprint rising 3 m).
- Seams: parts placed next to each other on the 2 m grid must meet without gaps or z-fighting (walls overlap the floor edge by the right amount). Render a preview of a small test assembly too (a 4x4 m room with a door, window, roof and stairs) as `base_preview_room.png` (a preview image only, not a model).
- Budget 300 triangles per part (door 400). Colliders on walls/floors/roof/stairs (use the same collider helpers as `kit.py`).

## Worker notes

