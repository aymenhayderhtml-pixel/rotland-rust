"""crowbar: red hex bar with a curved split claw, a chisel end and a rubber grip. Two-handed."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, hull_side, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "crowbar", "weapons", 1000
GRIP = (0.0, 0.0, 0.0)  # middle of the bar, where the hands meet


def build(model):
    model.add(cyl_y(0.014, 0.014, -0.30, 0.34, "red_paint", segments=6))  # hex shaft
    model.add(hull_side([(-0.30, 0.013), (-0.345, 0.003), (-0.345, -0.003), (-0.30, -0.013)], 0.028, "metal"))  # chisel
    # Ribbed rubber grip
    model.add(loft_y([(-0.142, 0.018), (-0.135, 0.020), (0.095, 0.020), (0.102, 0.018)], "gun_black", segments=10))
    for y in (-0.10, -0.05, 0.0, 0.05):
        model.add(loft_y([(y, 0.0215), (y + 0.012, 0.0215)], "gun_black", segments=10))
    # Curved hook, then the bare steel claw tip
    hook = [(0.31, 0.012), (0.36, 0.016), (0.42, 0.012), (0.46, -0.005), (0.455, -0.030), (0.425, -0.020),
            (0.42, -0.005), (0.39, 0.0), (0.33, -0.012)]
    model.add(prism_yz(hook, 0.026, "red_paint"))
    model.add(prism_yz([(0.455, -0.030), (0.43, -0.046), (0.41, -0.040), (0.425, -0.020)], 0.027, "metal"))
    model.add_empty("grip_L", (0.0, -0.20, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
