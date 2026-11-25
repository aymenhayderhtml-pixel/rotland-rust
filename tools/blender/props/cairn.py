"""cairn: stacked mountain trail-marker stone cairn (~0.7 m tall, <= 140 tris). No collider."""
import os
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "cairn", "props", 140


def build(model):
    # Six flat/angular stones stacked from wide base to small capstone
    stones = [
        ((0.00, 0.00, 0.07), (0.28, 0.24, 0.10), (0.04, -0.05, 0.2), "rock_dark"),
        ((0.01, -0.01, 0.20), (0.22, 0.19, 0.09), (-0.05, 0.04, 1.1), "rock_mid"),
        ((-0.01, 0.01, 0.32), (0.17, 0.15, 0.08), (0.06, 0.03, 2.3), "rock_light"),
        ((0.01, 0.00, 0.43), (0.13, 0.12, 0.07), (-0.04, -0.06, 0.7), "rock_mid"),
        ((0.00, -0.01, 0.53), (0.10, 0.09, 0.06), (0.05, 0.04, 1.8), "rock_dark"),
        ((0.00, 0.00, 0.62), (0.07, 0.06, 0.07), (0.02, -0.03, 2.9), "rock_light"),
    ]
    for idx, (loc, rad, rot, color) in enumerate(stones):
        rock = common.make_icosphere(
            subdiv=1, jitter=0.22, seed=130 + idx,
            radius=rad, loc=loc, rot=rot, color=color,
        )
        model.add(rock)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
