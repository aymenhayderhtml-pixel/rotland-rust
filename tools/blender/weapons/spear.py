"""spear: 2 m ash pole with a leaf-shaped steel head, cord lashing and wrapped grips. Two-handed."""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "spear", "weapons", 600
GRIP = (0.0, 0.0, 0.0)  # main hand grip position


def build(model):
    # Ash pole: 2m class weapon spanning y = -0.65 to +1.35
    model.add(cyl_y(0.014, 0.013, -0.640, 1.150, "wood_crate", segments=8))
    # Butt ferrule / iron heel
    model.add(cyl_y(0.015, 0.014, -0.660, -0.635, "metal", segments=8))
    model.add(rbox((0.024, 0.012, 0.024), (0.0, -0.655, 0.0), "gun_black", 0.3))

    # Main hand grip wrap (around y = 0.0)
    model.add(loft_y([(-0.080, 0.0165), (-0.070, 0.0175), (0.050, 0.0175), (0.060, 0.0165)],
                     "leather", segments=8))
    for y in (-0.04, 0.02):
        model.add(loft_y([(y, 0.0185), (y + 0.008, 0.0185)], "gun_black", segments=8))

    # Forward off-hand grip wrap (around y = 0.30)
    model.add(loft_y([(0.220, 0.0160), (0.230, 0.0170), (0.370, 0.0170), (0.380, 0.0160)],
                     "leather", segments=8))
    for y in (0.26, 0.34):
        model.add(loft_y([(y, 0.0180), (y + 0.008, 0.0180)], "gun_black", segments=8))

    # Heavy cord/sinew lashing securing blade tang to pole
    model.add(loft_y([(1.010, 0.0165), (1.020, 0.0180), (1.150, 0.0180), (1.160, 0.0165)],
                     "bark_dark", segments=8))
    for y in (1.04, 1.10):
        model.add(loft_y([(y, 0.0195), (y + 0.008, 0.0195)], "bark_dark", segments=8))

    # Leaf-shaped spearhead
    # Central spine / socket
    model.add(cyl_y(0.012, 0.004, 1.140, 1.340, "metal", segments=8))
    # Leaf blade contour: swells out then tapers to a sharp thrusting point
    blade_pts = [
        (1.140, 0.008),
        (1.210, 0.034),
        (1.280, 0.024),
        (1.365, 0.000),   # tip point
        (1.280, -0.024),
        (1.210, -0.034),
        (1.140, -0.008),
    ]
    model.add(prism_yz(blade_pts, 0.006, "metal"))
    # Sharpened cutting bevels along both edges
    edge_pts = [
        (1.145, 0.008),
        (1.210, 0.034),
        (1.280, 0.024),
        (1.365, 0.000),
        (1.350, 0.000),
        (1.275, 0.018),
        (1.210, 0.026),
        (1.155, 0.006),
    ]
    model.add(prism_yz(edge_pts, 0.004, "rock_light"))
    edge_pts_neg = [(y, -z) for y, z in edge_pts]
    model.add(prism_yz(edge_pts_neg, 0.004, "rock_light"))

    # Off-hand empty
    model.add_empty("grip_L", (0.0, 0.30, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
