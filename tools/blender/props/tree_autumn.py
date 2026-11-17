"""tree_autumn: autumn tree: the broadleaf shape with an orange and red crown (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_autumn", "nature", 1300

def build(model):
    veg.trunk(model, 0.25, 0.16, 3.2, "bark_dark", seed=6)
    model.add_collider("trunk", common.make_frustum(0.21, 0.21, 0.0, 3.2, segments=6, color="bark_dark"))
    veg.leafy_crown(model, seed=6, center=(-0.1, 0.1, 4.3), size=(1.9, 1.8, 1.5),
                    colors=("leaf_red", "leaf_orange", "leaf_orange"), count=11, blob_r=0.75,
                    branch_from=2.7, bark="bark_dark")


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
