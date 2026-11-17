"""tree_broadleaf_01_lod1: medium LOD for broadleaf tree (subdiv-1 leaf blobs)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_broadleaf_01_lod1", "nature", 250


def build(model):
    model.add(common.make_frustum(0.25, 0.16, 0.0, 3.2, segments=6, color="bark_light"))
    model.add_collider("trunk", common.make_frustum(0.21, 0.21, 0.0, 3.2, segments=6, color="bark_light"))
    # Two limbs
    model.add(veg.limb((0.0, 0.0, 2.7), (0.7, 0.5, 3.8), 0.10, 0.05, "bark_light", segs=4))
    model.add(veg.limb((0.0, 0.0, 2.8), (-0.6, -0.4, 3.7), 0.10, 0.05, "bark_light", segs=4))
    # Core crown blob + 4 outer clusters at subdiv 1
    c = (0.2, -0.1, 4.3)
    model.add(common.make_icosphere(subdiv=1, jitter=0.15, seed=4, radius=(1.4, 1.3, 1.2),
                                    loc=c, color="leaf_mid"))
    for k, (dx, dy, dz, r, col) in enumerate([
        (0.8, 0.6, -0.2, 0.75, "leaf_dark"),
        (-0.7, -0.5, -0.1, 0.75, "leaf_dark"),
        (0.1, 0.7, 0.5, 0.7, "leaf_light"),
        (-0.3, -0.2, 0.6, 0.7, "leaf_light"),
    ]):
        pos = (c[0] + dx, c[1] + dy, c[2] + dz)
        model.add(common.make_icosphere(subdiv=1, jitter=0.18, seed=40 + k, radius=(r, r, r * 0.85),
                                        loc=pos, color=col))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
