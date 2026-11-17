"""road_sign: a bent metal pole with a red-rimmed sign on top (<= 250 tris). Has a collider on the pole."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "road_sign", "props", 400


def build(model):
    # Straight lower pole, then an upper piece bent forward at 0.25 rad from its top
    lower = common.make_box((0.08, 0.08, 2.0), loc=(0.0, 0.0, 1.0), color="metal")
    upper = common.make_box((0.08, 0.08, 0.9), loc=(0.0, 0.0, 0.45), color="metal")
    common.transform(upper, rot=(0.25, 0.0, 0.0), loc=(0.0, 0.0, 2.0))
    model.add(lower)
    model.add(upper)
    # The sign: red back plate, white face, tilted with the bend. Its centre sits at the bent top.
    top = (0.0, -0.9 * math.sin(0.25) * 0.5, 2.0 + 0.9 * math.cos(0.25) * 0.5)
    back = common.make_box((0.8, 0.03, 0.8), loc=(0.0, 0.0, 0.0), color="red_paint")
    face = common.make_box((0.66, 0.05, 0.66), loc=(0.0, -0.01, 0.0), color="white")
    holes = [common.make_box((0.04, 0.02, 0.04), loc=(x, -0.04, z), color="gun_black")
             for x, z in ((-0.15, 0.12), (0.18, -0.05), (0.05, 0.2), (-0.08, -0.18))]
    bolts = [common.make_box((0.03, 0.03, 0.03), loc=(0.0, -0.045, z), color="metal") for z in (-0.25, 0.25)]
    sign = common.combine([back, face] + holes + bolts)
    common.transform(sign, rot=(0.25, 0.0, 0.0), loc=(top[0], top[1] - 0.05, top[2] + 0.45))
    model.add(sign)
    model.add_collider("pole", common.make_box((0.14, 0.14, 2.8), loc=(0.0, 0.0, 1.4), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
