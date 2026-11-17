"""tree_dead: dead tree: bare trunk with roots, broken limbs and twigs (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "tree_dead", "nature", 600

def build(model):
    rng = random.Random(8)
    veg.trunk(model, 0.22, 0.05, 4.8, "bark_dark", seed=8, roots=5, segs=6)
    model.add_collider("trunk", common.make_frustum(0.2, 0.06, 0.0, 4.8, segments=6, color="bark_dark"))
    # (base height, tilt, yaw, length): limbs of different length, each with a twig or two
    for z, tilt, yaw, length in [(1.6, 1.0, 1.4, 1.2), (2.0, 0.9, 0.5, 1.8), (2.8, 1.1, 2.6, 2.2),
                                 (3.5, 0.8, 4.0, 1.4), (4.2, 1.2, 5.3, 0.9), (3.1, 0.7, 5.9, 1.1)]:
        start = Vector((0.0, 0.0, z))
        end = start + Euler((0.0, tilt, yaw), "XYZ").to_matrix() @ Vector((0.0, 0.0, length))
        model.add(veg.limb(start, end, 0.075, 0.02, "bark_dark", segs=5))
        for f in (0.55, 0.8)[: 1 + (length > 1.3)]:
            p = start.lerp(end, f)
            twig = p + Euler((rng.uniform(-0.6, 0.6), tilt - 0.6, yaw + rng.uniform(-0.8, 0.8)), "XYZ").to_matrix() \
                @ Vector((0.0, 0.0, length * 0.35))
            model.add(veg.limb(p, twig, 0.03, 0.008, "bark_dark", segs=4))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
