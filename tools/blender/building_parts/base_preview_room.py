"""base_preview_room: renders base_preview_room.png showing a 4x4 m test assembly.

Verifies that player-built base parts (floor, walls, door, window, half wall, roof, stairs)
meet seamlessly on the 2 m grid without gaps or z-fighting.
Image only; no model is exported.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, HERE)

import bmesh
from mathutils import Vector

import common
import parts_common
import base_floor
import base_wall
import base_wall_door
import base_wall_window
import base_half_wall
import base_roof
import base_stairs


def make_part_bm(build_fn):
    """Builds a part into a combined bmesh without exporting."""
    m = common.Model("temp", parts_common.CATEGORY, 2000)
    build_fn(m)
    visuals = [bm for bm, _ in m._visual]
    for _, bm in m._parts:
        visuals.append(bm)
    return common.combine(visuals)


def build_room():
    common.reset_scene()

    # Room is 4x4 m, spanning X: [-2.0, 2.0], Y: [-2.0, 2.0]
    parts_to_place = []

    # 1. Four floor tiles covering the 4x4 m ground floor
    floor_spots = [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
    for x, y in floor_spots:
        bm = make_part_bm(base_floor.build)
        common.transform(bm, loc=(x, y, 0.0))
        parts_to_place.append(bm)

    # 2. Perimeter walls
    # South face (Y = -2.0, front)
    # Door on left: (-1.0, -2.0, 0.0)
    bm_door = make_part_bm(base_wall_door.build)
    common.transform(bm_door, loc=(-1.0, -2.0, 0.0), rot=(0.0, 0.0, 0.0))
    parts_to_place.append(bm_door)

    # Window wall on right: (1.0, -2.0, 0.0)
    bm_win_front = make_part_bm(base_wall_window.build)
    common.transform(bm_win_front, loc=(1.0, -2.0, 0.0), rot=(0.0, 0.0, 0.0))
    parts_to_place.append(bm_win_front)

    # North face (Y = 2.0, back)
    bm_wall_n1 = make_part_bm(base_wall.build)
    common.transform(bm_wall_n1, loc=(-1.0, 2.0, 0.0), rot=(0.0, 0.0, math.pi))
    parts_to_place.append(bm_wall_n1)

    bm_win_n2 = make_part_bm(base_wall_window.build)
    common.transform(bm_win_n2, loc=(1.0, 2.0, 0.0), rot=(0.0, 0.0, math.pi))
    parts_to_place.append(bm_win_n2)

    # West face (X = -2.0, left)
    bm_wall_w1 = make_part_bm(base_wall_window.build)
    common.transform(bm_wall_w1, loc=(-2.0, -1.0, 0.0), rot=(0.0, 0.0, math.pi / 2))
    parts_to_place.append(bm_wall_w1)

    bm_wall_w2 = make_part_bm(base_wall.build)
    common.transform(bm_wall_w2, loc=(-2.0, 1.0, 0.0), rot=(0.0, 0.0, math.pi / 2))
    parts_to_place.append(bm_wall_w2)

    # East face (X = 2.0, right)
    bm_wall_e1 = make_part_bm(base_wall.build)
    common.transform(bm_wall_e1, loc=(2.0, -1.0, 0.0), rot=(0.0, 0.0, -math.pi / 2))
    parts_to_place.append(bm_wall_e1)

    bm_wall_e2 = make_part_bm(base_wall.build)
    common.transform(bm_wall_e2, loc=(2.0, 1.0, 0.0), rot=(0.0, 0.0, -math.pi / 2))
    parts_to_place.append(bm_wall_e2)

    # 3. Interior stairs in the left room: from Y = -1.0 to +1.0 at X = -1.0, rising 3 m
    bm_stairs = make_part_bm(base_stairs.build)
    common.transform(bm_stairs, loc=(-1.0, 0.0, 0.0))
    parts_to_place.append(bm_stairs)

    # 4. Interior half-wall divider / mezzanine railing along the edge between cells
    bm_half = make_part_bm(base_half_wall.build)
    common.transform(bm_half, loc=(0.0, -1.0, 0.0), rot=(0.0, 0.0, -math.pi / 2))
    parts_to_place.append(bm_half)

    # 5. Roof tiles on the right half at Z = 3.0 (covering X = [0.0, 2.0], Y = [-2.0, 2.0])
    # Allows seeing the roof slope/overhang on the right, while leaving the left half open to view
    # the interior stairs, floorboards, doorway, and inner wall faces
    bm_roof_front = make_part_bm(base_roof.build)
    common.transform(bm_roof_front, loc=(1.0, -1.0, 3.0))
    parts_to_place.append(bm_roof_front)

    bm_roof_back = make_part_bm(base_roof.build)
    common.transform(bm_roof_back, loc=(1.0, 1.0, 3.0))
    parts_to_place.append(bm_roof_back)

    # Combine all parts into single object for preview render
    room_bm = common.combine(parts_to_place)
    bmesh.ops.triangulate(room_bm, faces=list(room_bm.faces))
    bmesh.ops.recalc_face_normals(room_bm, faces=list(room_bm.faces))

    obj = common._to_object(room_bm, "base_preview_room", painted=True)

    # Render preview image: 3/4 front-right view, slightly elevated for interior visibility
    view_dir = (1.2, -1.5, 1.1)
    common.render_preview("base_preview_room", obj, view=view_dir)
    print("PREVIEW ROOM: rendered assets/models/_previews/base_preview_room.png")


def main():
    build_room()
    return 0


if __name__ == "__main__":
    sys.exit(main())
