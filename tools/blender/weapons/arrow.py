"""arrow: wooden hunting arrow with a barbed metal broadhead, 3-fletch feathers and a notched nock.

Origin is at the tip (0, 0, 0), pointing forward (+Y) so projectile hit registration is exact.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "arrow", "weapons", 600
GRIP = (0.0, 0.0, 0.0)  # origin stays at the tip


def build(model):
    # Metal broadhead tip at y = 0.0 pointing +Y
    # Central ferrule sleeve around the shaft
    model.add(cyl_y(0.005, 0.002, -0.060, -0.015, "metal", segments=8))
    # Broadhead double-edged triangular cutting blade
    blade_pts = [
        (0.000, 0.000),      # sharp tip
        (-0.048, 0.014),     # top cutting edge
        (-0.062, 0.016),     # top barb
        (-0.052, 0.004),     # top notch
        (-0.052, -0.004),    # bottom notch
        (-0.062, -0.016),    # bottom barb
        (-0.048, -0.014),    # bottom cutting edge
    ]
    model.add(prism_yz(blade_pts, 0.0025, "metal"))
    # Bright sharpened cutting edge along the blade perimeter
    edge_pts = [
        (0.000, 0.000),
        (-0.048, 0.014),
        (-0.060, 0.0155),
        (-0.050, 0.011),
        (-0.020, 0.003),
        (-0.020, -0.003),
        (-0.050, -0.011),
        (-0.060, -0.0155),
        (-0.048, -0.014),
    ]
    model.add(prism_yz(edge_pts, 0.0035, "rock_light"))

    # Sinew / thread wrapping securing the broadhead to the wooden shaft
    model.add(cyl_y(0.0055, 0.0052, -0.075, -0.058, "bark_dark", segments=8))

    # Cedar wood shaft
    model.add(cyl_y(0.0042, 0.0042, -0.710, -0.075, "wood_crate", segments=8))

    # Fletching: 3 shield-cut feathers radially spaced at 120 degrees around the shaft
    feather_length = 0.12
    y_start = -0.690
    y_end = y_start + feather_length  # -0.570
    fletch_profile = [
        (y_start, 0.0042),
        (y_start + 0.015, 0.016),
        (y_end - 0.020, 0.016),
        (y_end, 0.0042),
    ]
    for deg in (0, 120, 240):
        bm_feather = prism_yz(fletch_profile, 0.0024, "white")
        rad = math.radians(deg)
        common.transform(bm_feather, rot=(0.0, rad, 0.0))
        model.add(bm_feather)

    # Front and rear thread bindings around the fletching
    model.add(cyl_y(0.0048, 0.0048, -0.700, -0.686, "bark_dark", segments=8))
    model.add(cyl_y(0.0048, 0.0048, -0.574, -0.560, "bark_dark", segments=8))

    # Notched nock at the tail
    model.add(cyl_y(0.0044, 0.0042, -0.730, -0.710, "gun_black", segments=8))
    # String slit cut
    model.add(box((0.003, 0.012, 0.008), (0.0, -0.725, 0.0), "gun_black"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
