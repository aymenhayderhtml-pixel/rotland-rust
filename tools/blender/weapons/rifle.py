"""rifle: bolt-action hunting rifle with a wood stock, scope on rings and a bolt handle. Two-handed."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, hull_side, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "rifle", "weapons", 2500
GRIP = (0.0, -0.14, -0.05)  # the pistol grip under the receiver


def build(model):
    # Stock with a raised cheek line, its pistol grip and a butt pad
    model.add(hull_side([(-0.16, 0.055), (-0.55, 0.065), (-0.55, -0.065), (-0.30, -0.035), (-0.16, -0.02)],
                        0.042, "wood_crate"))
    model.add(hull_side([(-0.175, 0.0), (-0.105, 0.0), (-0.115, -0.10), (-0.175, -0.10)], 0.034, "wood_crate"))
    model.add(rbox((0.044, 0.02, 0.13), (0.0, -0.56, 0.0), "gun_black", 0.3))
    # Forend under the barrel
    model.add(hull_side([(-0.16, 0.02), (0.36, 0.016), (0.36, -0.018), (-0.16, -0.035)], 0.040, "wood_crate"))
    # Round receiver, bolt handle with a ball knob, magazine floor plate, trigger guard and trigger
    model.add(loft_y([(-0.15, 0.019), (0.10, 0.019)], "metal", z=0.035))
    model.add(box((0.040, 0.008, 0.008), (0.022, -0.10, 0.035), "metal"))
    model.add(rbox((0.014, 0.014, 0.014), (0.045, -0.10, 0.027), "metal", 0.35))
    model.add(rbox((0.030, 0.07, 0.012), (0.0, 0.02, -0.040), "gun_black", 0.3))
    model.add(box((0.010, 0.06, 0.007), (0.0, -0.09, -0.050), "gun_black"))
    model.add(box((0.006, 0.006, 0.024), (0.0, -0.09, -0.035), "metal"))
    # Barrel, thinner toward the crown
    model.add(loft_y([(0.10, 0.013), (0.30, 0.012), (0.60, 0.009), (0.615, 0.0095)], "gun_black", z=0.035))
    # Scope: bells at both ends, turrets, lenses and two rings
    model.add(loft_y([(-0.11, 0.022), (-0.08, 0.022), (-0.06, 0.015), (0.06, 0.015), (0.09, 0.024), (0.13, 0.024)],
                     "gun_black", z=0.105))
    model.add(cyl_y(0.019, 0.019, 0.129, 0.132, "water_blue", z=0.105, segments=12))
    model.add(cyl_y(0.018, 0.018, -0.112, -0.109, "water_blue", z=0.105, segments=12))
    model.add(rbox((0.014, 0.014, 0.016), (0.0, 0.0, 0.127), "gun_black", 0.3))
    model.add(rbox((0.016, 0.014, 0.014), (0.022, 0.0, 0.105), "gun_black", 0.3))
    for y in (-0.05, 0.05):
        model.add(box((0.012, 0.012, 0.05), (0.0, y, 0.075), "gun_black"))
    model.add_empty("grip_L", (0.0, 0.25, -0.02))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
