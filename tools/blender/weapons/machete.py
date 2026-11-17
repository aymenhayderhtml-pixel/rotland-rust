"""machete: wide upturned blade with a bright edge, riveted wooden handle and bolster. One hand."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, hull_side, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "machete", "weapons", 1000
GRIP = (0.0, -0.05, 0.0)  # middle of the handle


def build(model):
    model.add(loft_y([(-0.126, 0.010), (-0.122, 0.017), (-0.110, 0.018), (-0.050, 0.015), (0.0, 0.016), (0.02, 0.016)],
                     "bark_dark", segments=10))
    for y in (-0.10, -0.06, -0.02):
        model.add(box((0.036, 0.008, 0.008), (0.0, y, 0.0), "metal"))  # rivets through the handle
    model.add(cyl_y(0.018, 0.017, 0.012, 0.03, "metal", segments=10))  # bolster
    model.add(prism_yz([(0.02, 0.018), (0.30, 0.020), (0.42, 0.012), (0.47, 0.0), (0.44, -0.030), (0.30, -0.032),
                        (0.02, -0.024)], 0.007, "metal"))
    model.add(prism_yz([(0.03, -0.022), (0.30, -0.030), (0.44, -0.028), (0.47, 0.0), (0.448, -0.036), (0.30, -0.040),
                        (0.03, -0.030)], 0.004, "rock_light"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
