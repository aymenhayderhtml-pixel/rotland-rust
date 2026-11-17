"""fern: fern: fronds that rise and then curl down at the tips (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "fern", "nature", 160

def build(model):
    for i in range(12):
        yaw = i * 0.52 + (i % 2) * 0.2
        lean = 0.7 + (i % 3) * 0.15
        color = "leaf_dark" if i % 2 == 0 else "leaf_mid"
        # Inner half rises, outer half bends down from its tip
        rot = Euler((lean, 0.0, yaw), "XYZ")
        tip = rot.to_matrix() @ Vector((0.0, 0.0, 0.32))
        model.add(veg.blade(0.0, 0.0, 0.32, lean, yaw, color, width=0.07))
        outer = common.make_frustum(0.06, 0.0, 0.0, 0.30, segments=3, color=color)
        common.transform(outer, rot=(lean + 0.7, 0.0, yaw), loc=tuple(tip))
        model.add(outer)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
