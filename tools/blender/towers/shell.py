"""Shared ground-floor lobby shell for all R39 tower bases.

Footprint is `size` x `size` centred on X=0, Y=0, ground at Z=0, front street on +Y.
The +Y wall has a 2.8 m x 3.0 m walkable door gap; the -Y wall has a closed `stair_door`
part + collider so R43 can open it later.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))

import common  # noqa: E402
import jam_util  # noqa: E402
import kit  # noqa: E402

WALL_T = 0.3  # new: DECISIONS 2026-11-26 (exterior tower wall thickness)
FLOOR_T = kit.FLOOR_TOP  # 0.1 m raised lobby slab
HALL_TOP = 7.3  # new: DECISIONS 2026-11-26 (interior hall ceiling height)
BASE_H = 7.6  # new: DECISIONS 2026-11-26 (2-floor base height)
DOOR_W = 2.8  # new: DECISIONS 2026-11-26 (walk-in lobby door width)
DOOR_H = 3.0  # new: DECISIONS 2026-11-26 (walk-in lobby door height)
STAIR_W = 1.2  # new: DECISIONS 2026-11-26 (back stair door width)
STAIR_H = 2.2  # new: DECISIONS 2026-11-26 (back stair door height)


def front_face_y(size):
    """Y coordinate of the +Y street-facing exterior wall."""
    return size / 2.0


def door_opening(size):
    """Front door gap `(lo, hi)` (x -1.4..1.4, z 0.1..3.1) cutting through the +Y wall."""
    fy = front_face_y(size)
    return (
        (-DOOR_W / 2.0, fy - WALL_T - 0.05, FLOOR_T),
        (DOOR_W / 2.0, fy + 0.05, FLOOR_T + DOOR_H),
    )


def _stair_door_bm(half):
    """Closed stair door on the -Y interior wall, centred on X=0, 1.2 m x 2.2 m."""
    y0 = -half + WALL_T + 0.02
    y1 = y0 + 0.06
    hw = STAIR_W / 2.0
    z0, z1 = FLOOR_T, FLOOR_T + STAIR_H
    parts = [
        jam_util.pbox((-hw, y0, z0), (hw, y1, z1), "metal", bevel=0.01),
        jam_util.pbox((-hw - 0.06, y0, z0), (-hw, y1 + 0.02, z1 + 0.06), "gun_black", bevel=0.008),
        jam_util.pbox((hw, y0, z0), (hw + 0.06, y1 + 0.02, z1 + 0.06), "gun_black", bevel=0.008),
        jam_util.pbox((-hw, y0, z1), (hw, y1 + 0.02, z1 + 0.06), "gun_black", bevel=0.008),
        jam_util.pbox((hw - 0.16, y1, z0 + 0.95), (hw - 0.08, y1 + 0.05, z0 + 1.10), "brass", bevel=0.005),
    ]
    return common.combine(parts)


def lobby_shell(k, size, facade_color):
    """Builds the walkable ground-floor lobby shell on `k` (a `kit.Kit`)."""
    half = size / 2.0
    door = door_opening(size)

    # Floor slab over the full footprint (visual + collider)
    k.solid((-half, -half, 0.0), (half, half, FLOOR_T), "concrete_grey", base="floor")

    # Four exterior walls (0.3 m thick, z 0.1..7.3) with colliders; front wall has the door gap
    k.wall((-half, half - WALL_T, FLOOR_T), (half, half, HALL_TOP),
           openings=[door], color=facade_color, base="wall_front")
    k.wall((-half, -half, FLOOR_T), (half, -half + WALL_T, HALL_TOP),
           color=facade_color, base="wall_back")
    k.wall((-half, -half + WALL_T, FLOOR_T), (-half + WALL_T, half - WALL_T, HALL_TOP),
           color=facade_color, base="wall_left")
    k.wall((half - WALL_T, -half + WALL_T, FLOOR_T), (half, half - WALL_T, HALL_TOP),
           color=facade_color, base="wall_right")

    # Interior plaster lining (rock_light, visual only, 0.02 m proud inside so no z-fighting)
    in_lo = -half + WALL_T
    in_hi = half - WALL_T
    k.box((in_lo, in_lo, FLOOR_T), (in_hi, in_lo + 0.02, HALL_TOP), "rock_light")
    k.box((in_lo, in_lo + 0.02, FLOOR_T), (in_lo + 0.02, in_hi - 0.02, HALL_TOP), "rock_light")
    k.box((in_hi - 0.02, in_lo + 0.02, FLOOR_T), (in_hi, in_hi - 0.02, HALL_TOP), "rock_light")
    k.box((in_lo, in_hi - 0.02, FLOOR_T), (-DOOR_W / 2.0, in_hi, HALL_TOP), "rock_light")
    k.box((DOOR_W / 2.0, in_hi - 0.02, FLOOR_T), (in_hi, in_hi, HALL_TOP), "rock_light")
    k.box((-DOOR_W / 2.0, in_hi - 0.02, FLOOR_T + DOOR_H), (DOOR_W / 2.0, in_hi, HALL_TOP), "rock_light")

    # Ceiling slab z 7.3..7.6 (visual only, roof=True so top_preview shows the open hall)
    k.box((in_lo, in_lo, HALL_TOP - 0.02), (in_hi, in_hi, HALL_TOP), "rock_light", roof=True)
    k.box((-half, -half, HALL_TOP), (half, half, BASE_H), facade_color, roof=True)

    # Closed stair door on the -Y back wall (separate node + collider for R43)
    k.model.add_part("stair_door", _stair_door_bm(half))
    col_bm = common.make_box((STAIR_W, 0.06, STAIR_H),
                             loc=(0.0, -half + WALL_T + 0.05, FLOOR_T + STAIR_H / 2.0),
                             color="white")
    k.model.add_collider("stair_door", col_bm)
