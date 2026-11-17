"""pine_tall: very tall thin pine: many small jagged tiers up a long trunk (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "pine_tall", "nature", 1000

def build(model):
    rng = random.Random(3)
    veg.trunk(model, 0.15, 0.08, 8.6, "bark_dark", seed=3, roots=4, segs=6)
    model.add_collider("trunk", common.make_frustum(0.14, 0.10, 0.0, 8.6, segments=6, color="bark_dark"))
    for i in range(9):
        radius = 1.75 - i * 0.17
        z0 = 1.2 + i * 0.78
        off = (rng.uniform(-0.1, 0.1), rng.uniform(-0.1, 0.1))
        color = "pine_dark" if i % 2 == 0 else "pine_mid"
        model.add(veg.tier(radius, z0, 1.6, color, seed=30 + i, center=off, points=9, droop=0.25))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
