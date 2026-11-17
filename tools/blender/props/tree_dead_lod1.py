"""tree_dead_lod1: medium LOD for dead tree (fewer limbs, no twigs)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_dead_lod1", "nature", 100


def build(model):
    model.add(common.make_frustum(0.22, 0.06, 0.0, 4.8, segments=5, color="bark_dark"))
    model.add_collider("trunk", common.make_frustum(0.2, 0.06, 0.0, 4.8, segments=6, color="bark_dark"))
    for z, tilt, yaw, length in [(1.6, 1.0, 1.4, 1.2), (2.4, 0.9, 0.5, 1.8), (3.3, 1.1, 2.6, 1.9), (4.0, 0.8, 4.0, 1.2)]:
        start = Vector((0.0, 0.0, z))
        end = start + Euler((0.0, tilt, yaw), "XYZ").to_matrix() @ Vector((0.0, 0.0, length))
        model.add(veg.limb(start, end, 0.07, 0.02, "bark_dark", segs=4))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
