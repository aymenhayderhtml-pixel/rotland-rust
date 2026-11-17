"""tree_birch: birch: slim white trunk with dark marks and a tall, light crown (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_birch", "nature", 1000

def build(model):
    veg.trunk(model, 0.14, 0.09, 4.4, "white", seed=7, roots=4, segs=7)
    # Dark bark marks: thin boxes at the trunk's edge, turned to different angles up the trunk
    for z, angle in [(0.5, 0.3), (0.9, 3.0), (1.4, 2.4), (1.9, 5.1), (2.4, 4.1), (2.9, 0.9), (3.4, 1.2), (3.9, 3.6)]:
        mark = common.make_box((0.07, 0.02, 0.12), loc=(0.0, 0.12 - z * 0.006, z), color="bark_dark")
        common.transform(mark, rot=(0.0, 0.0, angle))
        model.add(mark)
    model.add_collider("trunk", common.make_frustum(0.12, 0.10, 0.0, 4.2, segments=6, color="white"))
    veg.leafy_crown(model, seed=7, center=(0.1, 0.0, 4.4), size=(1.0, 1.0, 1.5),
                    colors=("leaf_mid", "leaf_light", "leaf_light"), count=8, blob_r=0.5,
                    branch_from=3.6, bark="white", limbs=3, limb_r=0.06)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
