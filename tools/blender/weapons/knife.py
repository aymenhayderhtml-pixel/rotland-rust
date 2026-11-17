"""knife: combat knife with a grooved handle, steel guard, pommel and a bevelled blade. One hand."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, hull_side, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "knife", "weapons", 800
GRIP = (0.0, -0.04, 0.0)  # middle of the handle


def build(model):
    model.add(loft_y([(-0.090, 0.012), (-0.080, 0.014), (-0.062, 0.012), (-0.045, 0.013), (-0.028, 0.012),
                      (-0.010, 0.013), (0.0, 0.012)], "gun_black", segments=10))
    model.add(loft_y([(-0.098, 0.008), (-0.094, 0.013), (-0.088, 0.013)], "metal", segments=10))  # pommel
    model.add(rbox((0.014, 0.008, 0.050), (0.0, 0.004, 0.0), "metal", 0.3))  # guard
    # Blade: thick steel spine, then a thin bright edge (reads as a bevel)
    model.add(prism_yz([(0.008, 0.013), (0.11, 0.013), (0.165, 0.002), (0.13, -0.004), (0.008, -0.004)], 0.0065, "metal"))
    model.add(prism_yz([(0.008, -0.004), (0.13, -0.004), (0.165, 0.002), (0.12, -0.013), (0.008, -0.013)], 0.004, "rock_light"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
