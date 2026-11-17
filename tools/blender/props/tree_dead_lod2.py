"""tree_dead_lod2: far impostor LOD for dead tree."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_dead_lod2", "nature", 50


def build(model):
    model.add(common.make_frustum(0.20, 0.06, 0.0, 4.8, segments=4, color="bark_dark"))
    model.add_collider("trunk", common.make_frustum(0.2, 0.06, 0.0, 4.8, segments=6, color="bark_dark"))
    for z, tilt, yaw, length in [(2.0, 0.9, 0.5, 1.5), (3.4, 1.1, 2.6, 1.6)]:
        start = Vector((0.0, 0.0, z))
        end = start + Euler((0.0, tilt, yaw), "XYZ").to_matrix() @ Vector((0.0, 0.0, length))
        model.add(veg.limb(start, end, 0.06, 0.02, "bark_dark", segs=3))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
