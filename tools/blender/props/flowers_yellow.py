"""flowers_yellow: yellow flowers: stems with leaves and five-petal heads (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "flowers_yellow", "nature", 250

HEAD_COLOR = "brass"
# (x, y, stem height, lean, yaw)
STEMS = [(0.0, 0.0, 0.34, 0.15, 0.4), (0.10, -0.06, 0.28, 0.30, 2.2),
         (-0.10, 0.04, 0.40, 0.25, 3.6), (0.05, 0.10, 0.25, 0.40, 5.0), (-0.04, -0.09, 0.31, 0.2, 1.3)]


def build(model):
    for x, y, height, lean, yaw in STEMS:
        rot = (lean, 0.0, yaw)
        stem = common.make_frustum(0.012, 0.009, 0.0, height, segments=3, color="grass")
        common.transform(stem, rot=rot, loc=(x, y, 0.0))
        model.add(stem)
        # The head sits on the stem's tip: five petals round a brown centre
        tip = Vector((0.0, 0.0, height))
        tip.rotate(Euler(rot, "XYZ"))
        head = common.make_frustum(0.012, 0.06, 0.0, 0.03, segments=5, color=HEAD_COLOR)
        common.transform(head, loc=(x + tip.x, y + tip.y, tip.z))
        model.add(head)
        model.add(common.make_box((0.022, 0.022, 0.02), loc=(x + tip.x, y + tip.y, tip.z + 0.03), color="bark_light"))
    # Leaves round the base
    for i in range(5):
        model.add(veg.blade(0.0, 0.0, 0.16, 1.0, i * 1.26, "leaf_mid", width=0.04))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
