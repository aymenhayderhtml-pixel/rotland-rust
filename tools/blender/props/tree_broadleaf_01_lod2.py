"""tree_broadleaf_01_lod2: far impostor LOD for broadleaf tree."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_broadleaf_01_lod2", "nature", 90


def build(model):
    model.add(common.make_frustum(0.24, 0.14, 0.0, 3.2, segments=4, color="bark_light"))
    model.add_collider("trunk", common.make_frustum(0.21, 0.21, 0.0, 3.2, segments=6, color="bark_light"))
    c = (0.2, -0.1, 4.3)
    # Main crown + top crown blob
    model.add(common.make_icosphere(subdiv=1, jitter=0.1, seed=4, radius=(1.6, 1.5, 1.3),
                                    loc=c, color="leaf_mid"))
    model.add(common.make_icosphere(subdiv=1, jitter=0.1, seed=5, radius=(1.0, 0.9, 0.9),
                                    loc=(c[0], c[1], c[2] + 0.6), color="leaf_light"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
