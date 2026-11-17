"""tree_pine_01_lod1: medium LOD for tall pine (fewer tiers and segments)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_pine_01_lod1", "nature", 200


def build(model):
    rng = random.Random(1)
    # Simple trunk without root wedges
    model.add(common.make_frustum(0.22, 0.16, 0.0, 2.6, segments=6, color="bark_dark"))
    model.add_collider("trunk", common.make_frustum(0.2, 0.2, 0.0, 2.4, segments=6, color="bark_dark"))
    # 4 single tiers instead of 12 sub-tiers
    for i, (radius, z0, color) in enumerate([
        (2.4, 1.8, "pine_dark"),
        (1.9, 3.2, "pine_mid"),
        (1.35, 4.6, "pine_dark"),
        (0.75, 6.0, "pine_mid"),
    ]):
        off = (rng.uniform(-0.08, 0.08), rng.uniform(-0.08, 0.08))
        model.add(veg.tier(radius, z0, 2.1, color, seed=10 * i, points=6, droop=0.3, center=off))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
