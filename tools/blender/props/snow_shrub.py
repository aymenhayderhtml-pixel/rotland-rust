"""snow_shrub: small snow-covered bush with pine_snow_dark branches poking out of a snow_white dome (<= 150 tris)."""
import math
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

NAME, CATEGORY, BUDGET = "snow_shrub", "props", 150
SINK_Z = -0.05


def build(model):
    rng = random.Random(5201)

    # Low snow-blanketed bush dome (80 tris)
    dome = common.make_icosphere(
        subdiv=2, jitter=0.18, seed=41,
        radius=(0.56, 0.50, 0.36), loc=(0.0, 0.0, 0.24), color="snow_white",
    )
    for v in dome.verts:
        if v.co.z < SINK_Z:
            v.co.z = SINK_Z
    dome.normal_update()
    common.paint_faces(dome, [f for f in dome.faces if f.normal.z < 0.08], "snow_shadow")
    model.add(dome)

    # 8 main pine_snow_dark branches poking out through the snow dome (4-sided cones, 6 tris each -> 48 tris)
    branches = [
        (0.0, 0.22, 0.68, 0.44, 0.065),
        (0.85, 0.20, 0.64, 0.56, 0.060),
        (1.65, 0.24, 0.72, 0.36, 0.065),
        (2.45, 0.18, 0.58, 0.64, 0.058),
        (3.25, 0.22, 0.70, 0.42, 0.062),
        (4.10, 0.20, 0.62, 0.54, 0.060),
        (4.95, 0.24, 0.68, 0.34, 0.064),
        (5.75, 0.14, 0.40, 0.74, 0.055),
    ]
    for i, (ang, r_in, r_out, z_tip, r0) in enumerate(branches):
        a = ang + rng.uniform(-0.15, 0.15)
        start = (math.cos(a) * r_in, math.sin(a) * r_in, 0.16 + rng.uniform(-0.04, 0.08))
        end = (math.cos(a) * r_out, math.sin(a) * r_out, z_tip)
        model.add(veg.limb(start, end, r0, 0.0, "pine_snow_dark", segs=4))

        # 3 small secondary twig forks off the tallest sprigs (3-sided cones, 4 tris each -> 12 tris)
        if i in (1, 3, 7):
            fork_a = a + (0.45 if i % 2 else -0.45)
            mid = (
                (start[0] + end[0]) * 0.55,
                (start[1] + end[1]) * 0.55,
                (start[2] + end[2]) * 0.55,
            )
            tip = (
                mid[0] + math.cos(fork_a) * 0.26,
                mid[1] + math.sin(fork_a) * 0.26,
                mid[2] + 0.18,
            )
            model.add(veg.limb(mid, tip, r0 * 0.65, 0.0, "pine_snow_dark", segs=3))

    # Small central collider for the woody bush core
    hull = common.make_icosphere(subdiv=1, radius=(0.40, 0.36, 0.32), loc=(0.0, 0.0, 0.24))
    for v in hull.verts:
        if v.co.z < SINK_Z:
            v.co.z = SINK_Z
    model.add_collider("hull", hull)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
