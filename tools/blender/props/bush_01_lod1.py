"""bush_01_lod1: medium LOD for bush (3 subdiv-1 blobs, no berries)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "bush_01_lod1", "nature", 100


def build(model):
    # 3 icospheres at subdiv 1 (20 tris each = 60 tris total)
    model.add(common.make_icosphere(subdiv=1, jitter=0.15, seed=20, radius=(0.42, 0.42, 0.36),
                                    loc=(0.0, 0.0, 0.36), color="leaf_mid"))
    model.add(common.make_icosphere(subdiv=1, jitter=0.15, seed=21, radius=(0.35, 0.35, 0.30),
                                    loc=(0.28, 0.15, 0.30), color="leaf_dark"))
    model.add(common.make_icosphere(subdiv=1, jitter=0.15, seed=22, radius=(0.32, 0.32, 0.28),
                                    loc=(-0.25, -0.15, 0.32), color="leaf_light"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
