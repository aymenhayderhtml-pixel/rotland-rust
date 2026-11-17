"""tree_oak_big: big oak: thick flared trunk, heavy limbs and a wide crown of many clusters (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_oak_big", "nature", 1600

def build(model):
    veg.trunk(model, 0.36, 0.22, 3.0, "bark_dark", seed=5, roots=6)
    model.add_collider("trunk", common.make_frustum(0.32, 0.22, 0.0, 3.0, segments=6, color="bark_dark"))
    veg.leafy_crown(model, seed=5, center=(0.0, -0.1, 4.3), size=(2.6, 2.3, 1.6),
                    colors=("leaf_dark", "leaf_mid", "leaf_light"), count=14, blob_r=0.9,
                    branch_from=2.6, bark="bark_dark", limbs=5, limb_r=0.16)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
