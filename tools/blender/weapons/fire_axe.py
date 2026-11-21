"""fire_axe: pick-head fire axe with a bright red head, ground cutting bit, rear pick and long handle. Two-handed."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "fire_axe", "weapons", 600
GRIP = (0.0, 0.080, 0.0)  # main hand grip position


def build(model):
    # Long curved axe handle: fawn-foot butt, slim waist, flared eye (from y = -0.44 to y = 0.49, length 0.93m)
    model.add(loft_y([(-0.435, 0.014), (-0.425, 0.023), (-0.410, 0.024), (-0.380, 0.018), (-0.100, 0.016),
                      (0.150, 0.016), (0.350, 0.018), (0.420, 0.021), (0.485, 0.020)], "wood_crate", segments=8))
    # Non-slip rubber grip sleeve at the butt
    model.add(loft_y([(-0.415, 0.022), (-0.180, 0.019)], "gun_black", segments=8))
    for y in (-0.39, -0.34, -0.29, -0.24, -0.19):
        model.add(loft_y([(y - 0.005, 0.0235), (y + 0.005, 0.0235)], "gun_black", segments=8))

    # Eye of the forged steel head
    eye_pts = [
        (-0.022, 0.420, -0.035), (0.022, 0.420, -0.035),
        (-0.022, 0.475, -0.035), (0.022, 0.475, -0.035),
        (-0.022, 0.420, 0.040), (0.022, 0.420, 0.040),
        (-0.022, 0.475, 0.040), (0.022, 0.475, 0.040),
    ]
    model.add(convex(eye_pts, "red_paint"))
    model.add(box((0.040, 0.014, 0.016), (0.0, 0.450, 0.045), "metal"))  # top eye wedge

    # Broad flared cutting blade (extends in +Z)
    bit_body = [
        (-0.020, 0.425, 0.035), (0.020, 0.425, 0.035),
        (-0.020, 0.470, 0.035), (0.020, 0.470, 0.035),
        (-0.012, 0.380, 0.125), (0.012, 0.380, 0.125),
        (-0.012, 0.515, 0.125), (0.012, 0.515, 0.125),
    ]
    model.add(convex(bit_body, "red_paint"))

    # Ground cutting edge and horns at the blade crown (extends to z = 0.155)
    edge_pts = [
        (-0.010, 0.380, 0.125), (0.010, 0.380, 0.125),
        (-0.010, 0.515, 0.125), (0.010, 0.515, 0.125),
        (-0.003, 0.360, 0.155), (0.003, 0.360, 0.155),  # bottom horn & sharp edge
        (-0.003, 0.535, 0.155), (0.003, 0.535, 0.155),  # top horn & sharp edge
    ]
    model.add(convex(edge_pts, "rock_light"))

    # Pick on the back of the head (extends in -Z, tapering to sharp point)
    pick_body = [
        (-0.018, 0.430, -0.030), (0.018, 0.430, -0.030),
        (-0.018, 0.465, -0.030), (0.018, 0.465, -0.030),
        (-0.010, 0.435, -0.085), (0.010, 0.435, -0.085),
        (-0.010, 0.460, -0.085), (0.010, 0.460, -0.085),
    ]
    model.add(convex(pick_body, "red_paint"))

    # Hardened steel pick point
    pick_tip = [
        (-0.009, 0.436, -0.083), (0.009, 0.436, -0.083),
        (-0.009, 0.459, -0.083), (0.009, 0.459, -0.083),
        (-0.003, 0.440, -0.125), (0.003, 0.440, -0.125),
        (-0.003, 0.455, -0.125), (0.003, 0.455, -0.125),
    ]
    model.add(convex(pick_tip, "rock_light"))

    # Off-hand grip empty (0.25m below main grip)
    model.add_empty("grip_L", (0.0, -0.170, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
