"""hunting_rifle: long-barreled bolt-action hunting rifle with wooden stock, bolt handle and scope. Two-handed."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, cyl_y, hull_side, loft_y, rbox  # noqa: E402

NAME, CATEGORY, BUDGET = "hunting_rifle", "weapons", 600
GRIP = (0.0, -0.14, -0.05)  # pistol grip under the receiver


def build(model):
    # Wooden stock, pistol grip, butt pad and forend
    model.add(hull_side([(-0.15, 0.048), (-0.51, 0.058), (-0.51, -0.068), (-0.28, -0.035), (-0.15, -0.020)],
                        0.040, "bark_light"))
    model.add(hull_side([(-0.170, 0.0), (-0.105, 0.0), (-0.115, -0.095), (-0.170, -0.095)], 0.032, "bark_light"))
    model.add(rbox((0.042, 0.018, 0.126), (0.0, -0.519, -0.005), "gun_black", 0.3))
    model.add(hull_side([(-0.15, 0.020), (0.34, 0.015), (0.34, -0.016), (-0.15, -0.032)], 0.038, "bark_light"))

    # Receiver, bolt handle with knob, floor plate, trigger guard and trigger
    model.add(cyl_y(0.019, 0.019, -0.14, 0.09, "gun_black", z=0.035, segments=10))
    model.add(box((0.038, 0.008, 0.008), (0.022, -0.09, 0.035), "metal"))
    model.add(rbox((0.014, 0.014, 0.014), (0.044, -0.09, 0.026), "metal", 0.35))
    model.add(rbox((0.028, 0.070, 0.010), (0.0, 0.02, -0.038), "gun_black", 0.3))
    model.add(box((0.009, 0.058, 0.006), (0.0, -0.09, -0.050), "gun_black"))
    model.add(box((0.006, 0.006, 0.022), (0.0, -0.09, -0.035), "metal"))

    # Long slender barrel (~1.15 m total rifle length from butt pad to muzzle)
    model.add(loft_y([(0.09, 0.013), (0.32, 0.011), (0.62, 0.0085), (0.63, 0.0095)],
                     "gun_black", z=0.035, segments=10))
    model.add(box((0.006, 0.012, 0.012), (0.0, 0.60, 0.048), "brass"))

    # Scope on top with lenses, turrets and two mounting rings
    model.add(loft_y([(-0.11, 0.021), (-0.08, 0.021), (-0.06, 0.014), (0.06, 0.014), (0.09, 0.024), (0.14, 0.024)],
                     "gun_black", z=0.095, segments=10))
    model.add(cyl_y(0.019, 0.019, 0.139, 0.142, "water_blue", z=0.095, segments=10))
    model.add(cyl_y(0.017, 0.017, -0.112, -0.109, "water_blue", z=0.095, segments=10))
    model.add(box((0.012, 0.012, 0.014), (0.0, 0.0, 0.116), "brass"))
    model.add(box((0.014, 0.012, 0.012), (0.020, 0.0, 0.095), "brass"))
    for y in (-0.05, 0.05):
        model.add(box((0.012, 0.012, 0.046), (0.0, y, 0.068), "metal"))

    model.add_empty("grip_L", (0.0, 0.25, -0.02))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
