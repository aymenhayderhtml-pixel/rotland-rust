"""pistol: handgun with a serrated slide, rail, wood grip panels and sights (visual plan step 3). One hand."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, hull_side, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "pistol", "weapons", 1500
GRIP = (0.0, 0.0, 0.0)  # the trigger, just in front of the grip


def build(model):
    # Slide with rear serrations, an ejection port and the barrel crown at the front
    model.add(rbox((0.032, 0.20, 0.036), (0.0, 0.0, 0.058), "metal", 0.18))
    for y in (-0.088, -0.079, -0.070, -0.061):
        model.add(box((0.034, 0.004, 0.024), (0.0, y, 0.058), "gun_black"))
    model.add(box((0.014, 0.034, 0.004), (0.006, 0.025, 0.0765), "gun_black"))
    model.add(cyl_y(0.0085, 0.0085, 0.098, 0.118, "gun_black", z=0.058, segments=12))
    # Frame with an accessory rail under the muzzle
    model.add(rbox((0.029, 0.17, 0.022), (0.0, 0.008, 0.030), "gun_black", 0.2))
    for y in (0.050, 0.065, 0.080):
        model.add(box((0.031, 0.006, 0.006), (0.0, y, 0.017), "gun_black"))
    # Angled grip, wooden side panels and a magazine base
    model.add(hull_side([(-0.070, 0.022), (-0.018, 0.022), (-0.032, -0.072), (-0.088, -0.072)], 0.030, "gun_black"))
    model.add(hull_side([(-0.066, 0.010), (-0.027, 0.010), (-0.037, -0.058), (-0.080, -0.058)], 0.034, "bark_dark"))
    model.add(rbox((0.034, 0.060, 0.010), (0.0, -0.061, -0.077), "gun_black", 0.3))
    # Trigger guard and trigger
    model.add(box((0.009, 0.052, 0.007), (0.0, 0.006, -0.012), "gun_black"))
    model.add(box((0.009, 0.007, 0.030), (0.0, 0.032, 0.004), "gun_black"))
    model.add(box((0.006, 0.006, 0.020), (0.0, 0.0, 0.006), "metal"))
    # Sights and hammer
    model.add(box((0.006, 0.010, 0.009), (0.0, 0.090, 0.080), "gun_black"))
    for x in (-0.009, 0.009):
        model.add(box((0.006, 0.010, 0.010), (x, -0.088, 0.080), "gun_black"))
    model.add(box((0.010, 0.010, 0.016), (0.0, -0.101, 0.068), "gun_black"))
    model.add_empty("mount_optic", (0.0, -0.035, 0.076))
    model.add_empty("mount_muzzle", (0.0, 0.118, 0.058))
    model.add_empty("mount_magazine", (0.0, -0.061, -0.077))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
