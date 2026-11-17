"""tree_oak_big_lod2: far impostor LOD for big oak tree."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_oak_big_lod2", "nature", 100


def build(model):
    model.add(common.make_frustum(0.36, 0.20, 0.0, 3.0, segments=5, color="bark_dark"))
    model.add_collider("trunk", common.make_frustum(0.32, 0.22, 0.0, 3.0, segments=6, color="bark_dark"))
    c = (0.0, -0.1, 4.3)
    model.add(common.make_icosphere(subdiv=1, jitter=0.1, seed=5, radius=(2.2, 2.0, 1.4),
                                    loc=c, color="leaf_mid"))
    model.add(common.make_icosphere(subdiv=1, jitter=0.1, seed=6, radius=(1.3, 1.2, 1.0),
                                    loc=(c[0], c[1], c[2] + 0.7), color="leaf_light"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
