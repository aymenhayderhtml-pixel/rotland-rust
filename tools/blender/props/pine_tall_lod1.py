"""pine_tall_lod1: medium LOD for very tall thin pine."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "pine_tall_lod1", "nature", 180


def build(model):
    rng = random.Random(3)
    model.add(common.make_frustum(0.15, 0.08, 0.0, 8.6, segments=5, color="bark_dark"))
    model.add_collider("trunk", common.make_frustum(0.14, 0.10, 0.0, 8.6, segments=6, color="bark_dark"))
    for i in range(5):
        radius = 1.7 - i * 0.28
        z0 = 1.3 + i * 1.4
        off = (rng.uniform(-0.05, 0.05), rng.uniform(-0.05, 0.05))
        color = "pine_dark" if i % 2 == 0 else "pine_mid"
        model.add(veg.tier(radius, z0, 2.0, color, seed=30 + i, points=6, droop=0.25, center=off))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
