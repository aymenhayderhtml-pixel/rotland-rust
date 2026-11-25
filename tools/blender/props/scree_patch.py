"""scree_patch: low scatter of angular mountain rock fragments (~1.0 m across, <= 180 tris). No collider."""
import os
import random
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "scree_patch", "props", 180


def build(model):
    rng = random.Random(3510)
    colors = ["rock_dark", "rock_mid", "rock_light"]
    chips = [
        ((0.02, 0.00, 0.06), (0.20, 0.15, 0.09)),
        ((-0.26, 0.14, 0.05), (0.16, 0.12, 0.08)),
        ((0.28, -0.12, 0.04), (0.15, 0.11, 0.07)),
        ((0.18, 0.24, 0.04), (0.13, 0.10, 0.06)),
        ((-0.18, -0.22, 0.04), (0.14, 0.11, 0.07)),
        ((-0.38, -0.06, 0.03), (0.11, 0.09, 0.05)),
        ((0.36, 0.12, 0.03), (0.10, 0.08, 0.05)),
        ((0.00, -0.34, 0.03), (0.12, 0.09, 0.05)),
    ]
    for idx, (loc, rad) in enumerate(chips):
        rot = (rng.uniform(-0.25, 0.25), rng.uniform(-0.25, 0.25), rng.uniform(0.0, 6.28))
        rock = common.make_icosphere(
            subdiv=1, jitter=0.28, seed=110 + idx,
            radius=rad, loc=loc, rot=rot, color=colors[idx % len(colors)],
        )
        model.add(rock)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
