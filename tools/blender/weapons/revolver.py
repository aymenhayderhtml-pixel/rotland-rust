"""revolver: 6-shot revolver with a wooden grip, round cylinder, long barrel, trigger guard and hammer. One hand."""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, cyl_y, hull_side, rbox  # noqa: E402

NAME, CATEGORY, BUDGET = "revolver", "weapons", 600
GRIP = (0.0, 0.0, 0.0)  # trigger area, matching pistol.py


def build(model):
    # Wooden revolver grip and metal butt cap
    model.add(hull_side([(-0.075, 0.014), (-0.018, 0.018), (-0.032, -0.068), (-0.095, -0.078)],
                        0.028, "bark_dark"))
    model.add(rbox((0.024, 0.038, 0.010), (0.0, -0.062, -0.076), "gun_black", 0.25))

    # Frame: lower frame, recoil shield behind cylinder, and top strap
    model.add(rbox((0.024, 0.115, 0.020), (0.0, 0.018, 0.030), "metal", 0.2))
    model.add(rbox((0.026, 0.042, 0.042), (0.0, -0.038, 0.052), "metal", 0.2))
    model.add(box((0.016, 0.068, 0.010), (0.0, 0.015, 0.078), "metal"))

    # Round 6-shot cylinder with flute ribs and center arbor pin
    model.add(cyl_y(0.021, 0.021, -0.014, 0.042, "gun_black", z=0.053, segments=12))
    model.add(cyl_y(0.005, 0.005, 0.042, 0.075, "metal", z=0.053, segments=6))
    for i in range(6):
        ang = i * math.pi / 3.0
        bm_rib = common.make_box((0.004, 0.048, 0.004), color="metal")
        common.transform(bm_rib, loc=(math.cos(ang) * 0.021, 0.014, 0.053 + math.sin(ang) * 0.021))
        model.add(bm_rib)

    # Long barrel, underlug shroud and ventilated top rib
    model.add(cyl_y(0.010, 0.010, 0.045, 0.185, "metal", z=0.058, segments=10))
    model.add(rbox((0.014, 0.130, 0.014), (0.0, 0.110, 0.043), "gun_black", 0.2))
    model.add(box((0.010, 0.135, 0.007), (0.0, 0.112, 0.070), "gun_black"))

    # Trigger guard and trigger
    model.add(box((0.009, 0.046, 0.006), (0.0, 0.006, -0.012), "metal"))
    model.add(box((0.009, 0.006, 0.028), (0.0, 0.028, 0.002), "metal"))
    model.add(box((0.005, 0.006, 0.018), (0.0, 0.0, 0.006), "gun_black"))

    # Hammer and sights
    model.add(box((0.008, 0.018, 0.016), (0.0, -0.062, 0.068), "gun_black"))
    model.add(box((0.014, 0.010, 0.007), (0.0, -0.020, 0.084), "gun_black"))
    model.add(box((0.005, 0.014, 0.011), (0.0, 0.175, 0.078), "brass"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
