"""pipe: heavy galvanized steel pipe with threaded ends, an end coupling and a ragged duct tape grip. One hand."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "pipe", "weapons", 600
GRIP = (0.0, -0.080, 0.0)  # center of the taped grip


def build(model):
    # Main steel pipe shaft (from y = -0.21 to y = 0.43, total 0.64m)
    model.add(cyl_y(0.017, 0.017, -0.210, 0.430, "metal", segments=8))

    # Hollow-end interior recesses
    model.add(cyl_y(0.013, 0.013, -0.212, -0.208, "gun_black", segments=8))
    model.add(cyl_y(0.013, 0.013, 0.428, 0.432, "gun_black", segments=8))

    # Threaded pipe ends (ridges)
    for y in (-0.200, -0.188):
        model.add(loft_y([(y - 0.002, 0.0178), (y + 0.002, 0.0178)], "rock_mid", segments=8))
    for y in (0.358, 0.370):
        model.add(loft_y([(y - 0.002, 0.0178), (y + 0.002, 0.0178)], "rock_mid", segments=8))

    # Heavy threaded pipe coupling / collar near the striking end for weight
    model.add(cyl_y(0.024, 0.024, 0.375, 0.425, "rock_dark", segments=8))
    model.add(rbox((0.050, 0.042, 0.050), (0.0, 0.400, 0.0), "metal", 0.35))
    model.add(cyl_y(0.020, 0.016, 0.425, 0.435, "rock_mid", segments=8))

    # Base rubber/duct tape grip wrap around the handle area
    model.add(loft_y([(-0.185, 0.0185), (-0.175, 0.0195), (0.035, 0.0195), (0.045, 0.0185)],
                     "gun_black", segments=8))
    # Overlapping spiral tape ridges
    for y in (-0.14, -0.08, -0.02):
        model.add(loft_y([(y - 0.005, 0.0210), (y + 0.005, 0.0210)], "gun_black", segments=8))
    # Frayed / peeling tape end flap
    model.add(box((0.024, 0.012, 0.005), (0.008, 0.048, 0.016), "canvas_khaki"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
