"""crate_loot: wooden loot crate with a dark plank line and metal corner strips (<= 120 tris).

The visible grid (3x3 cells per side) is what keeps the triangle count low.
"""
import os
import sys

from mathutils import Vector

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "crate_loot", "props", 120
HALF = {"x": 0.4, "y": 0.3, "z": 0.3}  # 0.8 x 0.6 x 0.6 m
CENTER = Vector((0.0, 0.0, 0.3))  # bottom sits on z = 0
EDGE = 0.06  # width of the metal corner strips
LINE = 0.02  # half-width of the dark plank line through the middle
AXES = {"x": (1, 2), "y": (2, 0), "z": (0, 1)}  # the two in-plane axes for each face normal
INDEX = {"x": 0, "y": 1, "z": 2}


def build(model):
    body = common.new_bm()
    for axis in ("x", "y", "z"):
        for sign in (-1.0, 1.0):
            u_axis, v_axis = AXES[axis]
            u_name = "xyz"[u_axis]
            v_name = "xyz"[v_axis]
            hu, hv = HALF[u_name], HALF[v_name]
            u_cuts = [-hu, -hu + EDGE, hu - EDGE, hu]
            v_cuts = [-hv, -LINE, LINE, hv]
            outward = Vector((0.0, 0.0, 0.0))
            outward[INDEX[axis]] = sign
            for i in range(3):
                for j in range(3):
                    if i in (0, 2):
                        color = "metal"
                    elif j == 1:
                        color = "bark_dark"
                    else:
                        color = "wood_crate"
                    corners = []
                    for a, b in [(u_cuts[i], v_cuts[j]), (u_cuts[i + 1], v_cuts[j]),
                                 (u_cuts[i + 1], v_cuts[j + 1]), (u_cuts[i], v_cuts[j + 1])]:
                        p = Vector((0.0, 0.0, 0.0))
                        p[INDEX[axis]] = sign * HALF[axis]
                        p[u_axis] = a
                        p[v_axis] = b
                        corners.append(CENTER + p)
                    common.add_quad(body, corners, outward, color)
    model.add(body)
    model.add_collider("box", common.make_box((0.8, 0.6, 0.6), loc=tuple(CENTER), color="wood_crate"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
