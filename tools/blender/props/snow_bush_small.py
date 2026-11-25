"""snow_bush_small: compact winter shrub with dark twigs and snow-dusted pine clumps (<= 140 tris). No collider."""
import os
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "snow_bush_small", "props", 140


def build(model):
    # Four short woody stems fanning out from the ground
    stems = [
        ((0.0, 0.0, -0.02), (0.12, 0.05, 0.0), 0.34),
        ((0.0, 0.0, -0.02), (-0.28, 0.20, 1.2), 0.30),
        ((0.0, 0.0, -0.02), (0.24, -0.26, 2.5), 0.28),
        ((0.0, 0.0, -0.02), (-0.18, -0.24, 4.1), 0.26),
    ]
    for loc, rot, h in stems:
        twig = common.make_frustum(0.024, 0.010, 0.0, h, segments=3, color="bark_dark")
        common.transform(twig, loc=loc, rot=rot)
        model.add(twig)

    # Four foliage clumps with snow_white top faces and pine_snow_dark undersides
    clumps = [
        ((0.02, 0.02, 0.30), (0.20, 0.18, 0.14), 51),
        ((-0.14, 0.10, 0.22), (0.15, 0.14, 0.11), 52),
        ((0.14, -0.10, 0.20), (0.14, 0.14, 0.11), 53),
        ((-0.06, -0.14, 0.18), (0.13, 0.12, 0.10), 54),
    ]
    for loc, rad, seed in clumps:
        bm = common.make_icosphere(subdiv=1, jitter=0.22, seed=seed, radius=rad, loc=loc, color="pine_snow_dark")
        bm.normal_update()
        common.paint_faces(bm, [f for f in bm.faces if f.normal.z > 0.35], "snow_white")
        model.add(bm)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
