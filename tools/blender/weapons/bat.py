"""bat: wooden baseball bat with a knob, taped handle and rusty nails near the tip. Two-handed."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, hull_side, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "bat", "weapons", 1000
GRIP = (0.0, 0.0, 0.0)  # middle of the handle


def build(model):
    # One turned profile: knob, thin handle, swelling barrel and a rounded end
    model.add(loft_y([(-0.170, 0.008), (-0.168, 0.029), (-0.155, 0.030), (-0.150, 0.018), (0.10, 0.020),
                      (0.25, 0.028), (0.45, 0.040), (0.62, 0.046), (0.685, 0.044), (0.70, 0.030), (0.703, 0.010)],
                     "wood_crate"))
    # Black grip tape with raised wraps
    model.add(loft_y([(-0.146, 0.0215), (0.06, 0.0225)], "gun_black"))
    for y in (-0.12, -0.07, -0.02, 0.03):
        model.add(loft_y([(y, 0.0235), (y + 0.008, 0.0235)], "gun_black", segments=10))
    # Nails driven through the barrel
    for y, axis in ((0.50, "x"), (0.55, "z"), (0.60, "-x"), (0.64, "-z"), (0.47, "-z")):
        size = (0.10, 0.004, 0.004) if "x" in axis else (0.004, 0.004, 0.10)
        model.add(box(size, (0.0, y, 0.0), "metal"))
    model.add_empty("grip_L", (0.0, -0.10, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
