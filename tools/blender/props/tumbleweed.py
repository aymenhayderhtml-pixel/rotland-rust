"""tumbleweed: dry tangled desert bush (~0.55 m diameter, <= 150 tris). No collider."""
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

NAME, CATEGORY, BUDGET = "tumbleweed", "props", 150


def build(model):
    rng = random.Random(3501)
    cx, cy, cz = 0.0, 0.0, 0.27
    colors = ["dry_brush", "sand_dark", "dry_brush", "bark_light"]
    for i in range(16):
        # Golden-angle-ish distribution of crisscrossing dry twigs around a loose sphere
        theta = i * 2.39996 + rng.uniform(-0.25, 0.25)
        phi = math.acos(1.0 - 2.0 * (i + 0.5) / 16.0)
        r = rng.uniform(0.14, 0.24)
        ox = cx + math.sin(phi) * math.cos(theta) * r * 0.45
        oy = cy + math.sin(phi) * math.sin(theta) * r * 0.45
        oz = cz + math.cos(phi) * r * 0.40
        length = rng.uniform(0.32, 0.48)
        rot = (
            rng.uniform(0.4, 2.7),
            rng.uniform(-1.4, 1.4),
            rng.uniform(0.0, math.tau),
        )
        twig = common.make_frustum(0.012, 0.006, -length * 0.5, length * 0.5, segments=3, color=colors[i % len(colors)])
        common.transform(twig, loc=(ox, oy, oz), rot=rot)
        model.add(twig)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
