"""tree_pine_02_lod2: far impostor LOD for short wide pine."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_pine_02_lod2", "nature", 80


def build(model):
    model.add(common.make_frustum(0.26, 0.16, 0.0, 2.0, segments=5, color="bark_light"))
    model.add_collider("trunk", common.make_frustum(0.24, 0.24, 0.0, 2.0, segments=6, color="bark_light"))
    for radius, z0, z1, color in [
        (2.8, 1.4, 3.5, "pine_mid"),
        (2.0, 2.8, 4.9, "pine_dark"),
        (1.1, 4.2, 6.2, "pine_mid"),
    ]:
        model.add(common.make_frustum(radius, 0.0, z0, z1, segments=5, color=color))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
