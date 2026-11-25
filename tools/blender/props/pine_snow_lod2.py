"""pine_snow_lod2: far low-poly LOD for snow-laden pine (<= 40 tris)."""
import math
import os
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "pine_snow_lod2", "props", 40
SINK_Z = -0.05


def build(model):
    # 4-sided trunk (12 tris)
    model.add(common.make_frustum(0.25, 0.14, SINK_Z, 3.2, segments=4, color="bark_dark"))
    model.add_collider("trunk", common.make_frustum(0.24, 0.20, SINK_Z, 2.8, segments=6, color="bark_dark"))

    # 2 pine cones + 2 snow cap cones (4-sided, 6 tris each -> 24 tris; total 36 tris)
    cones = [
        (2.45, 1.80, 5.40, 0.0, "pine_snow_dark"),
        (1.65, 3.20, 5.60, 0.0, "snow_white"),
        (1.55, 4.80, 8.60, math.pi / 4.0, "pine_snow_dark"),
        (0.95, 6.60, 9.75, math.pi / 4.0, "snow_white"),
    ]
    for radius, z0, z1, rot_z, color in cones:
        model.add(common.make_frustum(radius, 0.0, z0, z1, segments=4, rot_z=rot_z, color=color))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
