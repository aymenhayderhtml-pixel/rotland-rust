"""tree_pine_01: tall pine: jagged drooping tiers in two greens on a rooted trunk (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_pine_01", "nature", 1000

def build(model):
    rng = random.Random(1)
    veg.trunk(model, 0.22, 0.16, 2.6, "bark_dark", seed=1)
    model.add_collider("trunk", common.make_frustum(0.2, 0.2, 0.0, 2.4, segments=6, color="bark_dark"))
    # (radius, base height): two tiers per level, offset and turned, so the edge looks needled
    for i, (radius, z0) in enumerate([(2.5, 1.6), (2.1, 2.7), (1.75, 3.8), (1.35, 4.9), (0.95, 6.0), (0.55, 7.0)]):
        for k, color in enumerate(("pine_dark", "pine_mid")):
            off = (rng.uniform(-0.12, 0.12), rng.uniform(-0.12, 0.12))
            model.add(veg.tier(radius * (1.0 - 0.12 * k), z0 + 0.25 * k, 1.9, color, seed=10 * i + k, center=off))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
