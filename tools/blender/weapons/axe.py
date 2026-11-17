"""axe: wood-handled axe with a flared steel head, a bright bevel and a wrapped grip. Two-handed."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, hull_side, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "axe", "weapons", 1000
GRIP = (0.0, 0.05, 0.0)  # middle of the haft


def head_points():
    # Eye (wraps the haft top, centred in the head's height): flat side at x = -0.02,
    # short flat poll at z = -0.06, bevel side at x = +0.02 on the poll
    # The head sits at the far end: its forward reach must beat the butt's reach (WeaponGripConvention)
    back = [(-0.02, 0.41, -0.06), (-0.02, 0.47, -0.06), (0.02, 0.41, -0.06), (0.02, 0.47, -0.06)]
    eye_front = [(-0.02, 0.41, 0.04), (-0.02, 0.47, 0.04)]
    # Bit: a bulging edge on the flat side, with horns above and below the eye
    edge = [(-0.02, 0.49, 0.12), (-0.02, 0.38, 0.20), (-0.02, 0.27, 0.12)]
    return back + eye_front + edge


def build(model):
    # Haft: swelled knob at the butt, slim waist, thicker under the head
    model.add(loft_y([(-0.355, 0.012), (-0.350, 0.021), (-0.335, 0.022), (-0.30, 0.017), (0.0, 0.015),
                      (0.30, 0.016), (0.41, 0.018), (0.47, 0.018)], "wood_crate", segments=10))
    model.add(loft_y([(-0.29, 0.0185), (-0.13, 0.0175)], "bark_dark", segments=10))  # grip wrap
    for y in (-0.29, -0.13):
        model.add(loft_y([(y - 0.006, 0.020), (y + 0.006, 0.020)], "gun_black", segments=10))
    head = convex(head_points(), "metal")
    # The bevel faces (normal pointing +x) are the bright cutting edge, so they read at small sizes
    bevel = [f for f in head.faces if f.normal.x > 0.3]
    common.paint_faces(head, bevel, "rock_light")
    model.add(head)
    model.add(box((0.044, 0.012, 0.012), (0.0, 0.44, 0.045), "gun_black"))  # wedge on top of the eye
    model.add_empty("grip_L", (0.0, -0.15, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
