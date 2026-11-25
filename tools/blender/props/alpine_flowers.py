"""alpine_flowers: hardy low alpine moss cushion with small mountain blossoms (<= 150 tris). No collider."""
import os
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "alpine_flowers", "props", 150


def build(model):
    # Low hardy cushion base
    model.add(common.make_icosphere(subdiv=1, jitter=0.18, seed=121,
                                    radius=(0.22, 0.19, 0.08), loc=(0.00, 0.00, 0.04), color="leaf_dark"))
    model.add(common.make_icosphere(subdiv=1, jitter=0.18, seed=122,
                                    radius=(0.14, 0.13, 0.06), loc=(0.14, -0.06, 0.03), color="grass"))

    # Five short flower stems with bright alpine blossoms
    flowers = [
        (0.02, 0.02, 0.18, "water_blue"),
        (-0.10, 0.08, 0.15, "white"),
        (0.11, -0.07, 0.14, "wheat_gold"),
        (0.08, 0.10, 0.16, "water_blue"),
        (-0.07, -0.09, 0.14, "white"),
    ]
    for x, y, h, color in flowers:
        stem = common.make_frustum(0.008, 0.006, 0.04, h, segments=3, color="grass")
        common.transform(stem, loc=(x, y, 0.0))
        model.add(stem)

        cup = common.make_frustum(0.012, 0.036, h - 0.01, h + 0.03, segments=4, color=color)
        common.transform(cup, loc=(x, y, 0.0))
        model.add(cup)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
