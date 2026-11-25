"""pine_snow_lod1: medium LOD for snow-laden pine (<= 160 tris)."""
import os
import random
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402
import veg  # noqa: E402

NAME, CATEGORY, BUDGET = "pine_snow_lod1", "props", 160
SINK_Z = -0.05


def build(model):
    rng = random.Random(3107)
    model.add(common.make_frustum(0.25, 0.14, SINK_Z, 3.2, segments=5, color="bark_dark"))
    model.add_collider("trunk", common.make_frustum(0.24, 0.20, SINK_Z, 2.8, segments=6, color="bark_dark"))

    # 4 jagged pine tiers (points=5 -> 20 tris each) with simple 6-sided snow cap cones (10 tris each)
    tiers = [
        (2.55, 1.80, 2.50),
        (2.00, 3.40, 2.30),
        (1.42, 5.00, 2.10),
        (0.84, 6.60, 2.00),
    ]
    for i, (radius, z0, height) in enumerate(tiers):
        off = (rng.uniform(-0.06, 0.06), rng.uniform(-0.06, 0.06))
        model.add(veg.tier(radius, z0, height, "pine_snow_dark", seed=100 + 13 * i,
                           points=5, droop=0.28, inner=0.66, center=off))
        cap_z0 = z0 + 0.36 * height
        cap_z1 = z0 + height + 0.15
        model.add(common.make_frustum(radius * 0.72, 0.0, cap_z0, cap_z1,
                                      segments=6, center=off, rot_z=0.5 * i, color="snow_white"))

    # Snowy tip cone (8 tris)
    model.add(common.make_frustum(0.38, 0.0, 8.20, 9.75, segments=5, color="snow_white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
