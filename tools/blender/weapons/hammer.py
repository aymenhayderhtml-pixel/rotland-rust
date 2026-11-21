"""hammer: claw hammer with a steel head, curved split claw, striking poll and rubberized handle. Two-handed."""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "hammer", "weapons", 600
GRIP = (0.0, -0.050, 0.0)  # main hand grip


def build(model):
    # Handle: contoured wood haft with rubber grip
    model.add(loft_y([(-0.310, 0.014), (-0.300, 0.019), (-0.280, 0.018), (-0.050, 0.016),
                      (0.200, 0.017), (0.310, 0.020), (0.355, 0.020)], "wood_crate", segments=8))
    # Rubberized grip sleeve at bottom
    model.add(loft_y([(-0.295, 0.0205), (-0.040, 0.0185)], "gun_black", segments=8))
    for y in (-0.26, -0.21, -0.16, -0.11, -0.06):
        model.add(loft_y([(y, 0.022), (y + 0.010, 0.022)], "gun_black", segments=8))

    # Head eye wrapping the top of the handle
    model.add(rbox((0.038, 0.065, 0.044), (0.0, 0.325, 0.0), "metal", 0.25))
    model.add(box((0.016, 0.008, 0.012), (0.0, 0.352, 0.0), "rock_dark"))  # top wedge

    # Striking poll neck extending in +Z from the eye
    model.add(common.make_frustum(0.015, 0.014, 0.020, 0.062, segments=8, center=(0.0, 0.325), color="metal"))
    # Hardened round/octagonal striking face at the poll crown
    model.add(common.make_frustum(0.0155, 0.016, 0.062, 0.075, segments=8, center=(0.0, 0.325), color="rock_light"))

    # Curved split claw (extends in -Z and curves backward towards -Y)
    # Left claw tine (x < 0)
    claw_pts_l = [
        (-0.016, 0.310, -0.018), (-0.003, 0.310, -0.018),
        (-0.016, 0.340, -0.018), (-0.003, 0.340, -0.018),
        (-0.015, 0.290, -0.055), (-0.004, 0.290, -0.055),
        (-0.014, 0.240, -0.085), (-0.005, 0.245, -0.085),
        (-0.013, 0.220, -0.098), (-0.005, 0.225, -0.098),  # prying tip
    ]
    model.add(convex(claw_pts_l, "metal"))
    # Right claw tine (x > 0, mirror of left)
    claw_pts_r = [
        (0.003, 0.310, -0.018), (0.016, 0.310, -0.018),
        (0.003, 0.340, -0.018), (0.016, 0.340, -0.018),
        (0.004, 0.290, -0.055), (0.015, 0.290, -0.055),
        (0.005, 0.245, -0.085), (0.014, 0.240, -0.085),
        (0.005, 0.225, -0.098), (0.013, 0.220, -0.098),
    ]
    model.add(convex(claw_pts_r, "metal"))

    # Sharpened bevel on claw tips for nail pulling
    model.add(box((0.026, 0.006, 0.010), (0.0, 0.222, -0.096), "rock_light"))

    # Off-hand grip
    model.add_empty("grip_L", (0.0, -0.250, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
