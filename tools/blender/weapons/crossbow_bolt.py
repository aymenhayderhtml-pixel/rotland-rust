"""crossbow_bolt: 0.4 m crossbow bolt with a metal tip, dark shaft, 3 small fins and brass nock.

Origin is at the tip (0, 0, 0), pointing forward (+Y), matching arrow.py.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import cyl_y, prism_yz  # noqa: E402

NAME, CATEGORY, BUDGET = "crossbow_bolt", "weapons", 600
GRIP = (0.0, 0.0, 0.0)  # origin stays at the tip


def build(model):
    # Pointed metal tip and ferrule collar
    model.add(cyl_y(0.005, 0.0, -0.040, 0.0, "metal", segments=8))
    model.add(cyl_y(0.005, 0.005, -0.055, -0.040, "rock_light", segments=8))
    blade_pts = [
        (0.000, 0.000),
        (-0.036, 0.010),
        (-0.044, 0.003),
        (-0.044, -0.003),
        (-0.036, -0.010),
    ]
    model.add(prism_yz(blade_pts, 0.0025, "metal"))

    # Carbon/dark shaft
    model.add(cyl_y(0.0045, 0.0045, -0.390, -0.055, "gun_black", segments=8))

    # 3 small radial fins near the tail
    fin_profile = [
        (-0.382, 0.0045),
        (-0.372, 0.015),
        (-0.322, 0.015),
        (-0.312, 0.0045),
    ]
    for deg in (0, 120, 240):
        bm_fin = prism_yz(fin_profile, 0.0022, "red_paint")
        common.transform(bm_fin, rot=(0.0, math.radians(deg), 0.0))
        model.add(bm_fin)

    # Brass flat nock at the tail (total length 0.40 m)
    model.add(cyl_y(0.0048, 0.0045, -0.400, -0.390, "brass", segments=8))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
