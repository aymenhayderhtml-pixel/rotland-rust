"""tree_pine_01_lod2: far impostor/low-poly LOD for tall pine."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_pine_01_lod2", "nature", 80


def build(model):
    # Minimal trunk
    model.add(common.make_frustum(0.22, 0.14, 0.0, 2.6, segments=5, color="bark_dark"))
    model.add_collider("trunk", common.make_frustum(0.2, 0.2, 0.0, 2.4, segments=6, color="bark_dark"))
    # 3 simple cones (5-sided)
    for radius, z0, z1, color in [
        (2.3, 1.8, 4.0, "pine_dark"),
        (1.6, 3.6, 5.8, "pine_mid"),
        (0.9, 5.4, 7.5, "pine_dark"),
    ]:
        model.add(common.make_frustum(radius, 0.0, z0, z1, segments=5, color=color))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
