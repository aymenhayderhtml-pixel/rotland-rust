"""cattails: dense swamp cattail cluster with dark brown heads and olive blades (<= 160 tris). No collider."""
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

NAME, CATEGORY, BUDGET = "cattails", "props", 160
SINK = -0.04


def build(model):
    rng = random.Random(3507)

    # Low muddy root mound
    model.add(common.make_frustum(0.18, 0.10, SINK, 0.08, segments=5, color="swamp_mud"))

    # 6 arching green/moss base blades
    for i in range(6):
        a = math.tau * i / 6.0 + rng.uniform(-0.2, 0.2)
        dist = rng.uniform(0.04, 0.11)
        h = rng.uniform(0.45, 0.72)
        lean = rng.uniform(0.20, 0.42)
        yaw = a - math.pi * 0.5 + rng.uniform(-0.2, 0.2)
        blade = veg.blade(math.cos(a) * dist, math.sin(a) * dist, h, lean, yaw,
                          "swamp_moss" if i % 2 == 0 else "leaf_dark", width=0.036)
        common.transform(blade, loc=(0.0, 0.0, SINK))
        model.add(blade)

    # 5 upright cattail stalks with prominent dark brown seed heads
    for i in range(5):
        a = math.tau * i / 5.0 + rng.uniform(-0.25, 0.25)
        dist = rng.uniform(0.02, 0.10)
        bx, by = math.cos(a) * dist, math.sin(a) * dist
        h = rng.uniform(0.88, 1.18)
        lean = rng.uniform(0.04, 0.14)
        yaw = a - math.pi * 0.5
        stalk = common.make_frustum(0.014, 0.008, 0.0, h, segments=3, color="reed_tan")
        common.transform(stalk, rot=(lean, 0.0, yaw), loc=(bx, by, SINK))
        model.add(stalk)

        head_z0 = h * 0.64
        head = common.make_frustum(0.032, 0.028, head_z0, head_z0 + 0.20, segments=4, color="swamp_mud")
        common.transform(head, rot=(lean, 0.0, yaw), loc=(bx, by, SINK))
        model.add(head)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
