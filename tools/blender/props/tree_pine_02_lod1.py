"""tree_pine_02_lod1: medium LOD for short wide pine."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_pine_02_lod1", "nature", 200


def build(model):
    rng = random.Random(2)
    model.add(common.make_frustum(0.26, 0.18, 0.0, 2.0, segments=6, color="bark_light"))
    model.add_collider("trunk", common.make_frustum(0.24, 0.24, 0.0, 2.0, segments=6, color="bark_light"))
    for i, (radius, z0, color) in enumerate([
        (2.9, 1.3, "pine_mid"),
        (2.3, 2.4, "pine_dark"),
        (1.7, 3.5, "pine_mid"),
        (1.0, 4.6, "pine_dark"),
    ]):
        off = (rng.uniform(-0.1, 0.1), rng.uniform(-0.1, 0.1))
        model.add(veg.tier(radius, z0, 2.1, color, seed=20 * i, points=6, droop=0.35, center=off))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
