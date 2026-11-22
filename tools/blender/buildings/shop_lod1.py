"""shop_lod1: low LOD for shop, 16 x 8 m footprint on an 18 x 10 m lot.

Same footprint, outline, roof shape and main colors as shop, with no interior
or small detail parts.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "shop_lod1", "buildings", 500
TOP = 3.3
F = kit.FLOOR_TOP


def _make_simple_awning(k, x0, x1, color):
    width = x1 - x0
    x_mid = (x0 + x1) / 2
    dy = 0.8
    dz = -0.4
    length = math.hypot(dy, dz)
    rot_x = math.atan2(dz, dy)
    slab = common.make_box((width, length, 0.06), color=color)
    common.transform(slab, loc=(x_mid, 3.6, 2.65), rot=(rot_x, 0.0, 0.0))
    k.model.add(slab)
    # Front valance
    k.box((x0 + 0.01, 3.96, 2.30), (x1 - 0.01, 4.0, 2.45), "white")


def build(model):
    k = kit.Kit(model)

    # Floor / curb base
    k.box((-8.0, -4.0, 0.0), (8.0, 3.2, F), "rock_light")
    k.box((-8.0, 3.2, 0.0), (8.0, 4.0, F), "rock_mid")

    # Main building box (outer shell)
    k.box((-8.0, -4.0, F), (8.0, 3.2, TOP), "rock_light")

    # Roof slab and parapets
    k.box((-8.0, -4.0, TOP), (8.0, 3.2, TOP + 0.15), "rock_dark")
    k.box((-8.0, -4.0, TOP + 0.15), (8.0, -3.85, TOP + 0.35), "rock_mid")
    k.box((-8.0, 3.05, TOP + 0.15), (8.0, 3.2, TOP + 0.45), "rock_mid")
    k.box((-8.0, -3.85, TOP + 0.15), (-7.85, 3.05, TOP + 0.35), "rock_mid")
    k.box((7.85, -3.85, TOP + 0.15), (8.0, 3.05, TOP + 0.35), "rock_mid")

    # Rooftop AC units
    k.box((-4.5, -2.0, TOP + 0.15), (-3.0, -0.6, TOP + 0.8), "metal")
    k.box((2.8, -2.2, TOP + 0.15), (4.2, -0.8, TOP + 0.8), "metal")

    # Awnings
    _make_simple_awning(k, -7.2, -0.6, "red_paint")
    _make_simple_awning(k, 0.6, 7.2, "shirt_blue")

    # Sign boards (blank colored panels)
    k.box((-7.0, 3.21, 2.88), (-1.0, 3.26, 3.25), "canvas_khaki")
    k.box((1.0, 3.21, 2.88), (7.0, 3.26, 3.25), "cloth_olive")

    # Door and window colored panels on front face
    k.box((-2.0, 3.205, F), (-0.8, 3.215, kit.DOOR_H), "gun_black")
    k.box((-6.8, 3.205, 0.8), (-2.6, 3.215, 2.4), "rock_dark")
    k.box((0.8, 3.205, F), (2.0, 3.215, kit.DOOR_H), "gun_black")
    k.box((2.6, 3.205, 0.8), (6.8, 3.215, 2.4), "rock_dark")

    # Collision box
    k.collider((-8.0, -4.0, 0.0), (8.0, 3.2, TOP + 0.45), "body")

    # Lot size
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(18.0, 10.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
