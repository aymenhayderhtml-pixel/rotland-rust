"""tree_birch_lod1: medium LOD for birch tree."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_birch_lod1", "nature", 200


def build(model):
    model.add(common.make_frustum(0.14, 0.09, 0.0, 4.4, segments=6, color="white"))
    # 4 dark bark marks
    for z, angle in [(0.7, 0.3), (1.6, 2.4), (2.6, 4.1), (3.6, 1.2)]:
        mark = common.make_box((0.07, 0.02, 0.12), loc=(0.0, 0.12 - z * 0.006, z), color="bark_dark")
        common.transform(mark, rot=(0.0, 0.0, angle))
        model.add(mark)
    model.add_collider("trunk", common.make_frustum(0.12, 0.10, 0.0, 4.2, segments=6, color="white"))
    c = (0.1, 0.0, 4.4)
    model.add(common.make_icosphere(subdiv=1, jitter=0.15, seed=7, radius=(0.8, 0.8, 1.2),
                                    loc=c, color="leaf_mid"))
    for k, (dx, dy, dz, r) in enumerate([
        (0.4, 0.3, -0.2, 0.5),
        (-0.4, -0.2, 0.1, 0.5),
        (0.1, 0.3, 0.5, 0.45),
    ]):
        pos = (c[0] + dx, c[1] + dy, c[2] + dz)
        model.add(common.make_icosphere(subdiv=1, jitter=0.18, seed=70 + k, radius=(r, r, r * 0.9),
                                        loc=pos, color="leaf_light"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
