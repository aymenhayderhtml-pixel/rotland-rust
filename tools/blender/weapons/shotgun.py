"""shotgun: pump-action with a shaped wood stock, ribbed pump, tube magazine and bead sight. Two-handed."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, hull_side, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "shotgun", "weapons", 2000
GRIP = (0.0, -0.20, 0.0)  # on the stock wrist, behind the receiver


def build(model):
    # Stock: thin at the wrist, deep at the butt, with a rubber recoil pad
    model.add(hull_side([(-0.105, 0.040), (-0.44, 0.050), (-0.44, -0.075), (-0.105, -0.025)], 0.042, "wood_crate"))
    model.add(rbox((0.045, 0.022, 0.128), (0.0, -0.45, -0.0125), "gun_black", 0.3))
    # Receiver with an ejection port, trigger guard and trigger
    model.add(rbox((0.050, 0.22, 0.080), (0.0, 0.0, 0.010), "metal", 0.15))
    model.add(box((0.004, 0.07, 0.03), (0.026, 0.02, 0.025), "gun_black"))
    model.add(box((0.010, 0.07, 0.007), (0.0, -0.05, -0.045), "gun_black"))
    model.add(box((0.010, 0.007, 0.03), (0.0, -0.016, -0.032), "gun_black"))
    model.add(box((0.006, 0.006, 0.024), (0.0, -0.045, -0.026), "metal"))
    # Barrel with a muzzle ring, the tube magazine under it and a clamp joining them
    model.add(cyl_y(0.012, 0.012, 0.11, 0.58, "gun_black", z=0.035, segments=12))
    model.add(cyl_y(0.0135, 0.0135, 0.565, 0.58, "gun_black", z=0.035, segments=12))
    model.add(cyl_y(0.010, 0.010, 0.11, 0.50, "metal", z=-0.008, segments=12))
    model.add(cyl_y(0.011, 0.011, 0.49, 0.51, "gun_black", z=-0.008, segments=12))
    model.add(rbox((0.028, 0.016, 0.062), (0.0, 0.47, 0.014), "gun_black", 0.3))
    model.add(box((0.006, 0.006, 0.008), (0.0, 0.57, 0.050), "brass"))  # bead sight
    # Ribbed wooden pump
    model.add(rbox((0.058, 0.14, 0.060), (0.0, 0.27, -0.008), "wood_crate", 0.3))
    for y in (0.22, 0.245, 0.27, 0.295, 0.32):
        model.add(box((0.060, 0.006, 0.044), (0.0, y, -0.008), "bark_dark"))
    model.add_empty("grip_L", (0.0, 0.27, 0.0))
    model.add_empty("mount_optic", (0.0, 0.0, 0.050))
    model.add_empty("mount_grip", (0.0, 0.27, -0.038))
    model.add_empty("mount_stock", (0.0, -0.461, -0.0125))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
