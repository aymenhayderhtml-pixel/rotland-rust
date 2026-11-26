"""smg: compact submachine gun with a ribbed handguard, folded stock and straight magazine. Two-handed."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, hull_side, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "smg", "weapons", 2000
GRIP = (0.0, 0.0, 0.0)  # center of the pistol grip


def build(model):
    # Receiver with a cocking tube on top, rear sight drum and front sight hood
    model.add(rbox((0.048, 0.26, 0.060), (0.0, 0.07, 0.045), "metal", 0.25))
    model.add(cyl_y(0.008, 0.008, 0.05, 0.25, "metal", x=-0.012, z=0.082, segments=10))
    model.add(rbox((0.020, 0.018, 0.022), (0.0, -0.035, 0.088), "gun_black", 0.35))
    model.add(box((0.022, 0.012, 0.026), (0.0, 0.24, 0.088), "gun_black"))
    # Ribbed handguard, barrel and muzzle
    model.add(rbox((0.052, 0.10, 0.050), (0.0, 0.21, 0.032), "gun_black", 0.35))
    for y in (0.175, 0.20, 0.225, 0.25):
        model.add(box((0.054, 0.008, 0.036), (0.0, y, 0.030), "rock_dark"))
    model.add(cyl_y(0.009, 0.009, 0.26, 0.35, "gun_black", z=0.040, segments=10))
    model.add(cyl_y(0.012, 0.012, 0.33, 0.35, "gun_black", z=0.040, segments=10))
    # Pistol grip (the hand), trigger guard and trigger
    model.add(hull_side([(-0.025, 0.020), (0.020, 0.020), (0.005, -0.075), (-0.040, -0.075)], 0.034, "gun_black"))
    model.add(box((0.010, 0.050, 0.007), (0.0, 0.045, -0.035), "gun_black"))
    model.add(box((0.010, 0.007, 0.040), (0.0, 0.070, -0.017), "gun_black"))
    model.add(box((0.006, 0.006, 0.020), (0.0, 0.035, -0.010), "metal"))
    # Magazine well and the straight magazine with a base plate
    model.add(rbox((0.040, 0.060, 0.040), (0.0, 0.13, 0.000), "metal", 0.25))
    model.add(rbox((0.030, 0.045, 0.150), (0.0, 0.13, -0.090), "gun_black", 0.25))
    model.add(rbox((0.034, 0.050, 0.010), (0.0, 0.13, -0.168), "rock_dark", 0.3))
    # Folded stock: two rods along the sides and a butt plate
    for x in (-0.022, 0.022):
        model.add(box((0.006, 0.20, 0.008), (x, -0.15, 0.050), "metal"))
    model.add(rbox((0.050, 0.015, 0.050), (0.0, -0.25, 0.040), "gun_black", 0.3))
    model.add_empty("grip_L", (0.0, 0.18, -0.01))
    model.add_empty("mount_optic", (0.0, 0.03, 0.075))
    model.add_empty("mount_muzzle", (0.0, 0.35, 0.040))
    model.add_empty("mount_magazine", (0.0, 0.13, -0.110))
    model.add_empty("mount_grip", (0.0, 0.21, 0.007))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
