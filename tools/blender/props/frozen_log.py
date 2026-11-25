"""frozen_log: fallen timber log half-buried with snow cap along top (<= 150 tris). No collider."""
import math
import os
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "frozen_log", "props", 150


def build(model):
    # Main horizontal log resting slightly sunk into the snow
    log = common.make_frustum(0.19, 0.16, -0.78, 0.78, segments=6, color="bark_dark")
    common.transform(log, loc=(0.0, 0.0, 0.14), rot=(0.0, math.pi * 0.5, 0.08))
    model.add(log)

    # Pale cut wood end-caps
    cap_a = common.make_frustum(0.14, 0.14, -0.79, -0.77, segments=6, color="wood_crate")
    common.transform(cap_a, loc=(0.0, 0.0, 0.14), rot=(0.0, math.pi * 0.5, 0.08))
    model.add(cap_a)

    # Broken branch stub
    stub = common.make_frustum(0.065, 0.045, 0.0, 0.28, segments=5, color="bark_dark")
    common.transform(stub, loc=(0.18, 0.08, 0.20), rot=(-0.55, 0.35, 0.0))
    model.add(stub)

    # Three overlapping snow-cap blobs along the top of the log
    for idx, (x, y, z, rx, ry, rz) in enumerate([
        (-0.36, 0.02, 0.28, 0.34, 0.16, 0.08),
        (0.05, 0.00, 0.29, 0.30, 0.15, 0.09),
        (0.44, -0.02, 0.26, 0.28, 0.14, 0.07),
    ]):
        snow = common.make_icosphere(subdiv=1, jitter=0.16, seed=70 + idx,
                                     radius=(rx, ry, rz), loc=(x, y, z), color="snow_white")
        model.add(snow)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
