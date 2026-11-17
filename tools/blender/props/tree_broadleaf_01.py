"""tree_broadleaf_01: broadleaf tree: rooted trunk, four limbs and a lumpy crown of leaf clusters (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_broadleaf_01", "nature", 1300

def build(model):
    veg.trunk(model, 0.25, 0.16, 3.2, "bark_light", seed=4)
    model.add_collider("trunk", common.make_frustum(0.21, 0.21, 0.0, 3.2, segments=6, color="bark_light"))
    veg.leafy_crown(model, seed=4, center=(0.2, -0.1, 4.3), size=(1.9, 1.7, 1.5),
                    colors=("leaf_dark", "leaf_mid", "leaf_light"), count=11, blob_r=0.75,
                    branch_from=2.7, bark="bark_light")


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
