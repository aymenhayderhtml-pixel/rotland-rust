"""pine_tall_lod2: far impostor LOD for very tall thin pine."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "pine_tall_lod2", "nature", 70


def build(model):
    model.add(common.make_frustum(0.15, 0.08, 0.0, 8.6, segments=4, color="bark_dark"))
    model.add_collider("trunk", common.make_frustum(0.14, 0.10, 0.0, 8.6, segments=6, color="bark_dark"))
    for radius, z0, z1, color in [
        (1.6, 1.8, 3.8, "pine_dark"),
        (1.2, 3.6, 5.6, "pine_mid"),
        (0.8, 5.4, 7.4, "pine_dark"),
        (0.5, 7.0, 8.8, "pine_mid"),
    ]:
        model.add(common.make_frustum(radius, 0.0, z0, z1, segments=5, color=color))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
