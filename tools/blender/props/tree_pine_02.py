"""tree_pine_02: short wide pine: three broad jagged tiers on a thick rooted trunk (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_pine_02", "nature", 1000

def build(model):
    rng = random.Random(2)
    veg.trunk(model, 0.26, 0.18, 2.0, "bark_light", seed=2)
    model.add_collider("trunk", common.make_frustum(0.24, 0.24, 0.0, 2.0, segments=6, color="bark_light"))
    for i, (radius, z0) in enumerate([(3.0, 1.2), (2.6, 2.1), (2.1, 3.0), (1.6, 3.9), (1.0, 4.8)]):
        for k, color in enumerate(("pine_mid", "pine_dark")):
            off = (rng.uniform(-0.2, 0.2), rng.uniform(-0.2, 0.2))
            model.add(veg.tier(radius * (1.0 - 0.12 * k), z0 + 0.22 * k, 2.0, color, seed=20 * i + k,
                               center=off, points=13, droop=0.4))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
