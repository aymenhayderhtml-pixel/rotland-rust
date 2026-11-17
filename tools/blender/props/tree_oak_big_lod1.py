"""tree_oak_big_lod1: medium LOD for big oak tree."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_oak_big_lod1", "nature", 300


def build(model):
    model.add(common.make_frustum(0.36, 0.22, 0.0, 3.0, segments=6, color="bark_dark"))
    model.add_collider("trunk", common.make_frustum(0.32, 0.22, 0.0, 3.0, segments=6, color="bark_dark"))
    model.add(veg.limb((0.0, 0.0, 2.6), (1.0, 0.7, 3.8), 0.14, 0.07, "bark_dark", segs=4))
    model.add(veg.limb((0.0, 0.0, 2.6), (-1.0, -0.6, 3.7), 0.14, 0.07, "bark_dark", segs=4))
    c = (0.0, -0.1, 4.3)
    model.add(common.make_icosphere(subdiv=1, jitter=0.15, seed=5, radius=(2.0, 1.8, 1.3),
                                    loc=c, color="leaf_mid"))
    for k, (dx, dy, dz, r, col) in enumerate([
        (1.2, 0.8, -0.3, 0.9, "leaf_dark"),
        (-1.2, -0.7, -0.2, 0.9, "leaf_dark"),
        (0.2, 1.0, 0.6, 0.85, "leaf_light"),
        (-0.4, -0.3, 0.7, 0.85, "leaf_light"),
        (0.8, -0.9, 0.1, 0.8, "leaf_mid"),
    ]):
        pos = (c[0] + dx, c[1] + dy, c[2] + dz)
        model.add(common.make_icosphere(subdiv=1, jitter=0.18, seed=50 + k, radius=(r, r, r * 0.85),
                                        loc=pos, color=col))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
