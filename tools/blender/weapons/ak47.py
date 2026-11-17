"""ak47: AK-47 with wood furniture, dust cover, gas tube, curved magazine and front sight block. Two-handed.

The magazine is its own node named `mag`, so the reload clip can hide it during the swap.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, hull_side, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "ak47", "weapons", 2500
GRIP = (0.0, 0.0, 0.0)  # center of the pistol grip


def build(model):
    # Receiver, stamped dust cover on top, selector lever and charging handle on the right
    model.add(rbox((0.050, 0.22, 0.070), (0.0, 0.10, 0.060), "gun_black", 0.15))
    model.add(rbox((0.044, 0.20, 0.022), (0.0, 0.09, 0.102), "metal", 0.4))
    model.add(box((0.004, 0.10, 0.012), (0.027, 0.08, 0.075), "metal"))
    model.add(box((0.012, 0.035, 0.008), (0.030, 0.13, 0.090), "metal"))
    model.add(rbox((0.016, 0.010, 0.010), (0.040, 0.15, 0.090), "metal", 0.3))
    # Rear sight block, gas tube with an upper wood guard, and the barrel
    model.add(rbox((0.026, 0.035, 0.022), (0.0, 0.215, 0.110), "gun_black", 0.25))
    model.add(cyl_y(0.010, 0.010, 0.21, 0.43, "metal", z=0.100, segments=10))
    model.add(rbox((0.036, 0.13, 0.026), (0.0, 0.29, 0.103), "wood_crate", 0.35))
    model.add(cyl_y(0.008, 0.008, 0.21, 0.56, "metal", z=0.070, segments=10))
    # Gas block, front sight block with its post and ears, and the slanted muzzle brake
    model.add(rbox((0.024, 0.030, 0.050), (0.0, 0.42, 0.085), "gun_black", 0.25))
    model.add(rbox((0.022, 0.030, 0.026), (0.0, 0.53, 0.078), "gun_black", 0.25))
    model.add(box((0.005, 0.005, 0.030), (0.0, 0.535, 0.100), "gun_black"))
    for x in (-0.010, 0.010):
        model.add(box((0.004, 0.012, 0.026), (x, 0.535, 0.100), "gun_black"))
    model.add(loft_y([(0.56, 0.011), (0.60, 0.011), (0.605, 0.008)], "gun_black", z=0.070, segments=10))
    # Lower wood handguard with grip grooves
    model.add(rbox((0.052, 0.18, 0.050), (0.0, 0.27, 0.050), "wood_crate", 0.3))
    for y in (0.22, 0.25, 0.28, 0.31):
        model.add(box((0.054, 0.012, 0.006), (0.0, y, 0.045), "bark_dark"))
    # Stock: shallow at the wrist, deep at the heel, with a steel butt plate
    model.add(hull_side([(-0.03, 0.085), (-0.40, 0.090), (-0.40, -0.010), (-0.10, 0.032), (-0.03, 0.036)],
                        0.034, "wood_crate"))
    model.add(rbox((0.038, 0.014, 0.104), (0.0, -0.407, 0.040), "gun_black", 0.3))
    # Bakelite pistol grip (the hand), trigger guard and trigger
    model.add(hull_side([(-0.020, 0.030), (0.025, 0.030), (0.005, -0.060), (-0.045, -0.060)], 0.034, "bark_dark"))
    model.add(box((0.010, 0.060, 0.007), (0.0, 0.055, -0.010), "gun_black"))
    model.add(box((0.010, 0.007, 0.034), (0.0, 0.085, 0.008), "gun_black"))
    model.add(box((0.006, 0.006, 0.022), (0.0, 0.050, 0.012), "metal"))
    # Curved 30-round magazine with ribs and a base plate
    mag_outline = [(0.03, 0.02), (0.09, 0.02), (0.10, -0.08), (0.08, -0.17),
                   (0.05, -0.18), (0.04, -0.10), (0.03, -0.02)]
    model.add_part("mag", prism_yz(mag_outline, 0.032, "metal"))
    model.add_empty("grip_L", (0.0, 0.30, -0.01))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
