"""tree_birch_lod2: far impostor LOD for birch tree."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_birch_lod2", "nature", 80


def build(model):
    model.add(common.make_frustum(0.14, 0.08, 0.0, 4.4, segments=4, color="white"))
    model.add_collider("trunk", common.make_frustum(0.12, 0.10, 0.0, 4.2, segments=6, color="white"))
    c = (0.1, 0.0, 4.4)
    model.add(common.make_icosphere(subdiv=1, jitter=0.1, seed=7, radius=(0.9, 0.9, 1.4),
                                    loc=c, color="leaf_light"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
